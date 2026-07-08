<!-- AI-generated (Claude) -->
<script lang="ts">
  import type { Theme } from "$lib/stores/app.svelte.ts";
  import type { LogLevel } from "$lib/prefs.ts";
  import PrefsSidebar from "./PrefsSidebar.svelte";
  import AppearanceSection from "./AppearanceSection.svelte";
  import LoggingSection from "./LoggingSection.svelte";

  let {
    currentTheme,
    onThemeChange,
    currentLogLevel,
    onLogLevelChange,
  }: {
    currentTheme: Theme;
    onThemeChange: (t: Theme) => void;
    currentLogLevel: LogLevel;
    onLogLevelChange: (l: LogLevel) => void;
  } = $props();

  type SectionId = "appearance" | "logging";
  let activeSection = $state<SectionId>("appearance");
</script>

<div class="shell">
  <div class="sidebar-wrap">
    <PrefsSidebar {activeSection} onSelect={(id) => (activeSection = id)} />
  </div>
  <div class="content">
    {#if activeSection === "appearance"}
      <AppearanceSection {currentTheme} {onThemeChange} />
    {:else if activeSection === "logging"}
      <LoggingSection currentLevel={currentLogLevel} onLevelChange={onLogLevelChange} />
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
