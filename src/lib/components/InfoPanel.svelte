<!-- AI-generated (Claude) -->
<script lang="ts">
  import type { Dive } from "$lib/types.ts";
  import NotesTab from "./info/NotesTab.svelte";
  import EquipmentTab from "./info/EquipmentTab.svelte";
  import InformationTab from "./info/InformationTab.svelte";
  import SummaryTab from "./info/SummaryTab.svelte";

  let { dive }: { dive: Dive } = $props();
  const tabs = ["Notes", "Equipment", "Information", "Summary"] as const;
  let active = $state<(typeof tabs)[number]>("Notes");
</script>

<div class="info">
  <div class="tabs" role="tablist">
    {#each tabs as t}
      <button class="tab" class:active={active === t} role="tab" aria-selected={active === t} onclick={() => (active = t)}>{t}</button>
    {/each}
  </div>
  <div class="tab-body" role="tabpanel">
    {#if active === "Notes"}<NotesTab {dive} />
    {:else if active === "Equipment"}<EquipmentTab {dive} />
    {:else if active === "Information"}<InformationTab {dive} />
    {:else}<SummaryTab />{/if}
  </div>
</div>

<style>
  .info { display: flex; flex-direction: column; height: 100%; }
  .tabs { display: flex; gap: 2px; padding: 0 var(--space-2); height: 36px; flex: 0 0 36px; background: var(--panel); border-bottom: 1px solid var(--hair); align-items: flex-end; }
  .tab { position: relative; height: 30px; padding: 0 12px; border: 0; background: transparent; color: var(--txt-2); font: inherit; cursor: pointer; }
  .tab.active { color: var(--txt); font-weight: 560; }
  .tab.active::after { content: ""; position: absolute; left: 8px; right: 8px; bottom: -1px; height: 2px; background: var(--aqua); border-radius: 2px; }
  .tab-body { flex: 1; overflow: auto; padding: var(--space-4); }
</style>
