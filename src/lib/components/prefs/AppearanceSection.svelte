<!-- AI-generated (Claude) -->
<script lang="ts">
  import type { Theme } from "$lib/stores/app.svelte.ts";

  let {
    currentTheme,
    onThemeChange,
  }: { currentTheme: Theme; onThemeChange: (t: Theme) => void } = $props();

  const options: { value: Theme; label: string }[] = [
    { value: "light", label: "Light" },
    { value: "dark", label: "Dark" },
    { value: "auto", label: "Auto" },
  ];
</script>

<section>
  <h2 class="section-title">Appearance</h2>
  <hr class="divider" />
  <div class="row">
    <span class="label">Color Theme</span>
    <div class="radio-group">
      {#each options as opt}
        <label class="radio-label">
          <input
            type="radio"
            name="theme"
            value={opt.value}
            checked={currentTheme === opt.value}
            onchange={() => onThemeChange(opt.value)}
          />
          {opt.label}
        </label>
      {/each}
    </div>
  </div>
</section>

<style>
  .section-title { margin: 0 0 var(--space-2); font-size: 1rem; font-weight: 600; }
  .divider { border: none; border-top: 1px solid var(--hair); margin-bottom: var(--space-4); }
  .row { display: flex; align-items: center; gap: var(--space-4); }
  .label { color: var(--txt); min-width: 100px; }
  .radio-group { display: flex; gap: var(--space-4); }
  .radio-label { display: flex; align-items: center; gap: var(--space-1); cursor: pointer; }
</style>
