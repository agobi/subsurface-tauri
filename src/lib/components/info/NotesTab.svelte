<!-- AI-generated (Claude) -->
<script lang="ts">
  import type { Dive } from "$lib/types.ts";
  let { dive }: { dive: Dive } = $props();
  const rows = $derived<[string, string][]>([
    ["Date", dive.dateTime.replace("T", " ")],
    ["Dive guide", dive.diveGuide ?? "-"],
    ["Buddy", dive.buddy ?? "-"],
    ["Suit", dive.suit ?? "-"],
    ["Rating", "★".repeat(dive.rating ?? 0)],
  ]);
</script>
<div class="fields">
  {#each rows as [l, v]}<div class="fl">{l}</div><div class="fv">{v}</div>{/each}
</div>
<div class="tags">{#each dive.tags as t}<span class="tag">{t}</span>{/each}</div>
<p class="notes">{dive.notes ?? ""}</p>
<style>
  .fields { display: grid; grid-template-columns: max-content 1fr; gap: 6px 12px; }
  .fl { color: var(--txt-3); font-size: 11.5px; text-align: right; white-space: nowrap; }
  .fv { color: var(--txt); font-size: 13px; }
  .tags { display: flex; gap: 6px; flex-wrap: wrap; margin: 12px 0; }
  .tag { padding: 2px 9px; background: var(--elev); border: 1px solid var(--hair); border-radius: var(--r-pill); font-size: 11px; color: var(--txt-2); }
  .notes { color: var(--txt); font-size: 13px; line-height: 1.5; }
</style>
