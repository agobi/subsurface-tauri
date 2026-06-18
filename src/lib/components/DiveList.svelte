<!-- AI-generated (Claude) -->
<script lang="ts">
  import type { Dive, Trip, Site } from "$lib/types.ts";
  import { app } from "$lib/stores/app.svelte.ts";
  import { fmtMinSec } from "$lib/format.ts";

  let { dives, trips, sites, query = "" }: { dives: Dive[]; trips: Trip[]; sites: Site[]; query?: string } = $props();

  const siteName = (id?: string) => sites.find((s) => s.id === id)?.name ?? "Unknown site";

  let filtered = $derived(
    dives.filter((d) => {
      if (!query.trim()) return true;
      const q = query.toLowerCase();
      return siteName(d.siteId).toLowerCase().includes(q) || d.tags.some((t) => t.toLowerCase().includes(q)) || (d.notes ?? "").toLowerCase().includes(q);
    })
  );

  function stars(n = 0) { return "★".repeat(n) + "☆".repeat(5 - n); }
  function fmtDate(iso: string) { return iso.slice(0, 10); }

  // Trip grouping: a dive belongs to a trip if its number is in trip.diveNumbers.
  // Dives not in any trip render ungrouped after the trip groups.
  let collapsed = $state<Record<string, boolean>>({});
  function toggleTrip(label: string) { collapsed = { ...collapsed, [label]: !collapsed[label] }; }
  function tripDives(t: Trip) { return filtered.filter((d) => t.diveNumbers.includes(d.number)); }
  let groupedNumbers = $derived(new Set(trips.flatMap((t) => t.diveNumbers)));
  let ungrouped = $derived(filtered.filter((d) => !groupedNumbers.has(d.number)));
</script>

<div class="dl">
  <div class="dl-head"><span>#</span><span>Date</span><span>Rating</span><span>Depth</span><span>Duration</span><span>Location</span></div>

  {#each trips as t}
    {@const tds = tripDives(t)}
    {#if tds.length}
      <button class="trip" onclick={() => toggleTrip(t.label)}>
        <span class="tw">{collapsed[t.label] ? "+" : "-"} {t.label}</span>
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
</div>

{#snippet row(d: Dive, i: number)}
  <div class="dl-row" class:zebra={i % 2 === 1} class:sel={app.selectedDiveId === d.number}
    role="button" tabindex="0" onclick={() => (app.selectedDiveId = d.number)}
    onkeydown={(e) => e.key === "Enter" && (app.selectedDiveId = d.number)}>
    <span class="tnum">{d.number}</span>
    <span class="tnum">{fmtDate(d.dateTime)}</span>
    <span class="stars" aria-label={`rating ${d.rating ?? 0} of 5`}>{stars(d.rating)}</span>
    <span class="tnum depth">{d.maxDepthM?.toFixed(1) ?? "-"}</span>
    <span class="tnum">{fmtMinSec(d.durationSec)}</span>
    <span class="loc">{siteName(d.siteId)}</span>
  </div>
{/snippet}

<style>
  .dl { display: flex; flex-direction: column; font-size: 12px; }
  .dl-head { display: grid; grid-template-columns: 40px 88px 80px 56px 64px 1fr; gap: 8px; align-items: center; height: 28px; padding: 0 var(--space-3); position: sticky; top: 0; z-index: 1; background: var(--panel-2); border-bottom: 1px solid var(--hair); font-size: 10.5px; font-weight: 640; color: var(--txt-3); text-transform: uppercase; letter-spacing: .04em; }
  .trip { display: flex; align-items: center; gap: 8px; width: 100%; height: 30px; padding: 0 var(--space-3); background: var(--panel-2); border: 0; border-bottom: 1px solid var(--hair); border-top: 1px solid var(--hair); font: inherit; font-size: 11.5px; color: var(--txt-2); cursor: pointer; text-align: left; }
  .trip .tw { font-weight: 600; color: var(--txt); }
  .trip .cnt { margin-left: auto; color: var(--txt-3); font-size: 11px; }
  .dl-row { display: grid; grid-template-columns: 40px 88px 80px 56px 64px 1fr; gap: 8px; align-items: center; height: 40px; padding: 0 var(--space-3); border-bottom: 1px solid var(--hair); cursor: pointer; position: relative; }
  .dl-row.zebra { background: var(--panel-2); }
  .dl-row:hover { background: var(--elev); }
  .dl-row.sel::before { content: ""; position: absolute; left: 0; top: 0; bottom: 0; width: 3px; background: var(--blue); }
  .stars { color: var(--amber); font-size: 11px; letter-spacing: -1px; }
  .depth { text-align: right; }
  .loc { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
