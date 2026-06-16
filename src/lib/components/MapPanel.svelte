<!-- AI-generated (Claude) -->
<script lang="ts">
  let { siteName, gps }: { siteName?: string; gps?: { lat: number; lon: number } } = $props();
</script>

<div class="map">
  {#if siteName}
    <svg viewBox="0 0 400 240" role="img" aria-label="Stylized map">
      <rect width="400" height="240" fill="var(--panel-2)" />
      <path d="M0 160 Q100 140 200 160 T400 150" fill="none" stroke="var(--aqua)" stroke-opacity="0.4" />
      <circle data-testid="map-marker" cx="200" cy="120" r="7" fill="var(--rate-fast)" stroke="#fff" stroke-width="2" />
    </svg>
    <div class="cap">{siteName}{#if gps} <span class="tnum">({gps.lat.toFixed(3)}, {gps.lon.toFixed(3)})</span>{/if}</div>
  {:else}
    <div class="empty">No site for this dive</div>
  {/if}
</div>

<style>
  .map { height: 100%; display: flex; flex-direction: column; }
  .map svg { flex: 1; min-height: 0; }
  .cap { padding: var(--space-2) var(--space-3); font-size: 12px; color: var(--txt-2); border-top: 1px solid var(--hair); }
  .empty { margin: auto; color: var(--txt-3); font-size: 12px; }
</style>
