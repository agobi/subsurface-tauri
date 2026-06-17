<!-- AI-generated (Claude) -->
<script lang="ts">
  import { app } from "$lib/stores/app.svelte.ts";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import MenuBar from "$lib/components/MenuBar.svelte";
  import Toolbar from "$lib/components/Toolbar.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import QuadrantGrid from "$lib/components/QuadrantGrid.svelte";

  let search = $state("");
  $effect(() => {
    document.documentElement.setAttribute("data-theme", app.theme);
  });
</script>

<div class="app">
  <TitleBar fileName="logbook.ssrf" />
  <MenuBar />
  <Toolbar onSearch={(q) => (search = q)} />
  <QuadrantGrid query={search} />
  <StatusBar diveCount={app.dives.length} decoModel={app.selectedDive?.decoModel ?? "-"} synced={true} />
</div>

<style>
  .app { display: flex; flex-direction: column; height: 100vh; }
</style>
