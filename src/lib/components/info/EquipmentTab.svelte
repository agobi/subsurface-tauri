<!-- AI-generated (Claude) -->
<script lang="ts">
  import type { Dive } from "$lib/types.ts";
  import { fmtCylinderSize, fmtPressure } from "$lib/units.ts";
  import { app } from "$lib/stores/app.svelte.ts";
  let { dive }: { dive: Dive } = $props();
  let units = $derived(app.displayUnits);
  let volumeUnit = $derived(units === "IMPERIAL" ? "cuft" : "L");
  let pressureUnit = $derived(units === "IMPERIAL" ? "psi" : "bar");
</script>
<table class="eq">
  <thead><tr>
    <th>Type</th>
    <th>Size ({volumeUnit})</th>
    <th>Work ({pressureUnit})</th>
    <th>Start ({pressureUnit})</th>
    <th>End ({pressureUnit})</th>
    <th>O2%</th>
  </tr></thead>
  <tbody>
    {#each dive.cylinders as c}
      <tr>
        <td>{c.description}</td>
        <td class="tnum">{c.volumeL != null ? fmtCylinderSize(c.volumeL, c.workPressureBar, units, { suffix: false }) : "-"}</td>
        <td class="tnum">{c.workPressureBar != null ? fmtPressure(c.workPressureBar, units, { suffix: false }) : "-"}</td>
        <td class="tnum">{c.startBar != null ? fmtPressure(c.startBar, units, { suffix: false }) : "-"}</td>
        <td class="tnum">{c.endBar != null ? fmtPressure(c.endBar, units, { suffix: false }) : "-"}</td>
        <td class="tnum">{c.o2Percent ?? "-"}</td>
      </tr>
    {/each}
  </tbody>
</table>
<style>
  .eq { width: 100%; border-collapse: collapse; font-size: 12px; }
  .eq th { text-align: left; color: var(--txt-3); font-size: 10.5px; text-transform: uppercase; padding: 4px 8px; border-bottom: 1px solid var(--hair); }
  .eq td { padding: 6px 8px; border-bottom: 1px solid var(--hair); color: var(--txt); }
</style>
