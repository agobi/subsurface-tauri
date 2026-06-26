<!-- AI-generated (Claude) -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { open as openDialog, message as showMessage } from "@tauri-apps/plugin-dialog";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { platform } from "@tauri-apps/plugin-os";
  import { app, type VisiblePanels } from "$lib/stores/app.svelte.ts";
  import { loadAppearancePrefs, applyTheme, type AppearancePrefs } from "$lib/prefs.ts";
  import type { RecentEntry } from "$lib/types.ts";
  import Toolbar from "$lib/components/Toolbar.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import QuadrantGrid from "$lib/components/QuadrantGrid.svelte";
  import MobileLayout from "$lib/components/MobileLayout.svelte";
  import CloudLoginDialog from "$lib/components/CloudLoginDialog.svelte";
  import DcDownloadDialog from "$lib/components/DcDownloadDialog.svelte";

  let search = $state("");
  let initialized = $state(false);
  let startupError = $state<string | null>(null);
  let unlisteners: (() => void)[] = [];

  async function setWindowTitle() {
    await getCurrentWindow().setTitle(`${app.displayName} — Subsurface`);
  }

  async function handleOpen() {
    const dir = await openDialog({ directory: true });
    if (typeof dir === "string") {
      try {
        await app.open(dir);
        await setWindowTitle();
      } catch (e) {
        await showMessage(e instanceof Error ? e.message : String(e), { title: "Error", kind: "error" });
      }
    }
  }

  async function handleNew() {
    const dir = await openDialog({ directory: true });
    if (typeof dir === "string") {
      try {
        await app.newLogbook(dir);
        await setWindowTitle();
      } catch (e) {
        await showMessage(e instanceof Error ? e.message : String(e), { title: "Error", kind: "error" });
      }
    }
  }

  async function handleSync() {
    try {
      await app.syncCloud();
      await setWindowTitle();
    } catch (e) {
      await showMessage(e instanceof Error ? e.message : String(e), { title: "Sync Failed", kind: "error" });
    }
  }

  async function handleCloudSuccess(_email: string) {
    const cb = typeof app.showCloudDialog === "object" ? app.showCloudDialog.onSuccess : undefined;
    app.showCloudDialog = false;
    try { await setWindowTitle(); } catch { /* title update failure must not block onSuccess */ }
    await cb?.();
  }

  async function handleOpenRecent(entry: RecentEntry) {
    if (entry.kind === "Cloud") {
      try {
        await app.openRecentCloud(entry.email);
        await setWindowTitle();
      } catch {
        app.showCloudDialog = {
          email: entry.email,
          message: "Saved credentials could not be loaded. Please sign in again.",
        };
      }
    } else {
      try {
        await app.open(entry.path);
        await setWindowTitle();
      } catch (e) {
        await showMessage(e instanceof Error ? e.message : String(e), { title: "Error", kind: "error" });
      }
    }
  }

  onMount(async () => {
    try {
      const p = await platform();
      app.setPlatform(p === "android" || p === "ios" ? "mobile" : "desktop");

      await app.startup();
      await setWindowTitle();

      const prefs = await loadAppearancePrefs();
      app.setTheme(prefs.theme);
    } catch (e) {
      startupError = e instanceof Error ? e.message : String(e);
      return;
    }

    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    const handleColorScheme = () => applyTheme(app.theme);
    mql.addEventListener("change", handleColorScheme);

    if (!app.isMobile) {
      unlisteners = await Promise.all([
        listen("menu:file-open", handleOpen),
        listen("menu:file-new", handleNew),
        listen("menu:cloud-open", () => { app.showCloudDialog = { email: "" }; }),
        listen<VisiblePanels>("menu:set-panels", ({ payload }) => {
          app.visiblePanels = payload;
        }),
        listen<AppearancePrefs>("prefs:appearance-changed", ({ payload }) => {
          app.setTheme(payload.theme);
        }),
        listen<RecentEntry>("menu:open-recent", ({ payload }) => {
          handleOpenRecent(payload);
        }),
        listen("menu:dc-download", () => { app.showDcDialog = true; }),
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

  {#if app.showCloudDialog !== false}
    <CloudLoginDialog
      initialEmail={app.showCloudDialog.email}
      message={app.showCloudDialog.message}
      onClose={() => { app.showCloudDialog = false; }}
      onSuccess={handleCloudSuccess}
    />
  {/if}

  {#if app.showDcDialog}
    <DcDownloadDialog open={app.showDcDialog} onClose={() => (app.showDcDialog = false)} />
  {/if}
{/if}

{#if startupError}
  <div class="startup-error">{startupError}</div>
{/if}

<style>
  .app { display: flex; flex-direction: column; height: 100vh; }
  .startup-error { position: fixed; inset: 0; display: flex; align-items: center; justify-content: center; color: var(--txt-2); font-size: 13px; }
</style>
