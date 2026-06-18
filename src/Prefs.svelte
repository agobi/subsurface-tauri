<!-- AI-generated (Claude) -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import type { Theme } from "$lib/stores/app.svelte.ts";
  import { loadAppearancePrefs, applyTheme, saveAndEmitAppearance } from "$lib/prefs.ts";
  import PrefsShell from "$lib/components/prefs/PrefsShell.svelte";

  let currentTheme = $state<Theme>("dark");
  let cleanupMatchMedia: (() => void) | undefined;

  onMount(async () => {
    const prefs = await loadAppearancePrefs();
    currentTheme = prefs.theme;
    applyTheme(prefs.theme);

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
</script>

<PrefsShell {currentTheme} onThemeChange={handleThemeChange} />
