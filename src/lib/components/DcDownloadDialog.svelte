<!-- AI-generated (Claude) -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  let { open, onClose }: { open: boolean; onClose: () => void } = $props();

  type Step = "select" | "connect" | "progress" | "result";
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
  let errorMsg = $state<string | null>(null);

  let unlisteners: (() => void)[] = [];

  onMount(async () => {
    vendors = await invoke<string[]>("list_dc_vendors");
    unlisteners.push(await listen<{ name: string; address: string }>("dc-ble-found", (e) => {
      const existing = bleDevices.find((d) => d.address === e.payload.address);
      if (!existing) bleDevices = [...bleDevices, e.payload];
    }));
    unlisteners.push(await listen<{ current: number; maximum: number }>("dc-progress", (e) => {
      progressCurrent = e.payload.current;
      progressMaximum = e.payload.maximum;
    }));
    unlisteners.push(await listen<{ added: number; skipped: number }>("dc-complete", async (e) => {
      resultAdded = e.payload.added;
      resultSkipped = e.payload.skipped;
      errorMsg = null;
      step = "result";
      await invoke("startup_logbook");
    }));
    unlisteners.push(await listen<{ message: string }>("dc-error", (e) => {
      errorMsg = e.payload.message;
      step = "result";
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
    step = "progress";
    progressCurrent = 0;
    progressMaximum = 0;
    const transportArg = transport === "Serial"
      ? { kind: "Serial", port: serialPort }
      : transport === "Bluetooth"
      ? { kind: "Bluetooth", address: bluetoothAddress }
      : transport === "BLE"
      ? { kind: "Ble", name: selectedBleDevice ?? "" }
      : { kind: "UsbHid" };
    try {
      await invoke("start_dc_download", { vendor, model, transport: transportArg });
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : String(e);
      step = "result";
    }
  }

  async function scanBle() {
    bleDevices = [];
    await invoke("scan_ble_devices", { vendor, model });
  }

  function cancel() { invoke("cancel_dc_download").catch(() => {}); }
  function close() { step = "select"; onClose(); }
</script>

{#if open}
  <div class="dialog-backdrop" role="dialog" aria-modal="true">
    <div class="dialog">
      {#if step === "select"}
        <h2>Select Device</h2>
        <label>
          Vendor
          <select aria-label="Vendor" bind:value={vendor} on:change={onVendorChange}>
            <option value="">— select —</option>
            {#each vendors as v}<option value={v}>{v}</option>{/each}
          </select>
        </label>
        <label>
          Model
          <select bind:value={model} on:change={updateTransportDefault} disabled={!vendor}>
            {#each models as m}<option value={m.product}>{m.product}</option>{/each}
          </select>
        </label>
        <button disabled={!model} on:click={goConnect}>Next</button>

      {:else if step === "connect"}
        <h2>Connect</h2>
        <label>
          Transport
          <select bind:value={transport} on:change={onTransportChange}>
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
          <button on:click={scanBle}>Scan</button>
          {#each bleDevices as d}
            <label><input type="radio" bind:group={selectedBleDevice} value={d.name} />{d.name}</label>
          {/each}
        {/if}
        <button on:click={startDownload} disabled={
          (transport === "Serial" && !serialPort) ||
          (transport === "Bluetooth" && !bluetoothAddress) ||
          (transport === "BLE" && !selectedBleDevice)
        }>Download</button>
        <button on:click={() => (step = "select")}>Back</button>

      {:else if step === "progress"}
        <h2>Downloading…</h2>
        <progress value={progressCurrent} max={progressMaximum || undefined}></progress>
        <p>{progressCurrent} / {progressMaximum || "?"}</p>
        {#if errorMsg}
          <p class="warning">{errorMsg}</p>
        {/if}
        <button on:click={cancel}>Cancel</button>

      {:else if step === "result"}
        <h2>{errorMsg ? "Error" : "Done"}</h2>
        {#if errorMsg}
          <p class="error">{errorMsg}</p>
        {:else}
          <p>{resultAdded} dive{resultAdded !== 1 ? "s" : ""} added, {resultSkipped} skipped.</p>
        {/if}
        <button on:click={close}>Close</button>
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
    min-width: 360px; display: flex; flex-direction: column; gap: 1rem;
  }
  .error { color: red; }
  .warning { color: #ff8800; font-size: 0.875rem; }
</style>
