<!-- AI-generated (Claude) -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { platform } from "@tauri-apps/plugin-os";
  import { app, type VisiblePanels } from "$lib/stores/app.svelte.ts";
  import { loadAppearancePrefs, applyTheme, type AppearancePrefs } from "$lib/prefs.ts";
  import Toolbar from "$lib/components/Toolbar.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import QuadrantGrid from "$lib/components/QuadrantGrid.svelte";
  import MobileLayout from "$lib/components/MobileLayout.svelte";
  import CloudLoginDialog from "$lib/components/CloudLoginDialog.svelte";

  let search = $state("");
  let showCloudDialog = $state(false);
  let initialized = $state(false);
  let unlisteners: (() => void)[] = [];

  function basename(path: string): string {
    return path.split(/[\\/]/).pop() || path;
  }

  async function handleOpen() {
    const dir = await openDialog({ directory: true });
    if (typeof dir === "string") {
      try {
        await app.open(dir);
        await getCurrentWindow().setTitle(`${basename(dir)} — Subsurface`);
      } catch (e) {
        console.error("Failed to open logbook:", e);
      }
    }
  }

  async function handleNew() {
    const dir = await openDialog({ directory: true });
    if (typeof dir === "string") {
      try {
        await app.newLogbook(dir);
        await getCurrentWindow().setTitle(`${basename(dir)} — Subsurface`);
      } catch (e) {
        console.error("Failed to create logbook:", e);
      }
    }
  }

  async function handleSync() {
    await app.syncCloud();
  }

  async function handleCloudSuccess(email: string) {
    showCloudDialog = false;
    await getCurrentWindow().setTitle(`${email} — Subsurface`);
  }

  onMount(async () => {
    try {
      const p = await platform();
      app.setPlatform(p === "android" || p === "ios" ? "mobile" : "desktop");
    } catch (e) {
      console.error("Failed to detect platform:", e);
    }

    try {
      await app.startup();
    } catch (e) {
      console.error("Startup failed:", e);
    }

    try {
      const prefs = await loadAppearancePrefs();
      app.setTheme(prefs.theme);
    } catch (e) {
      console.error("Failed to load appearance prefs:", e);
    }

    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    const handleColorScheme = () => applyTheme(app.theme);
    mql.addEventListener("change", handleColorScheme);

    if (!app.isMobile) {
      unlisteners = await Promise.all([
        listen("menu:file-open", handleOpen),
        listen("menu:file-new", handleNew),
        listen("menu:cloud-open", () => { showCloudDialog = true; }),
        listen<VisiblePanels>("menu:set-panels", ({ payload }) => {
          app.visiblePanels = payload;
        }),
        listen<AppearancePrefs>("prefs:appearance-changed", ({ payload }) => {
          app.setTheme(payload.theme);
        }),
      ]);
    }

    unlisteners.push(() => mql.removeEventListener("change", handleColorScheme));
    initialized = true;
  });

  onDestroy(() => {
    unlisteners.forEach((fn) => fn());
  });

  $effect(() => {
    applyTheme(app.theme);
  });
</script>

{#if initialized}
  {#if app.isMobile}
    <MobileLayout />
  {:else}
    <div class="app">
      <Toolbar
        onSearch={(q) => (search = q)}
        isCloud={app.isCloudLogbook}
        onSync={handleSync}
      />
      <QuadrantGrid query={search} />
      <StatusBar diveCount={app.dives.length} decoModel={app.selectedDive?.decoModel ?? "-"} synced={true} />
    </div>
  {/if}

  {#if showCloudDialog}
    <CloudLoginDialog
      onClose={() => { showCloudDialog = false; }}
      onSuccess={handleCloudSuccess}
    />
  {/if}
{/if}

<style>
  .app { display: flex; flex-direction: column; height: 100vh; }
</style>
