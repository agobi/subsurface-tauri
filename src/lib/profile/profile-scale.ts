// AI-generated (Claude)
// Pure coordinate math for the dive profile, ported from subsurface-reskin.html.
// Kept isolated so DiveProfile can later move from SVG to <canvas> without touching this.
import type { Units } from "$lib/types.ts";

export interface PlotBox { x0: number; x1: number; y0: number; y1: number; }

export function timeToX(timeSec: number, maxTimeSec: number, plot: PlotBox): number {
  if (maxTimeSec <= 0) return plot.x0;
  return plot.x0 + (timeSec / maxTimeSec) * (plot.x1 - plot.x0);
}

export function depthToY(depthM: number, depthAxisMaxM: number, plot: PlotBox): number {
  if (depthAxisMaxM <= 0) return plot.y0;
  return plot.y0 + (depthM / depthAxisMaxM) * (plot.y1 - plot.y0);
}

// Round the depth axis up to a clean 6m line above max+4m headroom — a more
// realistic dive-instrument increment than the previous 5m step, and a
// multiple of three matching Subsurface's own "nice" gridline set.
export function depthAxisMax(maxDepthM: number): number {
  return Math.ceil((maxDepthM + 4) / 6) * 6;
}

// Rescales to the dive's actual max tank pressure (e.g. 300+ bar for steel/HP
// cylinders) instead of clipping at a fixed cap; 250 bar remains the floor so
// ordinary AL80-range dives keep their existing chart proportions.
export function tankPressureAxisMax(maxPressureBar: number): number {
  return Math.max(250, Math.ceil((maxPressureBar + 20) / 50) * 50);
}

// Rescales to the dive's actual max water temp instead of clipping at a fixed
// cap; 40°C remains the floor so ordinary temperate/tropical dives keep their
// existing chart proportions.
export function tempAxisMax(maxTempC: number): number {
  return Math.max(40, Math.ceil((maxTempC + 2) / 5) * 5);
}

const FT_PER_M = 3.28084;

// Tick positions for the depth axis, always computed in metres (matching the
// always-metric sample data) but spaced and labelled in the currently
// displayed unit: every 6m for metric, every 10ft (converted to metres for
// positioning) for imperial.
export function depthGridLines(axisMaxM: number, units: Units): { m: number; label: number }[] {
  const out: { m: number; label: number }[] = [];
  const EPS = 1e-9;
  if (units === "IMPERIAL") {
    for (let ft = 0; ft / FT_PER_M <= axisMaxM + EPS; ft += 10) {
      out.push({ m: ft / FT_PER_M, label: Math.round(ft) });
    }
  } else {
    for (let m = 0; m <= axisMaxM + EPS; m += 6) {
      out.push({ m, label: m });
    }
  }
  return out;
}

export type RateClass = "fast" | "ok" | "normal";

// Positive value = ascending (getting shallower). Thresholds in m/min.
// >= 9 m/min is the classic "too fast" ascent warning; gentle ascent is "ok"; everything else "normal".
export function ascentRate(prev: { timeSec: number; depthM: number }, cur: { timeSec: number; depthM: number }): number {
  const dtSec = cur.timeSec - prev.timeSec;
  if (dtSec <= 0) return 0;
  return (prev.depthM - cur.depthM) / (dtSec / 60);
}

export function ascentRateClass(prev: { timeSec: number; depthM: number }, cur: { timeSec: number; depthM: number }): RateClass {
  const rate = ascentRate(prev, cur);
  if (rate >= 9) return "fast";
  if (rate > 0) return "ok";
  return "normal";
}
