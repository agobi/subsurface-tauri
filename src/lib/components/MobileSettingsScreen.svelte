<!-- AI-generated (Claude) -->
<script lang="ts">
  import { onMount } from "svelte";
  import type { Theme } from "$lib/stores/app.svelte.ts";
  import { app } from "$lib/stores/app.svelte.ts";
  import type { RecentEntry } from "$lib/types.ts";
  import { fmtRecentLabel } from "$lib/format.ts";
  import { saveAndEmitAppearance, loadLoggingPrefs, applyLogLevel, type LogLevel } from "$lib/prefs.ts";
  import AppearanceSection from "./prefs/AppearanceSection.svelte";
  import LoggingSection from "./prefs/LoggingSection.svelte";

  let { onBack }: { onBack: () => void } = $props();

  let currentLogLevel = $state<LogLevel>("info");

  onMount(async () => {
    try {
      currentLogLevel = await loadLoggingPrefs();
    } catch (e) {
      console.error("Failed to load logging prefs:", e);
    }
  });

  async function handleLogLevelChange(level: LogLevel) {
    currentLogLevel = level;
    await applyLogLevel(level);
  }

  async function handleThemeChange(theme: Theme) {
    app.setTheme(theme);
    await saveAndEmitAppearance({ theme });
  }

  async function handleRecentTap(entry: RecentEntry) {
    if (entry.kind === "Cloud") {
      try {
        await app.openRecentCloud(entry.email);
        onBack();
      } catch {
        app.showCloudDialog = {
          email: entry.email,
          message: "Saved credentials could not be loaded. Please sign in again.",
          onSuccess: onBack,
        };
      }
    } else {
      try {
        await app.openRecent(entry);
        onBack();
      } catch (e) {
        console.error("Failed to open recent logbook:", e);
      }
    }
  }
</script>

<div class="settings-screen" data-testid="mobile-settings-screen">
  <header class="settings-header">
    <button class="back-btn" onclick={onBack} aria-label="Back">←</button>
    <span class="settings-title">Settings</span>
  </header>
  <div class="settings-body">
    <AppearanceSection currentTheme={app.theme} onThemeChange={handleThemeChange} />

    <div class="logging-section-wrap">
      <LoggingSection currentLevel={currentLogLevel} onLevelChange={handleLogLevelChange} />
    </div>

    <section class="recents-section">
      <div class="recents-header">
        <h3 class="section-label">Recent Logbooks</h3>
        {#if app.recents.length > 0}
          <button class="clear-btn" onclick={() => app.clearRecents()}>Clear All</button>
        {/if}
      </div>
      {#if app.recents.length === 0}
        <p class="recents-empty">No recent logbooks</p>
      {:else}
        <ul class="recents-list">
          {#each app.recents as entry, i}
            <li class="recent-row">
              <button class="recent-item" onclick={() => handleRecentTap(entry)}>
                {fmtRecentLabel(entry)}
              </button>
              <button
                class="remove-btn"
                aria-label={`Remove ${fmtRecentLabel(entry)}`}
                onclick={() => app.removeRecent(i)}
              >
                ×
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </section>
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

  .logging-section-wrap,
  .recents-section {
    margin-top: var(--space-6, 24px);
  }

  .recents-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 0 0 var(--space-2) 0;
  }

  .section-label {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--txt-2);
    margin: 0;
  }

  .clear-btn {
    background: none;
    border: none;
    font: inherit;
    font-size: 0.8125rem;
    color: var(--accent, var(--txt));
    cursor: pointer;
    padding: var(--space-1) 0;
  }

  .recents-empty {
    font-size: 0.875rem;
    color: var(--txt-3);
    margin: 0;
  }

  .recents-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .recent-row {
    display: flex;
    align-items: stretch;
    gap: var(--space-1);
  }

  .recent-item {
    display: block;
    flex: 1;
    min-width: 0;
    text-align: left;
    background: var(--panel);
    border: 1px solid var(--hair);
    border-radius: var(--r-control, 6px);
    padding: var(--space-2) var(--space-3);
    font: inherit;
    font-size: 0.875rem;
    color: var(--txt);
    cursor: pointer;
    min-height: 44px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .recent-item:active {
    background: var(--panel-2);
  }

  .remove-btn {
    flex: 0 0 44px;
    background: var(--panel);
    border: 1px solid var(--hair);
    border-radius: var(--r-control, 6px);
    color: var(--txt-2);
    font-size: 1.125rem;
    line-height: 1;
    cursor: pointer;
  }

  .remove-btn:active {
    background: var(--panel-2);
  }
</style>
