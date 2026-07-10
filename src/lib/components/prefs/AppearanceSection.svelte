<!-- AI-generated (Claude) -->
<script lang="ts">
  import type { Theme, UnitsPref } from "$lib/stores/app.svelte.ts";

  let {
    currentTheme,
    onThemeChange,
    currentUnits,
    onUnitsChange,
  }: {
    currentTheme: Theme;
    onThemeChange: (t: Theme) => void;
    currentUnits: UnitsPref;
    onUnitsChange: (u: UnitsPref) => void;
  } = $props();

  const themeOptions: { value: Theme; label: string }[] = [
    { value: "light", label: "Light" },
    { value: "dark", label: "Dark" },
    { value: "auto", label: "Auto" },
  ];

  const unitsOptions: { value: UnitsPref; label: string }[] = [
    { value: "auto", label: "Auto" },
    { value: "METRIC", label: "Metric" },
    { value: "IMPERIAL", label: "Imperial" },
  ];
</script>

<section>
  <h2 class="section-title">Appearance</h2>
  <hr class="divider" />
  <div class="row">
    <span class="label">Color Theme</span>
    <div class="radio-group" role="radiogroup" aria-label="Color Theme">
      {#each themeOptions as opt}
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
  <div class="row">
    <span class="label">Units</span>
    <div class="radio-group" role="radiogroup" aria-label="Units">
      {#each unitsOptions as opt}
        <label class="radio-label">
          <input
            type="radio"
            name="units"
            value={opt.value}
            checked={currentUnits === opt.value}
            onchange={() => onUnitsChange(opt.value)}
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
  .row { display: flex; align-items: center; gap: var(--space-4); margin-bottom: var(--space-2); }
  .label { color: var(--txt); min-width: 100px; }
  .radio-group { display: flex; gap: var(--space-4); }
  .radio-label { display: flex; align-items: center; gap: var(--space-1); cursor: pointer; }
</style>
