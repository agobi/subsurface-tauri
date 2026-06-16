<!-- AI-generated (Claude) -->
<script lang="ts">
  import type { Dive } from "$lib/types.ts";
  let { dive }: { dive: Dive } = $props();
</script>
<table class="eq">
  <thead><tr><th>Type</th><th>Size</th><th>Work</th><th>Start</th><th>End</th><th>O2%</th></tr></thead>
  <tbody>
    {#each dive.cylinders as c}
      <tr><td>{c.description}</td><td class="tnum">{c.volumeL ?? "-"} L</td><td class="tnum">{c.workPressureBar ?? "-"}</td><td class="tnum">{c.startBar ?? "-"}</td><td class="tnum">{c.endBar ?? "-"}</td><td class="tnum">{c.o2Percent ?? "-"}</td></tr>
    {/each}
  </tbody>
</table>
<style>
  .eq { width: 100%; border-collapse: collapse; font-size: 12px; }
  .eq th { text-align: left; color: var(--txt-3); font-size: 10.5px; text-transform: uppercase; padding: 4px 8px; border-bottom: 1px solid var(--hair); }
  .eq td { padding: 6px 8px; border-bottom: 1px solid var(--hair); color: var(--txt); }
</style>
