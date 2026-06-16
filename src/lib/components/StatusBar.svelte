<!-- AI-generated (Claude) -->
<script lang="ts">
  let { diveCount, decoModel, synced }: { diveCount: number; decoModel: string; synced: boolean } = $props();
</script>

<div class="statusbar">
  <span class="sync" class:ok={synced}>{synced ? "Synced" : "Offline copy"}</span>
  <span class="sep">|</span>
  <span class="tnum">{diveCount} dives</span>
  <span class="deco">{decoModel}</span>
</div>

<style>
  .statusbar { height: 26px; flex: 0 0 26px; display: flex; align-items: center; gap: var(--space-2); padding: 0 var(--space-3); background: var(--panel); border-top: 1px solid var(--hair); color: var(--txt-2); font-size: 11.5px; }
  .sync { color: var(--txt-3); } .sync.ok { color: var(--rate-ok); }
  .sep { color: var(--txt-3); } .deco { margin-left: auto; color: var(--txt-3); }
</style>
