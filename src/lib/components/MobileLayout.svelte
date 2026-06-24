<!-- AI-generated (Claude) -->
<script lang="ts">
  import { app } from "$lib/stores/app.svelte.ts";
  import { open as openDialog, message as showMessage } from "@tauri-apps/plugin-dialog";
  import DiveList from "$lib/components/DiveList.svelte";
  import DiveProfile from "$lib/components/DiveProfile.svelte";
  import InfoPanel from "$lib/components/InfoPanel.svelte";
  import MapPanel from "$lib/components/MapPanel.svelte";
  import MobileSettingsScreen from "$lib/components/MobileSettingsScreen.svelte";

  type Tab = "dives" | "profile" | "info" | "map";
  type Screen = "main" | "settings";

  let activeTab = $state<Tab>("dives");
  let screen = $state<Screen>("main");

  async function handleOpenLogbook() {
    const dir = await openDialog({ directory: true });
    if (typeof dir === "string") {
      try {
        await app.open(dir);
      } catch (e) {
        await showMessage(e instanceof Error ? e.message : String(e), { title: "Error", kind: "error" });
      }
    }
  }

  let selected = $derived(app.selectedDive);
  let selectedSite = $derived(app.logbook.sites.find((s) => s.id === selected?.siteId));

  const tabs: { key: Tab; label: string; icon: string }[] = [
    { key: "dives",   label: "Dives",   icon: "≡" },
    { key: "profile", label: "Profile", icon: "∿" },
    { key: "info",    label: "Info",    icon: "ℹ" },
    { key: "map",     label: "Map",     icon: "⊕" },
  ];
</script>

{#if screen === "settings"}
  <MobileSettingsScreen onBack={() => (screen = "main")} />
{:else}
  <div class="mobile-layout">
    <div class="mobile-content">
      {#if activeTab === "dives"}
        <div class="dives-header">
          <span class="dives-title">Dives</span>
          <div class="header-actions">
            <button class="header-btn" aria-label="Open logbook" onclick={handleOpenLogbook} title="Open logbook">⊞</button>
            <button class="header-btn" aria-label="Cloud logbook" onclick={() => (app.showCloudDialog = { email: "" })} title="Cloud logbook">☁</button>
            <button class="gear-btn" aria-label="Settings" onclick={() => (screen = "settings")}>⚙</button>
          </div>
        </div>
        <div class="mobile-panel" data-testid="mobile-panel-dives">
          {#if app.sortedDives.length === 0}
            <div class="empty-state" data-testid="mobile-empty-state">
              <p class="empty-msg">No dives yet.</p>
              <button class="empty-btn" onclick={handleOpenLogbook}>Open Logbook…</button>
              <button class="empty-btn" onclick={() => (app.showCloudDialog = { email: "" })}>Connect to Cloud…</button>
            </div>
          {:else}
            <DiveList dives={app.sortedDives} trips={app.logbook.trips} sites={app.logbook.sites} />
          {/if}
        </div>
      {:else if activeTab === "profile"}
        <div class="mobile-panel" data-testid="mobile-panel-profile">
          <DiveProfile dive={selected} loading={app.selectedDiveLoading} />
        </div>
      {:else if activeTab === "info"}
        <div class="mobile-panel" data-testid="mobile-panel-info">
          {#if selected}<InfoPanel dive={selected} />{/if}
        </div>
      {:else if activeTab === "map"}
        <div class="mobile-panel" data-testid="mobile-panel-map">
          <MapPanel siteName={selectedSite?.name} gps={selectedSite?.gps} />
        </div>
      {/if}
    </div>

    <div class="tab-bar" role="tablist">
      {#each tabs as t}
        <button
          class="tab-btn"
          class:active={activeTab === t.key}
          role="tab"
          aria-selected={activeTab === t.key}
          aria-label={t.label}
          onclick={() => (activeTab = t.key)}
        >
          <span class="tab-icon">{t.icon}</span>
          <span class="tab-label">{t.label}</span>
        </button>
      {/each}
    </div>
  </div>
{/if}

<style>
  .mobile-layout {
    display: flex;
    flex-direction: column;
    height: 100dvh;
    overflow: hidden;
  }

  .mobile-content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding-top: env(safe-area-inset-top);
  }

  .dives-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2) var(--space-4);
    background: var(--panel);
    border-bottom: 1px solid var(--hair);
    flex: 0 0 auto;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0;
  }

  .header-btn {
    background: none;
    border: none;
    font-size: 1.1rem;
    color: var(--txt-3);
    cursor: pointer;
    padding: 0;
    min-height: 44px;
    min-width: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    padding: var(--space-8, 48px) var(--space-4);
    height: 100%;
  }

  .empty-msg {
    color: var(--txt-3);
    font-size: 0.9rem;
    margin: 0;
  }

  .empty-btn {
    background: var(--blue);
    color: #fff;
    border: none;
    border-radius: var(--r-control, 6px);
    padding: var(--space-2) var(--space-4);
    font: inherit;
    font-size: 0.9rem;
    cursor: pointer;
    min-height: 44px;
    min-width: 200px;
  }

  .dives-title {
    font-weight: 600;
    font-size: 1rem;
  }

  .gear-btn {
    background: none;
    border: none;
    font-size: 1.2rem;
    color: var(--txt-3);
    cursor: pointer;
    padding: 0;
    min-height: 44px;
    min-width: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .mobile-panel {
    flex: 1;
    overflow: auto;
    height: 100%;
  }

  .tab-bar {
    flex: 0 0 56px;
    display: flex;
    align-items: stretch;
    background: var(--panel);
    border-top: 1px solid var(--hair);
    padding-bottom: env(safe-area-inset-bottom);
  }

  .tab-btn {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    border: 0;
    background: transparent;
    color: var(--txt-3);
    font: inherit;
    cursor: pointer;
    padding: 0;
    min-height: 44px;
  }

  .tab-btn.active {
    color: var(--blue);
  }

  .tab-icon {
    font-size: 18px;
    line-height: 1;
  }

  .tab-label {
    font-size: 10px;
    font-weight: 560;
    letter-spacing: 0.03em;
  }
</style>
