<!-- AI-generated (Claude) -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { app } from "$lib/stores/app.svelte.ts";
  import { loadDcConnections, saveDcConnection, type DcConnections, type Transport } from "$lib/dcConnections.ts";

  let { open, onClose }: { open: boolean; onClose: () => void } = $props();

  type Step = "list" | "setup" | "progress" | "review" | "result";
  let step = $state<Step>("list");

  type KnownDevice = { vendor: string; product: string; serial: string; nickname: string };
  let knownDevices = $state<KnownDevice[]>([]);
  let selectedKnownDeviceKey = $state("");
  let isKnownDevice = $state(false);
  let dcConnections = $state<DcConnections>({});
  let cachedBleAddress = $state<string | null>(null);

  function deviceKey(vendor: string, product: string, serial: string): string {
    return `${vendor} ${product} ${serial}`;
  }

  function formatSerial(serial: number): string {
    return serial.toString(16).padStart(8, "0");
  }

  let vendors = $state<string[]>([]);
  let vendor = $state("");
  let models = $state<{ product: string; transports: string[] }[]>([]);
  let model = $state("");
  let transport = $state<Transport>("BLE");
  let serialPort = $state("");
  let serialPorts = $state<string[]>([]);
  let bluetoothAddress = $state("");
  let bleDevices = $state<{ name: string; address: string }[]>([]);
  let selectedBleDevice = $state<string | null>(null);
  let bleScanning = $state(false);
  let bleScanTimedOut = $state(false);
  let bleScanTimer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    if (selectedBleDevice) {
      bleScanning = false;
      bleScanTimedOut = false;
    }
  });

  let progressCurrent = $state(0);
  let progressMaximum = $state(0);
  let resultAdded = $state(0);
  let resultSkipped = $state(0);
  let resultCancelled = $state(false);
  let errorMsg = $state<string | null>(null);
  let statusLabel = $state("Connecting…");

  type DiveSummary = { date: string; durationSec: number; maxDepthM: number };
  let pendingDives = $state<DiveSummary[]>([]);
  let selectedDives = $state<boolean[]>([]);
  let selectedCount = $derived(selectedDives.filter(Boolean).length);

  let unlisteners: (() => void)[] = [];

  function currentAddress(): string {
    if (transport === "Serial") return serialPort;
    if (transport === "Bluetooth") return bluetoothAddress;
    if (transport === "BLE") return selectedBleDevice ?? "";
    return "";
  }

  onMount(async () => {
    knownDevices = await invoke<KnownDevice[]>("list_known_devices");
    dcConnections = await loadDcConnections();
    if (knownDevices.length === 0) {
      step = "setup";
    } else {
      // Backend already orders knownDevices most-recently-seen first.
      selectedKnownDeviceKey = deviceKey(knownDevices[0].vendor, knownDevices[0].product, knownDevices[0].serial);
    }
    vendors = await invoke<string[]>("list_dc_vendors");
    unlisteners.push(await listen<{ name: string; address: string }>("dc-ble-found", (e) => {
      const existing = bleDevices.find((d) => d.address === e.payload.address);
      if (!existing) bleDevices = [...bleDevices, e.payload];
      if (cachedBleAddress && !selectedBleDevice && e.payload.address === cachedBleAddress) {
        selectedBleDevice = e.payload.address;
      }
    }));
    unlisteners.push(await listen<{ model: number; firmware: number; serial: number }>("dc-devinfo", (e) => {
      statusLabel = `Connected: ${vendor} ${model}`;
      saveDcConnection(deviceKey(vendor, model, formatSerial(e.payload.serial)), transport, currentAddress());
    }));
    unlisteners.push(await listen<{ diveNumber: number; date: string | null; added: boolean }>("dc-dive", (e) => {
      if (e.payload.date) {
        statusLabel = `Dive ${e.payload.diveNumber}: ${e.payload.date}`;
      } else {
        statusLabel = `Skipping dive ${e.payload.diveNumber}…`;
      }
    }));
    unlisteners.push(await listen<{ current: number; maximum: number }>("dc-progress", (e) => {
      progressCurrent = e.payload.current;
      progressMaximum = e.payload.maximum;
    }));
    unlisteners.push(await listen<{
      dives: DiveSummary[];
      skipped: number;
      cancelled: boolean;
    }>("dc-complete", (e) => {
      resultSkipped = e.payload.skipped;
      resultCancelled = e.payload.cancelled;
      errorMsg = null;
      if (e.payload.dives.length > 0) {
        // Show whatever was fetched for review, even if the download was
        // cancelled partway through — a cancel shouldn't discard progress
        // that was already downloaded.
        pendingDives = e.payload.dives;
        selectedDives = e.payload.dives.map(() => true);
        step = "review";
      } else {
        if (!e.payload.cancelled) {
          // Nothing new to review, but the device's fingerprint cutoff still
          // needs to advance, or every future download re-scans its full
          // history. commit_dc_download always does this regardless of an
          // empty selection.
          invoke("commit_dc_download", { selectedIndices: [] }).catch(() => {});
        }
        resultAdded = 0;
        step = "result";
      }
    }));
    unlisteners.push(await listen<{ message: string }>("dc-error", (e) => {
      errorMsg = e.payload.message;
    }));
  });

  onDestroy(() => {
    unlisteners.forEach((u) => u());
    clearTimeout(bleScanTimer);
  });

  async function onVendorChange() {
    models = await invoke<{ product: string; transports: string[] }[]>("list_dc_models", { vendor });
    model = models[0]?.product ?? "";
    updateTransportDefault();
    await onTransportChange();
  }

  function updateTransportDefault() {
    const m = models.find((m) => m.product === model);
    if (m?.transports.includes("BLE")) transport = "BLE";
    else if (m?.transports.includes("Serial")) transport = "Serial";
    else if (m?.transports.includes("Bluetooth")) transport = "Bluetooth";
    else transport = "USBHID";
  }

  async function onModelChange() {
    updateTransportDefault();
    await onTransportChange();
  }

  async function onTransportChange() {
    if (transport === "Serial") serialPorts = await invoke<string[]>("list_serial_ports");
  }

  function goSetupNew() {
    isKnownDevice = false;
    cachedBleAddress = null;
    vendor = "";
    model = "";
    models = [];
    serialPort = "";
    bluetoothAddress = "";
    selectedBleDevice = null;
    bleDevices = [];
    bleScanning = false;
    bleScanTimedOut = false;
    clearTimeout(bleScanTimer);
    step = "setup";
  }

  function applyCachedConnectionOrDefault(serial: string) {
    const supported = models.find((m) => m.product === model)?.transports ?? [];
    const cached = dcConnections[deviceKey(vendor, model, serial)];
    if (cached && supported.includes(cached.lastTransport)) {
      transport = cached.lastTransport;
    } else {
      updateTransportDefault();
    }
    serialPort = "";
    bluetoothAddress = "";
    selectedBleDevice = null;
    cachedBleAddress = null;
    const addr = cached?.addresses[transport];
    if (addr) {
      if (transport === "Serial") serialPort = addr;
      else if (transport === "Bluetooth") bluetoothAddress = addr;
      else if (transport === "BLE") cachedBleAddress = addr;
    }
  }

  async function selectKnownDevice(d: KnownDevice) {
    vendor = d.vendor;
    models = await invoke<{ product: string; transports: string[] }[]>("list_dc_models", { vendor });
    model = d.product;
    isKnownDevice = true;
    bleDevices = [];
    errorMsg = null;
    applyCachedConnectionOrDefault(d.serial);
    step = "setup";
    await onTransportChange();
    if (transport === "BLE") scanBle();
  }

  async function connectToSelectedKnownDevice() {
    const d = knownDevices.find((d) => deviceKey(d.vendor, d.product, d.serial) === selectedKnownDeviceKey);
    if (d) await selectKnownDevice(d);
  }

  async function startDownload() {
    errorMsg = null;
    statusLabel = "Connecting…";
    step = "progress";
    progressCurrent = 0;
    progressMaximum = 0;
    pendingDives = [];
    const transportArg = transport === "Serial"
      ? { kind: "Serial", port: serialPort }
      : transport === "Bluetooth"
      ? { kind: "Bluetooth", address: bluetoothAddress }
      : transport === "BLE"
      ? { kind: "Ble", address: selectedBleDevice ?? "" }
      : { kind: "UsbHid" };
    try {
      await invoke("start_dc_download", { vendor, model, transport: transportArg });
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : String(e);
      step = "result";
    }
  }

  async function saveToLogbook() {
    try {
      const selectedIndices = selectedDives
        .map((isSelected, i) => (isSelected ? i : -1))
        .filter((i) => i !== -1);
      const added = await invoke<number>("commit_dc_download", { selectedIndices });
      resultAdded = added;
      await app.startup();
      step = "result";
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : String(e);
      step = "result";
    }
  }

  async function discardDownload() {
    await invoke("discard_dc_download").catch(() => {});
    pendingDives = [];
    step = knownDevices.length > 0 ? "list" : "setup";
  }

  // Matches the backend's fixed scan window (commands.rs: 20 × 500ms poll loop),
  // plus a small margin for the final dc-ble-found emit to land.
  const BLE_SCAN_DURATION_MS = 10_500;

  async function scanBle() {
    bleDevices = [];
    errorMsg = null;
    bleScanning = true;
    bleScanTimedOut = false;
    clearTimeout(bleScanTimer);
    await invoke("scan_ble_devices", { vendor, model });
    bleScanTimer = setTimeout(() => {
      bleScanning = false;
      if (!selectedBleDevice) bleScanTimedOut = true;
    }, BLE_SCAN_DURATION_MS);
  }

  function cancel() { invoke("cancel_dc_download").catch(() => {}); }
  function close() {
    step = knownDevices.length > 0 ? "list" : "setup";
    onClose();
  }

  function fmtBytes(n: number): string {
    if (n >= 1_048_576) return (n / 1_048_576).toFixed(1) + " MiB";
    if (n >= 1_024) return Math.round(n / 1_024) + " KiB";
    return n + " B";
  }

  function fmtDuration(sec: number): string {
    return `${Math.floor(sec / 60)}:${String(sec % 60).padStart(2, "0")}`;
  }
</script>

{#if open}
  <div class="dialog-backdrop" role="dialog" aria-modal="true">
    <div class="dialog">
      {#if step === "list"}
        <h2>Select Device</h2>
        <label>
          Device
          <select aria-label="Known device" bind:value={selectedKnownDeviceKey}>
            {#each knownDevices as d}
              <option value={deviceKey(d.vendor, d.product, d.serial)}>
                {d.vendor} {d.product} ({d.nickname || `SN ${d.serial}`})
              </option>
            {/each}
          </select>
        </label>
        <button onclick={connectToSelectedKnownDevice} disabled={!selectedKnownDeviceKey}>Continue</button>
        <button onclick={goSetupNew}>Add new device</button>
        <button onclick={onClose}>Cancel</button>

      {:else if step === "setup"}
        <h2>{isKnownDevice ? "Connect" : "Add Device"}</h2>
        {#if isKnownDevice}
          <p class="known-device-label">{vendor} {model}</p>
          {#if knownDevices.length > 0}
            <button type="button" onclick={() => (step = "list")}>Use a different device</button>
          {/if}
        {:else}
          <label>
            Vendor
            <select aria-label="Vendor" bind:value={vendor} onchange={onVendorChange}>
              <option value="">— select —</option>
              {#each vendors as v}<option value={v}>{v}</option>{/each}
            </select>
          </label>
          <label>
            Model
            <select bind:value={model} onchange={onModelChange} disabled={!vendor}>
              {#each models as m}<option value={m.product}>{m.product}</option>{/each}
            </select>
          </label>
        {/if}
        {#if model}
          <label>
            Transport
            <select bind:value={transport} onchange={onTransportChange}>
              {#each (models.find((m2) => m2.product === model)?.transports ?? []) as t}
                <option value={t}>{t}</option>
              {/each}
            </select>
          </label>
          {#if transport === "Serial"}
            <label>Port <select bind:value={serialPort}>{#each serialPorts as p}<option value={p}>{p}</option>{/each}</select></label>
          {:else if transport === "Bluetooth"}
            <label>Address <input bind:value={bluetoothAddress} placeholder="00:11:22:33:44:55" /></label>
          {:else if transport === "BLE"}
            <button onclick={() => scanBle()} disabled={bleScanning}>Scan</button>
            {#if bleScanning}
              <p class="status">Scanning{cachedBleAddress ? " for your remembered device" : ""}…</p>
            {:else if bleScanTimedOut && cachedBleAddress}
              <p class="warning">Remembered device not found — check it's powered on and in range, then try Scan again.</p>
            {/if}
            {#each bleDevices as d}
              <label><input type="radio" bind:group={selectedBleDevice} value={d.address} />{d.name}</label>
            {/each}
          {/if}
        {/if}
        {#if errorMsg}
          <p class="warning">{errorMsg}</p>
        {/if}
        <button onclick={() => startDownload()} disabled={
          !model ||
          (transport === "Serial" && !serialPort) ||
          (transport === "Bluetooth" && !bluetoothAddress) ||
          (transport === "BLE" && !selectedBleDevice)
        }>Download</button>
        <button onclick={onClose}>Cancel</button>

      {:else if step === "progress"}
        <h2>Downloading…</h2>
        <p class="status">{statusLabel}</p>
        <progress value={progressCurrent} max={progressMaximum || undefined}></progress>
        <p>{fmtBytes(progressCurrent)} / {progressMaximum ? fmtBytes(progressMaximum) : "?"}</p>
        {#if errorMsg}
          <p class="warning">{errorMsg}</p>
        {/if}
        <button onclick={cancel}>Cancel</button>

      {:else if step === "review"}
        <h2>Review Downloaded Dives</h2>
        {#if resultCancelled}
          <p class="warning">Download was cancelled — showing what was fetched before it stopped.</p>
        {/if}
        <p>{pendingDives.length} new dive{pendingDives.length !== 1 ? "s" : ""}{resultSkipped > 0 ? `, ${resultSkipped} already in logbook` : ""}.</p>
        <div class="dive-list" role="list">
          {#each pendingDives as dive, i}
            <div class="dive-item" role="listitem">
              <input type="checkbox" bind:checked={selectedDives[i]} aria-label={`Include dive on ${dive.date}`} />
              <span class="dive-date">{dive.date.replace("T", " ")}</span>
              <span class="dive-depth">{dive.maxDepthM.toFixed(1)} m</span>
              <span class="dive-dur">{fmtDuration(dive.durationSec)}</span>
            </div>
          {/each}
        </div>
        <button onclick={saveToLogbook} disabled={selectedCount === 0}>
          Save {selectedCount} dive{selectedCount !== 1 ? "s" : ""} to logbook
        </button>
        <button onclick={discardDownload}>Discard</button>

      {:else if step === "result"}
        <h2>{errorMsg ? "Error" : resultCancelled ? "Cancelled" : "Done"}</h2>
        {#if errorMsg}
          <p class="error">{errorMsg}</p>
        {:else if resultCancelled}
          <p>Download cancelled. No dives saved.</p>
        {:else}
          <p>{resultAdded} dive{resultAdded !== 1 ? "s" : ""} added, {resultSkipped} skipped.</p>
        {/if}
        <button onclick={close}>Close</button>
      {/if}
    </div>
  </div>
{/if}

<style>
  .dialog-backdrop {
    position: fixed; inset: 0; background: rgb(0 0 0 / 50%);
    display: flex; align-items: center; justify-content: center; z-index: 100;
  }
  .dialog {
    background: var(--panel); color: var(--txt);
    border: 1px solid var(--hair); border-radius: var(--r-panel, 8px);
    padding: 1.5rem; min-width: 400px; max-width: 560px;
    display: flex; flex-direction: column; gap: 1rem;
  }
  h2 { margin: 0; color: var(--txt); }
  p { margin: 0; color: var(--txt-2); }
  label { color: var(--txt-2); }
  select,
  input:not([type="checkbox"]):not([type="radio"]) {
    background: var(--panel-2); border: 1px solid var(--hair);
    border-radius: var(--r-control); color: var(--txt);
    font: inherit; padding: 0.35rem 0.5rem;
  }
  select:focus, input:focus { border-color: var(--blue); outline: none; }
  button {
    background: var(--elev); border: 1px solid var(--hair);
    border-radius: var(--r-control); color: var(--txt);
    font: inherit; padding: 0.4rem 0.9rem; cursor: pointer;
  }
  button:disabled { opacity: 0.5; cursor: not-allowed; }
  .error { color: var(--rate-fast, #e5484d); }
  .warning { color: var(--amber, #f2a33c); font-size: 0.875rem; }
  .known-device-label { font-weight: 600; }
  .dive-list {
    border: 1px solid var(--hair); border-radius: 4px;
    max-height: 260px; overflow-y: auto;
  }
  .dive-item {
    display: grid; grid-template-columns: auto 1fr auto auto;
    align-items: center;
    gap: 0.75rem; padding: 0.4rem 0.75rem; font-size: 0.875rem;
    border-bottom: 1px solid var(--hair);
  }
  .dive-item:last-child { border-bottom: none; }
  .dive-depth, .dive-dur { color: var(--txt-3); white-space: nowrap; }
</style>
