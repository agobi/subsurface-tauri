<!-- AI-generated (Claude) -->
<script lang="ts">
  import { onMount } from "svelte";
  import { loadDcDownloadPrefs, saveDcDownloadPrefs, DEFAULT_DC_DOWNLOAD_PREFS, type DcDownloadPrefs } from "$lib/prefs.ts";

  let prefs = $state<DcDownloadPrefs>({ ...DEFAULT_DC_DOWNLOAD_PREFS });

  onMount(async () => {
    prefs = await loadDcDownloadPrefs();
  });

  async function onGapChange(value: string) {
    const parsed = Number(value);
    const mergeGapMinutes = Number.isFinite(parsed) && parsed >= 0 ? Math.round(parsed) : DEFAULT_DC_DOWNLOAD_PREFS.mergeGapMinutes;
    prefs = { ...prefs, mergeGapMinutes };
    await saveDcDownloadPrefs(prefs);
  }
</script>

<section>
  <h2 class="section-title">Dive Computer</h2>
  <hr class="divider" />
  <div class="row">
    <label class="label" for="merge-gap-minutes">Merge segments within</label>
    <input
      id="merge-gap-minutes"
      type="number"
      min="0"
      step="1"
      value={prefs.mergeGapMinutes}
      onchange={(e) => onGapChange((e.target as HTMLInputElement).value)}
    />
    <span class="unit">minutes of each other</span>
  </div>
  <p class="hint">
    When downloading from a dive computer, a short segment (e.g. a pre-dive
    loop check) that starts within this many minutes of another dive's end is
    folded into it for review, instead of showing up as a separate dive every
    time.
  </p>
</section>

<style>
  .section-title { margin: 0 0 var(--space-2); font-size: 1rem; font-weight: 600; }
  .divider { border: none; border-top: 1px solid var(--hair); margin-bottom: var(--space-4); }
  .row { display: flex; align-items: center; gap: var(--space-2); }
  .label { color: var(--txt); }
  input[type="number"] {
    width: 4rem;
    background: var(--panel-2); border: 1px solid var(--hair);
    border-radius: var(--r-control); color: var(--txt);
    font: inherit; padding: 0.3rem 0.5rem;
  }
  .unit { color: var(--txt-2); }
  .hint { margin-top: var(--space-4); color: var(--txt-muted, var(--txt)); font-size: 0.85rem; }
</style>
