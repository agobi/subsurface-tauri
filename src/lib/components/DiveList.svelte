<!-- AI-generated (Claude) -->
<script lang="ts">
  import type { DiveSummary, Trip, Site } from "$lib/types.ts";
  import type { RenderCtx } from "$lib/diveListColumns.ts";
  import { ALL_COLS } from "$lib/diveListColumns.ts";
  import { app } from "$lib/stores/app.svelte.ts";
  import ColumnPicker from "$lib/components/ColumnPicker.svelte";

  let { dives, trips, sites, query = "" }: {
    dives: DiveSummary[];
    trips: Trip[];
    sites: Site[];
    query?: string;
  } = $props();

  let prefs = $derived(app.diveListPrefs);
  let visibleCols = $derived(app.visibleCols);
  let gridCols = $derived(visibleCols.map(c => c.width).join(" ") + " 28px");
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

  type ListEntry =
    | { kind: "trip"; trip: Trip; tds: DiveSummary[] }
    | { kind: "dive"; dive: DiveSummary };

  // Interleaves trips (as a block positioned at their first dive) with
  // ungrouped dives, by position in `filtered` — so an ungrouped dive with
  // a higher number than every trip's dives still sorts ahead of those
  // trips when sorted "# desc" (previously all trips rendered before any
  // ungrouped dive, regardless of sort order or dive number).
  let combined = $derived.by(() => {
    const pos = new Map(filtered.map((d, i) => [d.number, i]));
    const entries: { pos: number; entry: ListEntry }[] = trips.map((t) => {
      const tds = tripDives(t);
      const p = t.diveNumbers.reduce((m, n) => Math.min(m, pos.get(n) ?? Infinity), Infinity);
      return { pos: p, entry: { kind: "trip", trip: t, tds } };
    });
    for (const d of ungrouped) {
      entries.push({ pos: pos.get(d.number) ?? Infinity, entry: { kind: "dive", dive: d } });
    }
    entries.sort((a, b) => a.pos - b.pos);
    return entries.map((e) => e.entry);
  });

  let collapsed = $state<Record<number, boolean>>({});
  function toggleTrip(key: number) { collapsed = { ...collapsed, [key]: !collapsed[key] }; }

  const numericCols = new Set(["nr", "date", "depth", "duration", "temp", "sac", "weight"]);
</script>

<div class="dl">
  <div class="dl-head" style="grid-template-columns: {gridCols}">
    {#each visibleCols as col}
      <button
        class="sort-hd"
        class:active={prefs.sortKey === col.id}
        onclick={() => void app.setSortCol(col.id)}>
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
    {#each combined as entry, gi (entry.kind === "trip" ? `trip-${entry.trip.diveNumbers[0] ?? entry.trip.label}` : `dive-${entry.dive.number}`)}
      {#if entry.kind === "trip"}
        {@const t = entry.trip}
        {@const tds = entry.tds}
        {@const tripKey = t.diveNumbers[0] ?? `trip-${t.label}`}
        {#if tds.length}
          <button class="trip" onclick={() => toggleTrip(tripKey)}>
            <span class="tw">{collapsed[tripKey] ? "+" : "−"} {t.label}</span>
            <span class="cnt">{tds.length} {tds.length === 1 ? "dive" : "dives"}</span>
          </button>
          {#if !collapsed[tripKey]}
            {#each tds as d, i (d.number)}
              {@render row(d, i, true)}
            {/each}
          {/if}
        {:else}
          <div class="trip trip--empty">
            <span class="tw">− {t.label}</span>
            <span class="cnt">(no dives parsed)</span>
          </div>
        {/if}
      {:else}
        {@render row(entry.dive, gi)}
      {/if}
    {/each}
  {:else}
    {#each filtered as d, i (d.number)}
      {@render row(d, i)}
    {/each}
  {/if}
</div>

{#snippet row(d: DiveSummary, i: number, inTrip = false)}
  <div
    class="dl-row"
    data-testid="dive-row"
    style="grid-template-columns: {gridCols}"
    class:zebra={i % 2 === 1}
    class:in-trip={inTrip}
    class:sel={app.selectedDiveId === d.number}
    role="button"
    tabindex="0"
    onclick={() => void app.selectDive(d.number)}
    onkeydown={(e) => e.key === "Enter" && void app.selectDive(d.number)}>
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
  .dl { display: flex; flex-direction: column; font-size: 12px; min-width: max-content; }

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
  .dl-row.in-trip span:first-child { padding-left: 16px; }
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
