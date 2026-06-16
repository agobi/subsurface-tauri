<!-- AI-generated (Claude) -->
<script lang="ts">
  import { app } from "$lib/stores/app.svelte.ts";
  let { fileName }: { fileName: string } = $props();
</script>

<div class="titlebar">
  <div class="lights"><span class="r"></span><span class="y"></span><span class="g"></span></div>
  <div class="title">{fileName} <span class="sub">Subsurface</span></div>
  <button class="theme" aria-label="Toggle theme" onclick={() => app.toggleTheme()}>
    {app.theme === "dark" ? "Light" : "Dark"}
  </button>
</div>

<style>
  .titlebar {
    height: 38px; flex: 0 0 38px; display: flex; align-items: center; gap: var(--space-3);
    padding: 0 var(--space-3); background: var(--panel); border-bottom: 1px solid var(--hair);
  }
  .title { flex: 1; text-align: center; font-size: 12.5px; font-weight: 560; }
  .sub { color: var(--txt-3); font-size: 11.5px; }
  .lights { display: flex; gap: 8px; }
  .lights span { width: 12px; height: 12px; border-radius: 999px; }
  .lights .r { background: #ff5f57; } .lights .y { background: #febc2e; } .lights .g { background: #28c840; }
  .theme {
    height: 24px; padding: 0 10px; color: var(--txt-2); background: var(--panel-2);
    border: 1px solid var(--hair); border-radius: var(--r-control); cursor: pointer; font: inherit; font-size: 11.5px;
  }
</style>
