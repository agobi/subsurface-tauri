<!-- AI-generated (Claude) -->
<script lang="ts">
  let {
    activeSection,
    onSelect,
  }: { activeSection: string; onSelect: (id: "appearance") => void } = $props();
</script>

<nav class="sidebar">
  <button
    class="nav-item"
    class:active={activeSection === "appearance"}
    onclick={() => onSelect("appearance")}
  >
    Appearance
  </button>
</nav>

<style>
  .sidebar {
    width: 180px;
    border-right: 1px solid var(--hair);
    background: var(--panel);
    padding: var(--space-2) 0;
    flex: 0 0 180px;
  }
  .nav-item {
    display: block;
    width: 100%;
    padding: var(--space-2) var(--space-4);
    background: none;
    border: none;
    text-align: left;
    color: var(--txt);
    font: inherit;
    cursor: pointer;
  }
  .nav-item:hover { background: var(--hair); }
  .nav-item.active { background: var(--elev); }
</style>
