<!-- AI-generated (Claude) -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import type { Theme } from "$lib/stores/app.svelte.ts";
  import {
    loadAppearancePrefs,
    applyTheme,
    saveAndEmitAppearance,
    loadLoggingPrefs,
    applyLogLevel,
    type LogLevel,
  } from "$lib/prefs.ts";
  import PrefsShell from "$lib/components/prefs/PrefsShell.svelte";

  let currentTheme = $state<Theme>("dark");
  let currentLogLevel = $state<LogLevel>("info");
  let cleanupMatchMedia: (() => void) | undefined;

  onMount(async () => {
    try {
      const prefs = await loadAppearancePrefs();
      currentTheme = prefs.theme;
      applyTheme(prefs.theme);
    } catch (e) {
      console.error("Failed to load appearance prefs:", e);
      applyTheme(currentTheme);
    }

    try {
      currentLogLevel = await loadLoggingPrefs();
    } catch (e) {
      console.error("Failed to load logging prefs:", e);
    }

    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    const handleColorScheme = () => applyTheme(currentTheme);
    mql.addEventListener("change", handleColorScheme);
    cleanupMatchMedia = () => mql.removeEventListener("change", handleColorScheme);
  });

  onDestroy(() => cleanupMatchMedia?.());

  async function handleThemeChange(theme: Theme) {
    currentTheme = theme;
    applyTheme(theme);
    await saveAndEmitAppearance({ theme });
  }

  async function handleLogLevelChange(level: LogLevel) {
    currentLogLevel = level;
    await applyLogLevel(level);
  }
</script>

<PrefsShell
  {currentTheme}
  onThemeChange={handleThemeChange}
  {currentLogLevel}
  onLogLevelChange={handleLogLevelChange}
/>
