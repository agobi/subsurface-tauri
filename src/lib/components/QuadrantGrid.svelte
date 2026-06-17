<!-- AI-generated (Claude) -->
<script lang="ts">
  import { onDestroy } from "svelte";
  import { app } from "$lib/stores/app.svelte.ts";
  import DiveProfile from "$lib/components/DiveProfile.svelte";
  import InfoPanel from "$lib/components/InfoPanel.svelte";
  import DiveList from "$lib/components/DiveList.svelte";
  import MapPanel from "$lib/components/MapPanel.svelte";

  let { query = "" }: { query?: string } = $props();

  let colFrac = $state(0.5);
  let rowFrac = $state(0.5);

  let selected = $derived(app.selectedDive);
  let selectedSite = $derived(app.logbook.sites.find((s) => s.id === selected?.siteId));

  // Track the active drag's listeners so they never outlive the drag or the
  // component (a mouseup outside the window, or unmount mid-drag, would otherwise
  // strand live window listeners writing to dead state).
  let dragCleanup: (() => void) | null = null;

  function startDrag(axis: "col" | "row") {
    return (e: MouseEvent) => {
      e.preventDefault();
      dragCleanup?.(); // end any prior drag that lost its mouseup
      const move = (ev: MouseEvent) => {
        if (axis === "col") colFrac = Math.min(0.8, Math.max(0.2, ev.clientX / window.innerWidth));
        else rowFrac = Math.min(0.8, Math.max(0.2, ev.clientY / window.innerHeight));
      };
      const up = () => {
        window.removeEventListener("mousemove", move);
        window.removeEventListener("mouseup", up);
        dragCleanup = null;
      };
      window.addEventListener("mousemove", move);
      window.addEventListener("mouseup", up);
      dragCleanup = up;
    };
  }

  const dragCol = startDrag("col");
  const dragRow = startDrag("row");

  onDestroy(() => dragCleanup?.());
</script>

<div class="quad-wrap">
  <div
    class="quad-grid"
    style="grid-template-columns: {colFrac}fr {1 - colFrac}fr; grid-template-rows: {rowFrac}fr {1 - rowFrac}fr;"
  >
    {#if app.visiblePanels.info}
      <section class="quad" data-testid="quad-info">
        <header class="panel-head"><span class="ttl">Info</span></header>
        <div class="body">
          {#if selected}<InfoPanel dive={selected} />{/if}
        </div>
      </section>
    {/if}
    {#if app.visiblePanels.profile}
      <section class="quad" data-testid="quad-profile">
        <header class="panel-head"><span class="ttl">Profile</span></header>
        <div class="body">
          {#if selected}<DiveProfile dive={selected} />{/if}
        </div>
      </section>
    {/if}
    {#if app.visiblePanels.list}
      <section class="quad" data-testid="quad-list">
        <header class="panel-head"><span class="ttl">Dive List</span></header>
        <div class="body">
          <DiveList dives={app.dives} trips={app.logbook.trips} sites={app.logbook.sites} {query} />
        </div>
      </section>
    {/if}
    {#if app.visiblePanels.map}
      <section class="quad" data-testid="quad-map">
        <header class="panel-head"><span class="ttl">Map</span></header>
        <div class="body">
          <MapPanel siteName={selectedSite?.name} gps={selectedSite?.gps} />
        </div>
      </section>
    {/if}
  </div>

  <!-- Vertical splitter (divides columns) -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex --><!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="splitter splitter-v"
    data-testid="splitter-v"
    style="left: calc({colFrac * 100}% - 3px);"
    onmousedown={dragCol}
    role="separator"
    aria-orientation="vertical"
    aria-label="Resize columns"
    tabindex="0"
  ></div>

  <!-- Horizontal splitter (divides rows) -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex --><!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="splitter splitter-h"
    data-testid="splitter-h"
    style="top: calc({rowFrac * 100}% - 3px);"
    onmousedown={dragRow}
    role="separator"
    aria-orientation="horizontal"
    aria-label="Resize rows"
    tabindex="0"
  ></div>
</div>

<style>
  .quad-wrap {
    position: relative;
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  .quad-grid {
    flex: 1;
    display: grid;
    gap: 1px;
    background: var(--hair);
    min-height: 0;
  }

  .quad {
    background: var(--bg);
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-head {
    height: 34px;
    flex: 0 0 34px;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0 var(--space-3);
    background: var(--panel);
    border-bottom: 1px solid var(--hair);
  }

  .ttl {
    font-size: 11px;
    font-weight: 680;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: var(--txt-3);
  }

  .body {
    flex: 1;
    overflow: auto;
    padding: var(--space-4);
    color: var(--txt-2);
  }

  .splitter {
    position: absolute;
    z-index: 10;
    background: transparent;
  }

  .splitter:hover {
    background: var(--hair-strong, rgba(128, 128, 128, 0.4));
  }

  .splitter-v {
    top: 0;
    bottom: 0;
    width: 6px;
    cursor: col-resize;
  }

  .splitter-h {
    left: 0;
    right: 0;
    height: 6px;
    cursor: row-resize;
  }
</style>
