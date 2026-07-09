<!-- AI-generated (Claude) -->
<script lang="ts">
  import { onMount } from "svelte";
  import { app } from "$lib/stores/app.svelte.ts";
  import { fmtRecentLabel } from "$lib/format.ts";

  onMount(() => {
    app.loadRecents();
  });
</script>

<section>
  <div class="header-row">
    <h2 class="section-title">Recent Logbooks</h2>
    {#if app.recents.length > 0}
      <button class="clear-btn" onclick={() => app.clearRecents()}>Clear All</button>
    {/if}
  </div>
  <hr class="divider" />
  {#if app.recents.length === 0}
    <p class="empty">No recent logbooks</p>
  {:else}
    <ul class="recents-list">
      {#each app.recents as entry, i}
        <li class="recent-row">
          <span class="recent-label">{fmtRecentLabel(entry)}</span>
          <button
            class="remove-btn"
            aria-label={`Remove ${fmtRecentLabel(entry)}`}
            onclick={() => app.removeRecent(i)}
          >
            ×
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .section-title { margin: 0; font-size: 1rem; font-weight: 600; }
  .divider { border: none; border-top: 1px solid var(--hair); margin-bottom: var(--space-4); }
  .header-row { display: flex; align-items: center; justify-content: space-between; margin-bottom: var(--space-2); }
  .clear-btn {
    background: none;
    border: none;
    font: inherit;
    font-size: 0.8125rem;
    color: var(--accent, var(--txt));
    cursor: pointer;
    padding: var(--space-1) var(--space-2);
  }
  .empty { color: var(--txt-2); font-size: 0.875rem; margin: 0; }
  .recents-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 2px; }
  .recent-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--hair);
    border-radius: var(--r-control, 6px);
    background: var(--panel);
  }
  .recent-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.875rem;
    color: var(--txt);
  }
  .remove-btn {
    flex: 0 0 auto;
    background: none;
    border: none;
    color: var(--txt-2);
    font-size: 1.125rem;
    line-height: 1;
    cursor: pointer;
    padding: 0 var(--space-1);
  }
  .remove-btn:hover { color: var(--txt); }
</style>
