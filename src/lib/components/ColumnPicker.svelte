<!-- AI-generated (Claude) -->
<script lang="ts">
  import { app } from "$lib/stores/app.svelte.ts";
  import { ALL_COLS } from "$lib/diveListColumns.ts";

  let { open = $bindable() }: { open: boolean } = $props();
  let container: HTMLDivElement;

  function handleWindowClick(e: MouseEvent) {
    if (container && !container.contains(e.target as Node)) open = false;
  }
</script>

<svelte:window
  onclick={handleWindowClick}
  onkeydown={(e) => e.key === "Escape" && (open = false)}
/>

<div class="picker" bind:this={container}>
  {#each ALL_COLS as col}
    {@const checked = app.diveListPrefs.colOrder.includes(col.id)}
    <label class="row">
      <input type="checkbox" {checked} onchange={() => app.toggleColumn(col.id)} />
      {col.label}
    </label>
  {/each}
</div>

<style>
  .picker {
    position: absolute;
    right: 0;
    top: 100%;
    z-index: 100;
    background: var(--panel);
    border: 1px solid var(--hair);
    border-radius: 4px;
    padding: 4px 0;
    min-width: 140px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 12px;
    font-size: 12px;
    cursor: pointer;
  }
  .row:hover { background: var(--elev); }
</style>
