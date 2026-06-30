<!-- AI-generated (Claude) -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { app } from "$lib/stores/app.svelte.ts";

  let { open, onClose }: { open: boolean; onClose: () => void } = $props();

  type Step = "select" | "connect" | "progress" | "review" | "result";
  let step = $state<Step>("select");

  let vendors = $state<string[]>([]);
  let vendor = $state("");
  let models = $state<{ product: string; transports: string[] }[]>([]);
  let model = $state("");
  let transport = $state<"Serial" | "BLE" | "Bluetooth" | "USBHID">("BLE");
  let serialPort = $state("");
  let serialPorts = $state<string[]>([]);
  let bluetoothAddress = $state("");
  let bleDevices = $state<{ name: string; address: string }[]>([]);
  let selectedBleDevice = $state<string | null>(null);

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

  onMount(async () => {
    vendors = await invoke<string[]>("list_dc_vendors");
    unlisteners.push(await listen<{ name: string; address: string }>("dc-ble-found", (e) => {
      const existing = bleDevices.find((d) => d.address === e.payload.address);
      if (!existing) bleDevices = [...bleDevices, e.payload];
    }));
    unlisteners.push(await listen<{ model: number; firmware: number; serial: number }>("dc-devinfo", () => {
      statusLabel = `Connected: ${vendor} ${model}`;
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
      if (!e.payload.cancelled && e.payload.dives.length > 0) {
        pendingDives = e.payload.dives;
        selectedDives = e.payload.dives.map(() => true);
        step = "review";
      } else {
        resultAdded = 0;
        step = "result";
      }
    }));
    unlisteners.push(await listen<{ message: string }>("dc-error", (e) => {
      errorMsg = e.payload.message;
    }));
  });

  onDestroy(() => unlisteners.forEach((u) => u()));

  async function onVendorChange() {
    models = await invoke<{ product: string; transports: string[] }[]>("list_dc_models", { vendor });
    model = models[0]?.product ?? "";
    updateTransportDefault();
  }

  function updateTransportDefault() {
    const m = models.find((m) => m.product === model);
    if (m?.transports.includes("BLE")) transport = "BLE";
    else if (m?.transports.includes("Serial")) transport = "Serial";
    else if (m?.transports.includes("Bluetooth")) transport = "Bluetooth";
    else transport = "USBHID";
  }

  async function onTransportChange() {
    if (transport === "Serial") serialPorts = await invoke<string[]>("list_serial_ports");
  }

  function goConnect() { step = "connect"; onTransportChange(); }

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
    step = "select";
  }

  async function scanBle() {
    bleDevices = [];
    await invoke("scan_ble_devices", { vendor, model });
  }

  function cancel() { invoke("cancel_dc_download").catch(() => {}); }
  function close() { step = "select"; onClose(); }

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
      {#if step === "select"}
        <h2>Select Device</h2>
        <label>
          Vendor
          <select aria-label="Vendor" bind:value={vendor} onchange={onVendorChange}>
            <option value="">— select —</option>
            {#each vendors as v}<option value={v}>{v}</option>{/each}
          </select>
        </label>
        <label>
          Model
          <select bind:value={model} onchange={updateTransportDefault} disabled={!vendor}>
            {#each models as m}<option value={m.product}>{m.product}</option>{/each}
          </select>
        </label>
        <button disabled={!model} onclick={goConnect}>Next</button>
        <button onclick={onClose}>Cancel</button>

      {:else if step === "connect"}
        <h2>Connect</h2>
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
          <button onclick={() => scanBle()}>Scan</button>
          {#each bleDevices as d}
            <label><input type="radio" bind:group={selectedBleDevice} value={d.address} />{d.name}</label>
          {/each}
        {/if}
        <button onclick={() => startDownload()} disabled={
          (transport === "Serial" && !serialPort) ||
          (transport === "Bluetooth" && !bluetoothAddress) ||
          (transport === "BLE" && !selectedBleDevice)
        }>Download</button>
        <button onclick={() => (step = "select")}>Back</button>

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
    background: var(--bg, #fff); padding: 1.5rem; border-radius: 8px;
    min-width: 400px; max-width: 560px; display: flex; flex-direction: column; gap: 1rem;
  }
  .error { color: red; }
  .warning { color: #ff8800; font-size: 0.875rem; }
  .dive-list {
    border: 1px solid var(--border, #ddd); border-radius: 4px;
    max-height: 260px; overflow-y: auto;
  }
  .dive-item {
    display: grid; grid-template-columns: auto 1fr auto auto;
    align-items: center;
    gap: 0.75rem; padding: 0.4rem 0.75rem; font-size: 0.875rem;
    border-bottom: 1px solid var(--border, #eee);
  }
  .dive-item:last-child { border-bottom: none; }
  .dive-depth, .dive-dur { color: var(--fg-muted, #666); white-space: nowrap; }
</style>
