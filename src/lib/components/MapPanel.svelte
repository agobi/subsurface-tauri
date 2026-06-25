<!-- AI-generated (Claude) -->
<script lang="ts">
  import { onDestroy } from 'svelte';
  import L from 'leaflet';
  import 'leaflet/dist/leaflet.css';
  import { cachedTileLayer } from '$lib/map/cachedTileLayer.ts';

  let { siteName, gps }: { siteName?: string; gps?: { lat: number; lon: number } } = $props();

  let mapEl: HTMLDivElement | undefined = $state(undefined);
  let map: L.Map | undefined;
  let marker: L.Marker | undefined;

  const TILE_STYLE = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark_all' : 'light_all';
  const TILE_URL = `https://{s}.basemaps.cartocdn.com/${TILE_STYLE}/{z}/{x}/{y}{r}.png`;
  const ATTRIBUTION =
    '© <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors, ' +
    '© <a href="https://carto.com/attributions">CARTO</a>';

  $effect(() => {
    if (!mapEl) {
      if (map) { map.remove(); map = undefined; marker = undefined; }
      return;
    }
    if (!gps) return;
    if (!map) {
      map = L.map(mapEl, { zoomControl: true });
      cachedTileLayer(TILE_URL, { attribution: ATTRIBUTION }).addTo(map);
      map.setView([gps.lat, gps.lon], 13);
      const icon = L.divIcon({
        className: '',
        html: '<div class="lf-marker"></div>',
        iconSize: [14, 14],
        iconAnchor: [7, 7],
        popupAnchor: [0, -7],
      });
      marker = L.marker([gps.lat, gps.lon], { icon }).addTo(map);
      if (siteName) marker.bindPopup(siteName);
    } else {
      map.flyTo([gps.lat, gps.lon], 13);
      marker?.setLatLng([gps.lat, gps.lon]);
    }
  });

  onDestroy(() => {
    map?.remove();
  });
</script>

<div class="map">
  {#if gps}
    <div class="map-inner" bind:this={mapEl}></div>
    <div class="cap">
      {siteName ?? ''}<span class="tnum">&nbsp;({gps.lat.toFixed(3)}, {gps.lon.toFixed(3)})</span>
    </div>
  {:else if siteName}
    <div class="empty">No GPS for this site</div>
  {:else}
    <div class="empty">No site for this dive</div>
  {/if}
</div>

<style>
  .map { height: 100%; display: flex; flex-direction: column; }
  .map-inner { flex: 1; min-height: 0; }
  .cap {
    padding: var(--space-2) var(--space-3);
    font-size: 12px;
    color: var(--txt-2);
    border-top: 1px solid var(--hair);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .tnum { color: var(--txt-3); }
  .empty { margin: auto; color: var(--txt-3); font-size: 12px; }
  :global(.lf-marker) {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #39C2E0;
    border: 2px solid #fff;
    box-sizing: border-box;
  }
</style>
