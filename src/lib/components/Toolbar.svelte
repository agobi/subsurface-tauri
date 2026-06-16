<!-- AI-generated (Claude) -->
<script lang="ts">
  let { onSearch }: { onSearch: (q: string) => void } = $props();
  let query = $state("");
  const actions = ["Add dive", "Download", "Plan", "Statistics", "Filter"];
  $effect(() => onSearch(query));
</script>

<div class="toolbar">
  {#each actions as a, i}
    <button class="tbtn" class:primary={i === 0}>{a}</button>
    {#if i === 0}<span class="tb-div"></span>{/if}
  {/each}
  <div class="search">
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="7"/><path d="m21 21-4.3-4.3"/></svg>
    <input placeholder="Search dives" bind:value={query} />
  </div>
</div>

<style>
  .toolbar { height: 46px; flex: 0 0 46px; display: flex; align-items: center; gap: var(--space-2); padding: 0 var(--space-3); background: var(--panel); border-bottom: 1px solid var(--hair); }
  .tbtn { display: flex; align-items: center; gap: 7px; height: 30px; padding: 0 12px; border-radius: var(--r-control); border: 1px solid transparent; background: transparent; color: var(--txt-2); font: inherit; cursor: pointer; }
  .tbtn:hover { background: var(--elev); color: var(--txt); }
  .tbtn.primary { background: var(--blue); color: #fff; font-weight: 560; }
  .tb-div { width: 1px; height: 22px; background: var(--hair); margin: 0 2px; }
  .search { display: flex; align-items: center; gap: 7px; height: 30px; margin-left: auto; padding: 0 12px; background: var(--panel-2); border: 1px solid var(--hair); border-radius: var(--r-pill); }
  .search input { flex: 1; background: 0; border: 0; outline: 0; color: var(--txt); font: inherit; font-size: 12.5px; }
  .search svg { width: 14px; height: 14px; color: var(--txt-3); }
</style>
