// AI-generated (Claude)
// Pure coordinate math for the dive profile, ported from subsurface-reskin.html.
// Kept isolated so DiveProfile can later move from SVG to <canvas> without touching this.

export interface PlotBox { x0: number; x1: number; y0: number; y1: number; }

export function timeToX(timeSec: number, maxTimeSec: number, plot: PlotBox): number {
  if (maxTimeSec <= 0) return plot.x0;
  return plot.x0 + (timeSec / maxTimeSec) * (plot.x1 - plot.x0);
}

export function depthToY(depthM: number, depthAxisMaxM: number, plot: PlotBox): number {
  if (depthAxisMaxM <= 0) return plot.y0;
  return plot.y0 + (depthM / depthAxisMaxM) * (plot.y1 - plot.y0);
}

// Round the depth axis up to a clean 5m line above max+4m headroom (POC behavior).
export function depthAxisMax(maxDepthM: number): number {
  return Math.ceil((maxDepthM + 4) / 5) * 5;
}

export type RateClass = "fast" | "ok" | "normal";

// Positive value = ascending (getting shallower). Thresholds in m/min.
// >= 9 m/min is the classic "too fast" ascent warning; gentle ascent is "ok"; everything else "normal".
export function ascentRate(prev: { timeSec: number; depthM: number }, cur: { timeSec: number; depthM: number }): number {
  const dtMin = (cur.timeSec - prev.timeSec) / 60 || 0.001;
  return (prev.depthM - cur.depthM) / dtMin;
}

export function ascentRateClass(prev: { timeSec: number; depthM: number }, cur: { timeSec: number; depthM: number }): RateClass {
  const rate = ascentRate(prev, cur);
  if (rate >= 9) return "fast";
  if (rate > 0) return "ok";
  return "normal";
}
