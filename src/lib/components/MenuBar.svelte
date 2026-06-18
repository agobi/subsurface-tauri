<!-- AI-generated (Claude) -->
<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { app, type PanelKey } from "$lib/stores/app.svelte.ts";
  let activeMenu = $state<string | null>(null);
  const menus = ["File", "Edit", "Import", "Log", "View", "Help"];
  const panels: { key: PanelKey; label: string }[] = [
    { key: "list", label: "Dive List" },
    { key: "profile", label: "Dive Profile" },
    { key: "info", label: "Info" },
    { key: "map", label: "Map" },
  ];
  function toggle(name: string) { activeMenu = activeMenu === name ? null : name; }

  async function openLogbook() {
    activeMenu = null;
    const dir = await open({ directory: true });
    if (dir) await app.open(dir as string);
  }

  async function newLogbook() {
    activeMenu = null;
    const dir = await open({ directory: true });
    if (dir) await app.newLogbook(dir as string);
  }
</script>

<div class="menubar">
  {#each menus as m}
    <div class="menu-wrap">
      <button class="menu" onclick={() => toggle(m)}>{m}</button>
      {#if activeMenu === m}
        <div class="dropdown" role="menu">
          {#if m === "File"}
            <button role="menuitem" onclick={openLogbook}>Open Logbook…</button>
            <button role="menuitem" onclick={newLogbook}>New Logbook…</button>
          {:else if m === "View"}
            {#each panels as p}
              <button role="menuitem" onclick={() => app.togglePanel(p.key)}>
                <span class="check">{app.visiblePanels[p.key] ? "X" : ""}</span>{p.label}
              </button>
            {/each}
          {:else}
            <button role="menuitem" disabled>(prototype - visual only)</button>
          {/if}
        </div>
      {/if}
    </div>
  {/each}
</div>

<style>
  .menubar {
    height: 30px; flex: 0 0 30px; display: flex; align-items: center; gap: 2px;
    padding: 0 var(--space-2); background: var(--panel); border-bottom: 1px solid var(--hair);
  }
  .menu-wrap { position: relative; }
  .menu { height: 22px; padding: 0 10px; border: 0; background: transparent; color: var(--txt-2); font: inherit; font-size: 12.5px; cursor: pointer; border-radius: 5px; }
  .menu:hover { background: var(--elev); color: var(--txt); }
  .dropdown {
    position: absolute; top: 26px; left: 0; min-width: 210px; background: var(--panel);
    border: 1px solid var(--hair-strong); border-radius: var(--r-control); box-shadow: var(--shadow);
    padding: 6px; z-index: 10;
  }
  .dropdown button { display: flex; align-items: center; gap: 6px; width: 100%; height: 28px; padding: 0 8px; border: 0; background: transparent; color: var(--txt); font: inherit; text-align: left; cursor: pointer; border-radius: 5px; }
  .dropdown button:hover:not(:disabled) { background: var(--blue); color: #fff; }
  .dropdown button:disabled { color: var(--txt-3); cursor: default; }
  .check { width: 14px; color: var(--aqua); }
</style>
