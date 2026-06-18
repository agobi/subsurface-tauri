<!-- AI-generated (Claude) -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { app, type PanelKey } from "$lib/stores/app.svelte.ts";
  import Toolbar from "$lib/components/Toolbar.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import QuadrantGrid from "$lib/components/QuadrantGrid.svelte";

  let search = $state("");
  let unlisteners: (() => void)[] = [];

  function basename(path: string): string {
    return path.split(/[\\/]/).pop() ?? path;
  }

  onMount(async () => {
    await app.startup();

    unlisteners.push(
      await listen("menu:file-open", async () => {
        const dir = await openDialog({ directory: true });
        if (typeof dir === "string") {
          await app.open(dir);
          await getCurrentWindow().setTitle(`${basename(dir)} — Subsurface`);
        }
      })
    );

    unlisteners.push(
      await listen("menu:file-new", async () => {
        const dir = await openDialog({ directory: true });
        if (typeof dir === "string") {
          await app.newLogbook(dir);
          await getCurrentWindow().setTitle(`${basename(dir)} — Subsurface`);
        }
      })
    );

    unlisteners.push(
      await listen<{ panel: PanelKey; visible: boolean }>(
        "menu:toggle-panel",
        ({ payload }) => {
          app.setPanelVisible(payload.panel, payload.visible);
        }
      )
    );
  });

  onDestroy(() => {
    unlisteners.forEach((fn) => fn());
  });

  $effect(() => {
    document.documentElement.setAttribute("data-theme", app.theme);
  });
</script>

<div class="app">
  <Toolbar onSearch={(q) => (search = q)} />
  <QuadrantGrid query={search} />
  <StatusBar diveCount={app.dives.length} decoModel={app.selectedDive?.decoModel ?? "-"} synced={true} />
</div>

<style>
  .app { display: flex; flex-direction: column; height: 100vh; }
</style>
