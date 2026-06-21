<!-- AI-generated (Claude) -->
<script lang="ts">
  import type { Theme } from "$lib/stores/app.svelte.ts";
  import { app } from "$lib/stores/app.svelte.ts";
  import type { RecentEntry } from "$lib/types.ts";
  import { saveAndEmitAppearance } from "$lib/prefs.ts";
  import AppearanceSection from "./prefs/AppearanceSection.svelte";

  let { onBack }: { onBack: () => void } = $props();

  function recentLabel(entry: RecentEntry): string {
    if (entry.kind === "Local") {
      return entry.path.split(/[\\/]/).pop() || entry.path;
    }
    const host = entry.url.replace(/^https?:\/\//, "");
    return `${entry.email}@${host}`;
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
        };
      }
    } else {
      app.openRecent(entry);
      onBack();
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

    <section class="recents-section">
      <h3 class="section-label">Recent Logbooks</h3>
      {#if app.recents.length === 0}
        <p class="recents-empty">No recent logbooks</p>
      {:else}
        <ul class="recents-list">
          {#each app.recents as entry}
            <li>
              <button class="recent-item" onclick={() => handleRecentTap(entry)}>
                {recentLabel(entry)}
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

  .recents-section {
    margin-top: var(--space-6, 24px);
  }

  .section-label {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--txt-2);
    margin: 0 0 var(--space-2) 0;
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

  .recent-item {
    display: block;
    width: 100%;
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
</style>
