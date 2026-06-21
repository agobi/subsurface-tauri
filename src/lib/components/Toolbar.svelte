<!-- AI-generated (Claude) -->
<script lang="ts">
  let {
    onSearch,
    isCloud = false,
    onSync,
  }: {
    onSearch: (q: string) => void;
    isCloud?: boolean;
    onSync?: () => Promise<void>;
  } = $props();

  let query = $state("");
  let syncing = $state(false);
  let syncError = $state<string | null>(null);

  const actions = ["Add dive", "Download", "Plan", "Statistics", "Filter"];

  async function handleSync() {
    syncing = true;
    syncError = null;
    try {
      await onSync?.();
    } catch (e) {
      syncError = typeof e === "string" ? e : "Sync failed.";
    } finally {
      syncing = false;
    }
  }
</script>

<div class="toolbar-wrap">
  <div class="toolbar">
    {#each actions as a, i}
      <button class="tbtn" class:primary={i === 0}>{a}</button>
      {#if i === 0}<span class="tb-div"></span>{/if}
    {/each}
    {#if isCloud}
      <span class="tb-div"></span>
      <button class="tbtn" onclick={handleSync} disabled={syncing}>
        {syncing ? "Syncing…" : "Sync"}
      </button>
    {/if}
    <div class="search">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="7"/><path d="m21 21-4.3-4.3"/></svg>
      <input placeholder="Search dives" bind:value={query} oninput={(e) => onSearch(e.currentTarget.value)} />
    </div>
  </div>
  {#if syncError}
    <div class="sync-error" role="alert">
      <span>{syncError}</span>
      <button class="dismiss" aria-label="dismiss" onclick={() => (syncError = null)}>×</button>
    </div>
  {/if}
</div>

<style>
  .toolbar-wrap { display: flex; flex-direction: column; flex: 0 0 auto; }
  .toolbar { height: 46px; display: flex; align-items: center; gap: var(--space-2); padding: 0 var(--space-3); background: var(--panel); border-bottom: 1px solid var(--hair); }
  .tbtn { display: flex; align-items: center; gap: 7px; height: 30px; padding: 0 12px; border-radius: var(--r-control); border: 1px solid transparent; background: transparent; color: var(--txt-2); font: inherit; cursor: pointer; }
  .tbtn:hover { background: var(--elev); color: var(--txt); }
  .tbtn.primary { background: var(--blue); color: #fff; font-weight: 560; }
  .tb-div { width: 1px; height: 22px; background: var(--hair); margin: 0 2px; }
  .search { display: flex; align-items: center; gap: 7px; height: 30px; margin-left: auto; padding: 0 12px; background: var(--panel-2); border: 1px solid var(--hair); border-radius: var(--r-pill); }
  .search input { flex: 1; background: 0; border: 0; outline: 0; color: var(--txt); font: inherit; font-size: 12.5px; }
  .search svg { width: 14px; height: 14px; color: var(--txt-3); }
  .sync-error { display: flex; align-items: center; justify-content: space-between; padding: 6px var(--space-3); background: var(--red, #c0392b); color: #fff; font-size: 12.5px; }
  .dismiss { background: transparent; border: none; color: #fff; cursor: pointer; font-size: 16px; line-height: 1; padding: 0 4px; }
</style>
