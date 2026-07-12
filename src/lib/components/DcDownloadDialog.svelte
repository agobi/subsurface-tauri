<!-- AI-generated (Claude) -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { app } from "$lib/stores/app.svelte.ts";
  import { fmtDepth } from "$lib/units.ts";
  import { loadDcConnections, saveDcConnection, type DcConnections, type Transport } from "$lib/dcConnections.ts";
  import { loadDcDownloadPrefs } from "$lib/prefs.ts";

  let { open, onClose }: { open: boolean; onClose: () => void } = $props();

  type Step = "list" | "setup" | "progress" | "review" | "result";
  let step = $state<Step>("list");

  type KnownDevice = { vendor: string; product: string; serial: string; nickname: string };
  let knownDevices = $state<KnownDevice[]>([]);
  let selectedKnownDeviceKey = $state("");
  let isKnownDevice = $state(false);
  let dcConnections = $state<DcConnections>({});

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
  let bleScanTimer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    if (selectedBleDevice) bleScanning = false;
  });

  let progressCurrent = $state(0);
  let progressMaximum = $state(0);
  let resultAdded = $state(0);
  let resultSkipped = $state(0);
  let resultCancelled = $state(false);
  let errorMsg = $state<string | null>(null);
  let statusLabel = $state("Connecting…");

  type DiveSummary = { date: string; durationSec: number; maxDepthM: number };
  type DiveGroup = { merged: DiveSummary; segments: DiveSummary[] };
  type GroupState = { expanded: boolean; mergedChecked: boolean; segmentChecked: boolean[] };
  let pendingGroups = $state<DiveGroup[]>([]);
  let groupStates = $state<GroupState[]>([]);
  let selectedCount = $derived(
    groupStates.reduce(
      (n, g) => n + (g.expanded ? g.segmentChecked.filter(Boolean).length : (g.mergedChecked ? 1 : 0)),
      0,
    ),
  );

  function toggleExpanded(i: number) {
    const g = groupStates[i];
    if (!g.expanded) {
      // Expanding: seed each segment's checked state from the merged checkbox.
      groupStates[i] = { ...g, expanded: true, segmentChecked: g.segmentChecked.map(() => g.mergedChecked) };
    } else {
      // Collapsing: merged is checked if any segment was checked.
      groupStates[i] = { ...g, expanded: false, mergedChecked: g.segmentChecked.some(Boolean) };
    }
  }

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
      groups: DiveGroup[];
      skipped: number;
      cancelled: boolean;
    }>("dc-complete", (e) => {
      resultSkipped = e.payload.skipped;
      resultCancelled = e.payload.cancelled;
      errorMsg = null;
      if (e.payload.groups.length > 0) {
        // Show whatever was fetched for review, even if the download was
        // cancelled partway through — a cancel shouldn't discard progress
        // that was already downloaded.
        pendingGroups = e.payload.groups;
        groupStates = e.payload.groups.map((g) => ({
          expanded: false,
          mergedChecked: true,
          segmentChecked: g.segments.map(() => true),
        }));
        step = "review";
      } else {
        if (!e.payload.cancelled) {
          // Nothing new to review, but the device's fingerprint cutoff still
          // needs to advance, or every future download re-scans its full
          // history. commit_dc_download always does this regardless of an
          // empty selection.
          invoke("commit_dc_download", { selections: [] }).catch(() => {});
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
    vendor = "";
    model = "";
    models = [];
    serialPort = "";
    bluetoothAddress = "";
    selectedBleDevice = null;
    bleDevices = [];
    bleScanning = false;
    permissionDenied = false;
    clearTimeout(bleScanTimer);
    step = "setup";
  }

  /// Prefills transport + address from the cache when available, falling back
  /// to updateTransportDefault(). Returns true when a full cached address was
  /// found for the resolved transport — the caller uses this to decide
  /// whether a connection can be attempted directly, skipping setup/scan.
  function applyCachedConnectionOrDefault(serial: string): boolean {
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
    const addr = cached?.addresses[transport];
    if (!addr) return false;
    if (transport === "Serial") serialPort = addr;
    else if (transport === "Bluetooth") bluetoothAddress = addr;
    else if (transport === "BLE") selectedBleDevice = addr;
    return true;
  }

  async function selectKnownDevice(d: KnownDevice) {
    vendor = d.vendor;
    models = await invoke<{ product: string; transports: string[] }[]>("list_dc_models", { vendor });
    model = d.product;
    isKnownDevice = true;
    bleDevices = [];
    errorMsg = null;
    const hasCachedAddress = applyCachedConnectionOrDefault(d.serial);
    if (hasCachedAddress) {
      // We already know how to reach this device — skip setup/scan and try connecting directly.
      await startDownload();
    } else {
      step = "setup";
      await onTransportChange();
    }
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
    pendingGroups = [];
    const transportArg = transport === "Serial"
      ? { kind: "Serial", port: serialPort }
      : transport === "Bluetooth"
      ? { kind: "Bluetooth", address: bluetoothAddress }
      : transport === "BLE"
      ? { kind: "Ble", address: selectedBleDevice ?? "" }
      : { kind: "UsbHid" };
    const { mergeGapMinutes } = await loadDcDownloadPrefs();
    try {
      await invoke("start_dc_download", { vendor, model, transport: transportArg, mergeGapMinutes });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      if (msg === "PermissionDenied") {
        // The cached-device fast path (selectKnownDevice) can land here with
        // no scan ever having run — route to "setup" so the BLE permission
        // recovery UI (warning + Open Settings) is reachable, same as scanBle().
        permissionDenied = true;
        step = "setup";
      } else {
        errorMsg = msg;
        step = "result";
      }
    }
  }

  async function saveToLogbook() {
    try {
      const selections: { group: number; merged: boolean; segments: number[] }[] = [];
      groupStates.forEach((g, i) => {
        if (g.expanded) {
          const segments = g.segmentChecked.map((c, si) => (c ? si : -1)).filter((si) => si !== -1);
          if (segments.length > 0) selections.push({ group: i, merged: false, segments });
        } else if (g.mergedChecked) {
          selections.push({ group: i, merged: true, segments: [] });
        }
      });
      const added = await invoke<number>("commit_dc_download", { selections });
      resultAdded = added;
      resultCancelled = false;
      await app.startup();
      step = "result";
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : String(e);
      step = "result";
    }
  }

  async function discardDownload() {
    await invoke("discard_dc_download").catch(() => {});
    pendingGroups = [];
    step = knownDevices.length > 0 ? "list" : "setup";
  }

  // Matches the backend's fixed scan window (commands.rs: 20 × 500ms poll loop),
  // plus a small margin for the final dc-ble-found emit to land.
  const BLE_SCAN_DURATION_MS = 10_500;

  let permissionDenied = $state(false);

  async function scanBle() {
    bleDevices = [];
    errorMsg = null;
    permissionDenied = false;
    bleScanning = true;
    clearTimeout(bleScanTimer);
    try {
      await invoke("scan_ble_devices", { vendor, model });
      bleScanTimer = setTimeout(() => { bleScanning = false; }, BLE_SCAN_DURATION_MS);
    } catch (e) {
      bleScanning = false;
      const msg = e instanceof Error ? e.message : String(e);
      if (msg === "PermissionDenied") {
        permissionDenied = true;
      } else {
        errorMsg = msg;
      }
    }
  }

  async function openAppSettings() {
    await invoke("open_app_settings").catch(() => {});
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
              <p class="status">Scanning…</p>
            {/if}
            {#if permissionDenied}
              <p class="warning">Bluetooth permission is required to scan for devices.</p>
              <button type="button" onclick={openAppSettings}>Open Settings</button>
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
        <p>{pendingGroups.length} new dive{pendingGroups.length !== 1 ? "s" : ""}{resultSkipped > 0 ? `, ${resultSkipped} already in logbook` : ""}.</p>
        <div class="dive-list" role="list">
          {#each pendingGroups as group, i}
            <div class="group">
              {#if !groupStates[i].expanded}
                <div class="dive-item" role="listitem">
                  <input type="checkbox" bind:checked={groupStates[i].mergedChecked} aria-label={`Include dive on ${group.merged.date}`} />
                  <span class="dive-date">{group.merged.date.replace("T", " ")}</span>
                  <span class="dive-depth">{fmtDepth(group.merged.maxDepthM, app.displayUnits)}</span>
                  <span class="dive-dur">{fmtDuration(group.merged.durationSec)}</span>
                  {#if group.segments.length > 1}
                    <span class="merge-badge">merged from {group.segments.length} segments</span>
                    <button type="button" class="unmerge-btn" onclick={() => toggleExpanded(i)}>Unmerge</button>
                  {/if}
                </div>
              {:else}
                {#each group.segments as seg, si}
                  <div class="dive-item" role="listitem">
                    <input type="checkbox" bind:checked={groupStates[i].segmentChecked[si]} aria-label={`Include dive on ${seg.date}`} />
                    <span class="dive-date">{seg.date.replace("T", " ")}</span>
                    <span class="dive-depth">{fmtDepth(seg.maxDepthM, app.displayUnits)}</span>
                    <span class="dive-dur">{fmtDuration(seg.durationSec)}</span>
                  </div>
                {/each}
                <div class="dive-item merge-row">
                  <button type="button" class="unmerge-btn" onclick={() => toggleExpanded(i)}>Merge</button>
                </div>
              {/if}
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
    display: flex; align-items: center; justify-content: center; z-index: 1001;
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
  progress { width: 100%; }
  .error { color: var(--rate-fast, #e5484d); }
  .warning { color: var(--amber, #f2a33c); font-size: 0.875rem; }
  .known-device-label { font-weight: 600; }
  .dive-list {
    border: 1px solid var(--hair); border-radius: 4px;
    max-height: 260px; overflow-y: auto;
  }
  .dive-item {
    display: flex; align-items: center;
    gap: 0.75rem; padding: 0.4rem 0.75rem; font-size: 0.875rem;
    border-bottom: 1px solid var(--hair);
  }
  .group:last-child .dive-item:last-child { border-bottom: none; }
  .dive-date { flex: 1; }
  .dive-depth, .dive-dur { color: var(--txt-3); white-space: nowrap; }
  .merge-badge { color: var(--txt-3); font-size: 0.75rem; white-space: nowrap; }
  .unmerge-btn { padding: 0.2rem 0.5rem; font-size: 0.75rem; }
  .merge-row { justify-content: flex-end; }
</style>
