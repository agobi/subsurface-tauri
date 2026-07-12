<!-- AI-generated (Claude) -->
<!-- src/lib/components/MobileLayout.svelte -->
<script lang="ts">
  import { app } from "$lib/stores/app.svelte.ts";
  import { open as openDialog, message as showMessage } from "@tauri-apps/plugin-dialog";
  import DiveList from "$lib/components/DiveList.svelte";
  import DiveProfile from "$lib/components/DiveProfile.svelte";
  import InfoPanel from "$lib/components/InfoPanel.svelte";
  import MapPanel from "$lib/components/MapPanel.svelte";
  import MobileSettingsScreen from "$lib/components/MobileSettingsScreen.svelte";
  import { computeActiveIndex } from "$lib/swipePanel.ts";
  import { onDestroy, onMount } from "svelte";

  type PanelKey = "info" | "profile" | "map";
  type Screen = "main" | "settings";

  const panels: { key: PanelKey; label: string }[] = [
    { key: "info", label: "Info" },
    { key: "profile", label: "Profile" },
    { key: "map", label: "Map" },
  ];

  let screen = $state<Screen>("main");
  let activePanelIndex = $state(1); // Profile: most useful panel right after picking a dive
  let swipeEl: HTMLDivElement;
  let scrollRafId = 0;
  let wrapEl: HTMLElement;
  let topFrac = $state(0.45);
  let dragCleanup: (() => void) | null = null;

  function startRowDrag(e: PointerEvent) {
    e.preventDefault();
    dragCleanup?.();
    const target = e.currentTarget as HTMLElement;
    target.setPointerCapture?.(e.pointerId);
    const move = (ev: PointerEvent) => {
      const rect = wrapEl.getBoundingClientRect();
      topFrac = Math.min(0.8, Math.max(0.2, (ev.clientY - rect.top) / rect.height));
    };
    const up = (ev?: PointerEvent) => {
      target.releasePointerCapture?.(ev?.pointerId ?? e.pointerId);
      target.removeEventListener("pointermove", move);
      target.removeEventListener("pointerup", up);
      dragCleanup = null;
    };
    target.addEventListener("pointermove", move);
    target.addEventListener("pointerup", up);
    dragCleanup = up;
  }

  onMount(() => {
    swipeEl.scrollLeft = swipeEl.clientWidth * activePanelIndex;
  });

  onDestroy(() => dragCleanup?.());

  let selected = $derived(app.selectedDive);
  let selectedSite = $derived(app.logbook.sites.find((s) => s.id === selected?.siteId));

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

  function handleSwipeScroll() {
    if (scrollRafId) return;
    scrollRafId = requestAnimationFrame(() => {
      scrollRafId = 0;
      activePanelIndex = computeActiveIndex(swipeEl.scrollLeft, swipeEl.clientWidth, panels.length);
    });
  }

  function jumpToPanel(index: number) {
    activePanelIndex = index;
    swipeEl.scrollTo({ left: index * swipeEl.clientWidth, behavior: "smooth" });
  }
</script>

{#if screen === "settings"}
  <MobileSettingsScreen onBack={() => (screen = "main")} />
{:else}
  <div class="mobile-layout">
    <div class="mobile-header">
      <span class="dives-title">Dives</span>
      <div class="header-actions">
        <button class="header-btn" aria-label="Open logbook" onclick={handleOpenLogbook} title="Open logbook">⊞</button>
        <button class="header-btn" aria-label="Cloud logbook" onclick={() => (app.showCloudDialog = { email: "" })} title="Cloud logbook">☁</button>
        <button class="header-btn" aria-label="Download from dive computer" onclick={() => (app.showDcDialog = true)} title="Download from dive computer">⇩</button>
        <button class="gear-btn" aria-label="Settings" onclick={() => (screen = "settings")}>⚙</button>
      </div>
    </div>

    <div class="mobile-body" bind:this={wrapEl}>
      <div class="top-row" style="flex: 0 0 {topFrac * 100}%">
        <div
          class="swipe-container"
          bind:this={swipeEl}
          onscroll={handleSwipeScroll}
          data-testid="mobile-swipe"
          data-active-panel={panels[activePanelIndex].key}
        >
          <div class="swipe-panel" data-testid="mobile-panel-info">{#if selected}<InfoPanel dive={selected} />{/if}</div>
          <div class="swipe-panel" data-testid="mobile-panel-profile"><DiveProfile dive={selected} loading={app.selectedDiveLoading} /></div>
          <div class="swipe-panel" data-testid="mobile-panel-map"><MapPanel siteName={selectedSite?.name} gps={selectedSite?.gps} /></div>
        </div>
        <div class="dots-row">
          <div class="dots" role="group" aria-label="Panel selector">
            {#each panels as p, i}
              <button
                class="dot"
                class:active={i === activePanelIndex}
                data-testid={`mobile-dot-${p.key}`}
                aria-label={`Show ${p.label} panel`}
                onclick={() => jumpToPanel(i)}
              >{i === activePanelIndex ? "●" : "○"}</button>
            {/each}
          </div>
          <span class="active-panel-label" data-testid="mobile-active-panel-label">{panels[activePanelIndex].label}</span>
        </div>
      </div>

      <!-- svelte-ignore a11y_no_noninteractive_tabindex --><!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div
        class="splitter-h"
        data-testid="mobile-splitter"
        role="separator"
        aria-orientation="horizontal"
        aria-label="Resize panels"
        tabindex="0"
        onpointerdown={startRowDrag}
      ></div>

      <div class="bottom-row" data-testid="mobile-panel-dives">
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

  .mobile-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2) var(--space-4);
    padding-top: calc(env(safe-area-inset-top) + var(--space-2));
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

  .mobile-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  .top-row {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .swipe-container {
    flex: 1;
    display: flex;
    overflow-x: auto;
    overflow-y: hidden;
    scroll-snap-type: x mandatory;
    -webkit-overflow-scrolling: touch;
    min-height: 0;
  }

  .swipe-panel {
    flex: 0 0 100%;
    scroll-snap-align: start;
    overflow: auto;
    min-width: 0;
  }

  .dots-row {
    position: relative;
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px 0;
    background: var(--panel);
    border-bottom: 1px solid var(--hair);
  }

  .dots {
    display: flex;
    gap: var(--space-2);
  }

  .dot {
    background: none;
    border: none;
    font-size: 12px;
    color: var(--txt-3);
    cursor: pointer;
    padding: 4px 6px;
    min-height: 32px;
    min-width: 32px;
  }

  .dot.active {
    color: var(--blue);
  }

  .active-panel-label {
    /* Positioned out of flow so its variable width (Info/Profile/Map) never
       shifts the centered .dots block — see issue #87. */
    position: absolute;
    right: var(--space-4);
    font-size: 11px;
    font-weight: 600;
    color: var(--txt-2);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .bottom-row {
    flex: 1;
    overflow: auto;
    min-height: 0;
    padding-bottom: env(safe-area-inset-bottom);
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

  .splitter-h {
    flex: 0 0 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: row-resize;
    touch-action: none;
    background: var(--panel);
    border-bottom: 1px solid var(--hair);
  }

  .splitter-h::after {
    content: "";
    width: 36px;
    height: 4px;
    border-radius: 2px;
    background: var(--hair-strong, rgba(128, 128, 128, 0.4));
  }
</style>
