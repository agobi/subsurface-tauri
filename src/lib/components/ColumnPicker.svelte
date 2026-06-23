<!-- AI-generated (Claude) -->
<script lang="ts">
  import { app } from "$lib/stores/app.svelte.ts";
  import { ALL_COLS } from "$lib/diveListColumns.ts";
  import type { ColId } from "$lib/diveListColumns.ts";

  let { open = $bindable() }: { open: boolean } = $props();
  let container: HTMLDivElement;
  let dragFrom: ColId | null = null;
  let dropTarget = $state<ColId | null>(null);

  function handleWindowClick(e: MouseEvent) {
    if (container && !container.contains(e.target as Node)) open = false;
  }
</script>

<svelte:window
  onclick={handleWindowClick}
  onkeydown={(e) => e.key === "Escape" && (open = false)}
/>

<div class="picker" bind:this={container}>
  {#each app.diveListPrefs.colOrder as id}
    {@const col = ALL_COLS.find(c => c.id === id)!}
    {@const hidden = app.diveListPrefs.hiddenCols.includes(id)}
    <label
      class="row"
      class:hidden
      class:drag-over={dropTarget === id}
      data-testid="col-row-{id}"
      draggable="true"
      ondragstart={() => (dragFrom = id)}
      ondragover={(e) => { e.preventDefault(); dropTarget = id; }}
      ondrop={() => {
        if (dragFrom && dragFrom !== id) app.reorderColumn(dragFrom, id);
        dragFrom = null;
        dropTarget = null;
      }}
      ondragleave={() => { if (dropTarget === id) dropTarget = null; }}
      ondragend={() => { dragFrom = null; dropTarget = null; }}
    >
      <span class="handle" aria-hidden="true">⠿</span>
      <input type="checkbox" checked={!hidden} onchange={() => app.toggleColumn(id)} />
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
    min-width: 160px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 12px;
    font-size: 12px;
    cursor: grab;
    border-top: 2px solid transparent;
  }
  .row:hover { background: var(--elev); }
  .row.hidden {
    opacity: 0.45;
    color: var(--txt-3);
  }
  .row.drag-over {
    border-top-color: var(--blue);
  }
  .handle {
    font-size: 14px;
    color: var(--txt-3);
    user-select: none;
    cursor: grab;
  }
</style>
