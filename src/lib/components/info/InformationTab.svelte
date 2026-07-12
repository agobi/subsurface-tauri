<!-- AI-generated (Claude) -->
<script lang="ts">
  import type { Dive } from "$lib/types.ts";
  import { fmtMinSec } from "$lib/format.ts";
  import { fmtDepth, fmtTemp } from "$lib/units.ts";
  import { app } from "$lib/stores/app.svelte.ts";
  let { dive }: { dive: Dive } = $props();
  const rows = $derived<[string, string][]>([
    ["Max depth", dive.maxDepthM != null ? fmtDepth(dive.maxDepthM, app.displayUnits) : "-"],
    ["Mean depth", dive.meanDepthM != null ? fmtDepth(dive.meanDepthM, app.displayUnits) : "-"],
    ["Water temp", dive.waterTempC != null ? fmtTemp(dive.waterTempC, app.displayUnits) : "-"],
    ["Duration", fmtMinSec(dive.durationSec)],
    ["Deco model", dive.decoModel ?? "-"],
  ]);
</script>
<div class="fields">{#each rows as [l, v]}<div class="fl">{l}</div><div class="fv tnum">{v}</div>{/each}</div>
<style>
  .fields { display: grid; grid-template-columns: max-content 1fr; gap: 6px 12px; }
  .fl { color: var(--txt-3); font-size: 11.5px; text-align: right; }
  .fv { color: var(--txt); font-size: 13px; }
</style>
