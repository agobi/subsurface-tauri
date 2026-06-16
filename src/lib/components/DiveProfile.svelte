<!-- AI-generated (Claude) -->
<script lang="ts">
  import type { Dive } from "$lib/types.ts";
  import { timeToX, depthToY, depthAxisMax, ascentRateClass } from "$lib/profile/profile-scale.ts";

  let { dive }: { dive: Dive } = $props();

  const VB = { w: 1000, h: 380 };
  const M = { l: 44, r: 18, t: 24, b: 34 };
  const plot = { x0: M.l, x1: VB.w - M.r, y0: M.t, y1: VB.h - M.b };

  let series = $state({ depth: true, temp: true, tank: true, ceiling: true, po2: false, mod: false, hr: false });

  let maxTime = $derived(dive.samples.length ? dive.samples[dive.samples.length - 1].timeSec : 1);
  let axisMax = $derived(depthAxisMax(dive.maxDepthM ?? Math.max(0, ...dive.samples.map((s) => s.depthM))));

  let depthSegments = $derived(
    dive.samples.slice(1).map((s, i) => {
      const prev = dive.samples[i];
      return {
        x1: timeToX(prev.timeSec, maxTime, plot), y1: depthToY(prev.depthM, axisMax, plot),
        x2: timeToX(s.timeSec, maxTime, plot), y2: depthToY(s.depthM, axisMax, plot),
        cls: ascentRateClass(prev, s),
      };
    })
  );

  let cursor = $state<{ x: number; sample: typeof dive.samples[number] } | null>(null);
  function onMove(e: MouseEvent) {
    const svg = e.currentTarget as unknown as SVGSVGElement;
    const r = svg.getBoundingClientRect();
    const px = ((e.clientX - r.left) / r.width) * VB.w;
    const t = ((px - plot.x0) / (plot.x1 - plot.x0)) * maxTime;
    let nearest = dive.samples[0];
    for (const s of dive.samples) if (Math.abs(s.timeSec - t) < Math.abs(nearest.timeSec - t)) nearest = s;
    cursor = { x: timeToX(nearest.timeSec, maxTime, plot), sample: nearest };
  }

  const toggles: { key: keyof typeof series; label: string }[] = [
    { key: "temp", label: "Temperature" }, { key: "tank", label: "Tank pressure" },
    { key: "ceiling", label: "Ceiling" }, { key: "po2", label: "pO2" },
    { key: "mod", label: "MOD" }, { key: "hr", label: "Heart rate" },
  ];
  function fmtTime(sec: number) { return `${Math.floor(sec / 60)}:${String(sec % 60).padStart(2, "0")}`; }
  function gridLines(max: number) { const out: number[] = []; for (let m = 0; m <= max; m += 5) out.push(m); return out; }
</script>

<div class="profile">
  <div class="ptoolbar">
    {#each toggles as t}
      <button class="ptog" class:on={series[t.key]} aria-pressed={series[t.key]}
        onclick={() => (series = { ...series, [t.key]: !series[t.key] })}>{t.label}</button>
    {/each}
  </div>

  <!-- svelte-ignore a11y_no_static_element_interactions — interactive crosshair tooltip on a decorative chart; role=img already describes the content -->
  <svg viewBox="0 0 {VB.w} {VB.h}" role="img" aria-label="Dive depth profile" onmousemove={onMove} onmouseleave={() => (cursor = null)}>
    <defs>
      <linearGradient id="water" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="var(--aqua)" stop-opacity="0.18" />
        <stop offset="1" stop-color="var(--aqua)" stop-opacity="0.02" />
      </linearGradient>
    </defs>

    {#each gridLines(axisMax) as m}
      <line x1={plot.x0} y1={depthToY(m, axisMax, plot)} x2={plot.x1} y2={depthToY(m, axisMax, plot)} stroke="var(--grid)" stroke-width="1" />
      <text x={plot.x0 - 8} y={depthToY(m, axisMax, plot) + 3} text-anchor="end" fill="var(--txt-3)" font-size="10" font-family="var(--font-mono)">{m}m</text>
    {/each}

    {#if series.depth}
      <path d={`M ${plot.x0} ${plot.y0} ` + dive.samples.map((s) => `L ${timeToX(s.timeSec, maxTime, plot)} ${depthToY(s.depthM, axisMax, plot)}`).join(" ") + ` L ${plot.x1} ${plot.y0} Z`} fill="url(#water)" />
      {#each depthSegments as seg}
        <line x1={seg.x1} y1={seg.y1} x2={seg.x2} y2={seg.y2} stroke-width="2"
          stroke={seg.cls === "fast" ? "var(--rate-fast)" : seg.cls === "ok" ? "var(--rate-ok)" : "var(--aqua)"}
          stroke-dasharray={seg.cls === "fast" ? "3 2" : null} />
      {/each}
    {/if}

    {#if series.temp}
      <polyline fill="none" stroke="var(--teal)" stroke-width="1.5"
        points={dive.samples.filter((s) => s.tempC != null).map((s) => `${timeToX(s.timeSec, maxTime, plot)},${plot.y1 - ((s.tempC ?? 0) / 40) * (plot.y1 - plot.y0)}`).join(" ")} />
    {/if}

    {#if series.tank}
      <polyline fill="none" stroke="var(--amber)" stroke-width="1.5"
        points={dive.samples.filter((s) => s.pressureBar != null).map((s) => `${timeToX(s.timeSec, maxTime, plot)},${plot.y1 - ((s.pressureBar ?? 0) / 250) * (plot.y1 - plot.y0)}`).join(" ")} />
    {/if}

    {#if cursor}
      <line x1={cursor.x} y1={plot.y0} x2={cursor.x} y2={plot.y1} stroke="var(--rate-fast)" stroke-width="1" stroke-dasharray="2 2" />
      <g transform={`translate(${Math.min(cursor.x + 8, plot.x1 - 150)}, ${plot.y0 + 8})`}>
        <rect width="150" height="64" rx="6" fill="rgba(0,0,0,.75)" />
        <text x="8" y="18" fill="#fff" font-size="11" font-family="var(--font-mono)">@ {fmtTime(cursor.sample.timeSec)}</text>
        <text x="8" y="34" fill="#fff" font-size="11" font-family="var(--font-mono)">D {cursor.sample.depthM.toFixed(1)} m</text>
        <text x="8" y="50" fill="#fff" font-size="11" font-family="var(--font-mono)">{cursor.sample.ndlSec != null ? `NDL ${fmtTime(cursor.sample.ndlSec)}` : ""}</text>
      </g>
    {/if}
  </svg>

  <div class="legend">
    <span><i class="sw" style="background:var(--aqua)"></i>Depth</span>
    <span><i class="sw" style="background:var(--rate-fast)"></i>Fast ascent (dashed)</span>
    <span><i class="sw" style="background:var(--teal)"></i>Temp</span>
  </div>
</div>

<style>
  .profile { position: relative; display: flex; height: 100%; }
  .ptoolbar { display: flex; flex-direction: column; gap: 4px; padding: var(--space-2); border-right: 1px solid var(--hair); }
  .ptog { height: 26px; padding: 0 8px; border: 1px solid var(--hair); border-radius: 5px; background: var(--panel-2); color: var(--txt-3); font: inherit; font-size: 11px; cursor: pointer; white-space: nowrap; }
  .ptog.on { color: var(--txt); border-color: var(--aqua); }
  svg { flex: 1; min-width: 0; }
  .legend { position: absolute; right: 24px; bottom: 12px; display: flex; gap: 12px; font-size: 10.5px; color: var(--txt-3); }
  .sw { display: inline-block; width: 10px; height: 10px; border-radius: 2px; margin-right: 4px; vertical-align: -1px; }
</style>
