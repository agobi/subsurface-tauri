<!-- AI-generated (Claude) -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import type { Theme, UnitsPref } from "$lib/stores/app.svelte.ts";
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
  let currentUnits = $state<UnitsPref>("auto");
  let currentLogLevel = $state<LogLevel>("info");
  let cleanupMatchMedia: (() => void) | undefined;

  onMount(async () => {
    try {
      const prefs = await loadAppearancePrefs();
      currentTheme = prefs.theme;
      currentUnits = prefs.units;
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
    await saveAndEmitAppearance({ theme, units: currentUnits });
  }

  async function handleUnitsChange(units: UnitsPref) {
    currentUnits = units;
    await saveAndEmitAppearance({ theme: currentTheme, units });
  }

  async function handleLogLevelChange(level: LogLevel) {
    currentLogLevel = level;
    await applyLogLevel(level);
  }
</script>

<PrefsShell
  {currentTheme}
  onThemeChange={handleThemeChange}
  {currentUnits}
  onUnitsChange={handleUnitsChange}
  {currentLogLevel}
  onLogLevelChange={handleLogLevelChange}
/>
