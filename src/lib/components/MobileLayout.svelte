<!-- AI-generated (Claude) -->
<script lang="ts">
  import { app } from "$lib/stores/app.svelte.ts";
  import DiveList from "$lib/components/DiveList.svelte";
  import DiveProfile from "$lib/components/DiveProfile.svelte";
  import InfoPanel from "$lib/components/InfoPanel.svelte";
  import MapPanel from "$lib/components/MapPanel.svelte";

  type Tab = "dives" | "profile" | "info" | "map";
  let activeTab = $state<Tab>("dives");

  let selected = $derived(app.selectedDive);
  let selectedSite = $derived(app.logbook.sites.find((s) => s.id === selected?.siteId));

  const tabs: { key: Tab; label: string; icon: string }[] = [
    { key: "dives",   label: "Dives",   icon: "≡" },
    { key: "profile", label: "Profile", icon: "∿" },
    { key: "info",    label: "Info",    icon: "ℹ" },
    { key: "map",     label: "Map",     icon: "⊕" },
  ];
</script>

<div class="mobile-layout">
  <div class="mobile-content">
    {#if activeTab === "dives"}
      <div class="mobile-panel" data-testid="mobile-panel-dives">
        <DiveList dives={app.dives} trips={app.logbook.trips} sites={app.logbook.sites} />
      </div>
    {:else if activeTab === "profile"}
      <div class="mobile-panel" data-testid="mobile-panel-profile">
        {#if selected}<DiveProfile dive={selected} />{/if}
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

<style>
  .mobile-layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .mobile-content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-height: 0;
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
