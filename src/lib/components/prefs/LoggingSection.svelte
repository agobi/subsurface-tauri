<!-- AI-generated (Claude) -->
<script lang="ts">
  import type { LogLevel } from "$lib/prefs.ts";

  let {
    currentLevel,
    onLevelChange,
  }: { currentLevel: LogLevel; onLevelChange: (l: LogLevel) => void } = $props();

  const options: { value: LogLevel; label: string }[] = [
    { value: "error", label: "Error" },
    { value: "warn", label: "Warning" },
    { value: "info", label: "Info" },
    { value: "debug", label: "Debug" },
    { value: "trace", label: "Trace" },
  ];
</script>

<section>
  <h2 class="section-title">Logging</h2>
  <hr class="divider" />
  <div class="row">
    <span class="label">Log Level</span>
    <div class="radio-group">
      {#each options as opt}
        <label class="radio-label">
          <input
            type="radio"
            name="logLevel"
            value={opt.value}
            checked={currentLevel === opt.value}
            onchange={() => onLevelChange(opt.value)}
          />
          {opt.label}
        </label>
      {/each}
    </div>
  </div>
  <p class="hint">
    Higher verbosity (Debug/Trace) helps diagnose dive-computer download issues but
    produces much larger log files. Takes effect immediately.
  </p>
</section>

<style>
  .section-title { margin: 0 0 var(--space-2); font-size: 1rem; font-weight: 600; }
  .divider { border: none; border-top: 1px solid var(--hair); margin-bottom: var(--space-4); }
  .row { display: flex; align-items: center; gap: var(--space-4); }
  .label { color: var(--txt); min-width: 100px; }
  .radio-group { display: flex; gap: var(--space-4); flex-wrap: wrap; }
  .radio-label { display: flex; align-items: center; gap: var(--space-1); cursor: pointer; }
  .hint { margin-top: var(--space-4); color: var(--txt-muted, var(--txt)); font-size: 0.85rem; }
</style>
