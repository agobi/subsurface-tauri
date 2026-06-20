<!-- AI-generated (Claude) -->
<script lang="ts">
  import type { Theme } from "$lib/stores/app.svelte.ts";
  import { app } from "$lib/stores/app.svelte.ts";
  import { saveAndEmitAppearance } from "$lib/prefs.ts";
  import AppearanceSection from "./prefs/AppearanceSection.svelte";

  let { onBack }: { onBack: () => void } = $props();

  async function handleThemeChange(theme: Theme) {
    app.setTheme(theme);
    await saveAndEmitAppearance({ theme });
  }
</script>

<div class="settings-screen" data-testid="mobile-settings-screen">
  <header class="settings-header">
    <button class="back-btn" onclick={onBack} aria-label="Back">←</button>
    <span class="settings-title">Settings</span>
  </header>
  <div class="settings-body">
    <AppearanceSection currentTheme={app.theme} onThemeChange={handleThemeChange} />
  </div>
</div>

<style>
  .settings-screen {
    position: fixed;
    inset: 0;
    background: var(--bg);
    color: var(--txt);
    display: flex;
    flex-direction: column;
    animation: slide-in 220ms ease-out;
  }

  @keyframes slide-in {
    from { transform: translateX(100%); }
    to   { transform: translateX(0); }
  }

  .settings-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    padding-top: calc(var(--space-3) + env(safe-area-inset-top));
    border-bottom: 1px solid var(--hair);
    background: var(--panel);
  }

  .back-btn {
    background: none;
    border: none;
    font: inherit;
    font-size: 1.2rem;
    color: var(--blue);
    cursor: pointer;
    padding: 0 var(--space-2) 0 0;
    min-height: 44px;
  }

  .settings-title {
    font-weight: 600;
    font-size: 1rem;
  }

  .settings-body {
    flex: 1;
    overflow: auto;
    padding: var(--space-4);
    padding-bottom: calc(var(--space-4) + env(safe-area-inset-bottom));
  }
</style>
