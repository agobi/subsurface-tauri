<!-- AI-generated (Claude) -->
<script lang="ts">
  import type { Theme, UnitsPref } from "$lib/stores/app.svelte.ts";
  import type { LogLevel } from "$lib/prefs.ts";
  import PrefsSidebar from "./PrefsSidebar.svelte";
  import AppearanceSection from "./AppearanceSection.svelte";
  import LoggingSection from "./LoggingSection.svelte";
  import RecentsSection from "./RecentsSection.svelte";
  import DcDownloadSection from "./DcDownloadSection.svelte";

  let {
    currentTheme,
    onThemeChange,
    currentUnits,
    onUnitsChange,
    currentLogLevel,
    onLogLevelChange,
  }: {
    currentTheme: Theme;
    onThemeChange: (t: Theme) => void;
    currentUnits: UnitsPref;
    onUnitsChange: (u: UnitsPref) => void;
    currentLogLevel: LogLevel;
    onLogLevelChange: (l: LogLevel) => void;
  } = $props();

  type SectionId = "appearance" | "logging" | "recents" | "dcDownload";
  let activeSection = $state<SectionId>("appearance");
</script>

<div class="shell">
  <div class="sidebar-wrap">
    <PrefsSidebar {activeSection} onSelect={(id) => (activeSection = id)} />
  </div>
  <div class="content">
    {#if activeSection === "appearance"}
      <AppearanceSection {currentTheme} {onThemeChange} {currentUnits} {onUnitsChange} />
    {:else if activeSection === "logging"}
      <LoggingSection currentLevel={currentLogLevel} onLevelChange={onLogLevelChange} />
    {:else if activeSection === "recents"}
      <RecentsSection />
    {:else if activeSection === "dcDownload"}
      <DcDownloadSection />
    {/if}
  </div>
</div>

<style>
  .shell { display: flex; height: 100vh; background: var(--bg); color: var(--txt); }
  .content { flex: 1; padding: var(--space-4); }

  @media (max-width: 600px) {
    .sidebar-wrap { display: none; }
  }
</style>
