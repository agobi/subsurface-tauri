<!-- AI-generated (Claude) -->
<script lang="ts">
  import type { Dive } from "$lib/types.ts";
  import { fmtMinSec } from "$lib/format.ts";
  let { dive }: { dive: Dive } = $props();
  const rows = $derived<[string, string][]>([
    ["Max depth", `${dive.maxDepthM?.toFixed(1) ?? "-"} m`],
    ["Mean depth", `${dive.meanDepthM?.toFixed(1) ?? "-"} m`],
    ["Water temp", `${dive.waterTempC ?? "-"} °C`],
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
