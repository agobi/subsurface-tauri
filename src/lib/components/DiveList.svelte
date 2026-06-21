<!-- AI-generated (Claude) -->
<script lang="ts">
  import type { Dive, Trip, Site } from "$lib/types.ts";
  import type { RenderCtx } from "$lib/diveListColumns.ts";
  import { ALL_COLS } from "$lib/diveListColumns.ts";
  import { app } from "$lib/stores/app.svelte.ts";
  import ColumnPicker from "$lib/components/ColumnPicker.svelte";

  let { dives, trips, sites, query = "" }: {
    dives: Dive[];
    trips: Trip[];
    sites: Site[];
    query?: string;
  } = $props();

  let prefs = $derived(app.diveListPrefs);
  let visibleCols = $derived(app.visibleCols);
  let gridCols = $derived(
    visibleCols.map(c => c.width.endsWith("fr") ? `minmax(36px, ${c.width})` : c.width).join(" ") + " 28px"
  );
  let ctx: RenderCtx = $derived({ sites });
  let pickerOpen = $state(false);

  let filtered = $derived(
    dives.filter((d) => {
      if (!query.trim()) return true;
      const q = query.toLowerCase();
      const loc = sites.find(s => s.id === d.siteId)?.name ?? "";
      return loc.toLowerCase().includes(q)
        || d.tags.some(t => t.toLowerCase().includes(q))
        || (d.notes ?? "").toLowerCase().includes(q);
    })
  );

  // Trip grouping — only active when sortKey === 'nr'
  function tripDives(t: Trip) { return filtered.filter(d => t.diveNumbers.includes(d.number)); }
  let groupedNumbers = $derived(new Set(trips.flatMap(t => t.diveNumbers)));
  let ungrouped = $derived(filtered.filter(d => !groupedNumbers.has(d.number)));
  let collapsed = $state<Record<string, boolean>>({});
  function toggleTrip(label: string) { collapsed = { ...collapsed, [label]: !collapsed[label] }; }

  const numericCols = new Set(["nr", "date", "depth", "duration", "temp", "sac", "weight"]);
</script>

<div class="dl">
  <div class="dl-head" style="grid-template-columns: {gridCols}">
    {#each visibleCols as col}
      <button
        class="sort-hd"
        class:active={prefs.sortKey === col.id}
        onclick={() => app.setSortCol(col.id)}>
        {col.label}{#if prefs.sortKey === col.id}{prefs.sortDir === "asc" ? " ↑" : " ↓"}{/if}
      </button>
    {/each}
    <div class="col-menu">
      <button
        class="dots"
        aria-label="Column options"
        onclick={(e) => { e.stopPropagation(); pickerOpen = !pickerOpen; }}>⋮</button>
      {#if pickerOpen}
        <ColumnPicker bind:open={pickerOpen} />
      {/if}
    </div>
  </div>

  {#if prefs.sortKey === "nr"}
    {#each trips as t}
      {@const tds = tripDives(t)}
      {#if tds.length}
        <button class="trip" onclick={() => toggleTrip(t.label)}>
          <span class="tw">{collapsed[t.label] ? "+" : "−"} {t.label}</span>
          <span class="cnt">{tds.length} dives</span>
        </button>
        {#if !collapsed[t.label]}
          {#each tds as d, i (d.number)}
            {@render row(d, i)}
          {/each}
        {/if}
      {/if}
    {/each}
    {#each ungrouped as d, i (d.number)}
      {@render row(d, i)}
    {/each}
  {:else}
    {#each filtered as d, i (d.number)}
      {@render row(d, i)}
    {/each}
  {/if}
</div>

{#snippet row(d: Dive, i: number)}
  <div
    class="dl-row"
    data-testid="dive-row"
    style="grid-template-columns: {gridCols}"
    class:zebra={i % 2 === 1}
    class:sel={app.selectedDiveId === d.number}
    role="button"
    tabindex="0"
    onclick={() => (app.selectedDiveId = d.number)}
    onkeydown={(e) => e.key === "Enter" && (app.selectedDiveId = d.number)}>
    {#each visibleCols as col}
      <span
        class:tnum={numericCols.has(col.id)}
        class:stars={col.id === "rating"}
        aria-label={col.id === "rating" ? `rating ${d.rating ?? 0} of 5` : undefined}>
        {col.render(d, ctx)}
      </span>
    {/each}
    <span></span>
  </div>
{/snippet}

<style>
  .dl { display: flex; flex-direction: column; font-size: 12px; min-width: min-content; }

  .dl-head {
    display: grid;
    gap: 8px;
    align-items: center;
    height: 28px;
    padding: 0 var(--space-3);
    position: sticky;
    top: 0;
    z-index: 1;
    background: var(--panel-2);
    border-bottom: 1px solid var(--hair);
    font-size: 10.5px;
    font-weight: 640;
    color: var(--txt-3);
    text-transform: uppercase;
    letter-spacing: .04em;
  }

  .sort-hd {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    font-size: 10.5px;
    font-weight: 640;
    color: var(--txt-3);
    text-transform: uppercase;
    letter-spacing: .04em;
    cursor: pointer;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sort-hd:hover, .sort-hd.active { color: var(--txt); }

  .col-menu { position: relative; display: flex; align-items: center; justify-content: center; }

  .dots {
    background: none;
    border: none;
    padding: 0 2px;
    font-size: 14px;
    cursor: pointer;
    color: var(--txt-3);
    line-height: 1;
  }
  .dots:hover { color: var(--txt); }

  .trip {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    height: 30px;
    padding: 0 var(--space-3);
    background: var(--panel-2);
    border: 0;
    border-bottom: 1px solid var(--hair);
    border-top: 1px solid var(--hair);
    font: inherit;
    font-size: 11.5px;
    color: var(--txt-2);
    cursor: pointer;
    text-align: left;
  }
  .trip .tw { font-weight: 600; color: var(--txt); }
  .trip .cnt { margin-left: auto; color: var(--txt-3); font-size: 11px; }

  .dl-row {
    display: grid;
    gap: 8px;
    align-items: center;
    height: 40px;
    padding: 0 var(--space-3);
    border-bottom: 1px solid var(--hair);
    cursor: pointer;
    position: relative;
  }
  .dl-row.zebra { background: var(--panel-2); }
  .dl-row:hover { background: var(--elev); }
  .dl-row.sel::before {
    content: "";
    position: absolute;
    left: 0; top: 0; bottom: 0;
    width: 3px;
    background: var(--blue);
  }

  .dl-row span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .stars { color: var(--amber); font-size: 11px; letter-spacing: -1px; }
  .tnum { font-variant-numeric: tabular-nums; }
</style>
