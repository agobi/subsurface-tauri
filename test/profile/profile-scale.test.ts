// AI-generated (Claude)
import { describe, it, expect } from "vitest";
import { timeToX, depthToY, depthAxisMax, depthGridLines, ascentRateClass, tankPressureAxisMax, tempAxisMax } from "$lib/profile/profile-scale.ts";

const plot = { x0: 44, x1: 982, y0: 24, y1: 346 };

describe("profile-scale", () => {
  it("maps t=0 to x0 and t=max to x1", () => {
    expect(timeToX(0, 3300, plot)).toBe(44);
    expect(timeToX(3300, 3300, plot)).toBe(982);
  });

  it("maps depth 0 to y0 (top) and axisMax to y1 (bottom)", () => {
    expect(depthToY(0, 40, plot)).toBe(24);
    expect(depthToY(40, 40, plot)).toBe(346);
  });

  it("rounds the depth axis up to the next 6m above max+4 (dive-instrument increment)", () => {
    expect(depthAxisMax(34.7)).toBe(42);
    expect(depthAxisMax(18.2)).toBe(24);
  });

  it("classifies ascent rate (positive depth delta over time = ascending)", () => {
    expect(ascentRateClass({ timeSec: 0, depthM: 30 }, { timeSec: 15, depthM: 25 })).toBe("fast");
    expect(ascentRateClass({ timeSec: 0, depthM: 10 }, { timeSec: 60, depthM: 12 })).toBe("normal");
    expect(ascentRateClass({ timeSec: 0, depthM: 10 }, { timeSec: 60, depthM: 7 })).toBe("ok");
  });

  it("returns normal (not fast) for same-timestamp consecutive samples", () => {
    // Two samples at t=0 is common at dive start; must not produce an inflated rate.
    expect(ascentRateClass({ timeSec: 0, depthM: 0 }, { timeSec: 0, depthM: 1 })).toBe("normal");
    expect(ascentRateClass({ timeSec: 10, depthM: 5 }, { timeSec: 10, depthM: 4 })).toBe("normal");
  });

  it("tankPressureAxisMax rescales past the 250 bar floor for HP/steel cylinders", () => {
    expect(tankPressureAxisMax(300)).toBe(350);
    expect(tankPressureAxisMax(180)).toBe(250); // stays at the floor for ordinary AL80-range dives
  });

  it("tempAxisMax rescales past the 40°C floor for unusually warm water", () => {
    expect(tempAxisMax(45)).toBe(50);
    expect(tempAxisMax(28)).toBe(40); // stays at the floor for ordinary dives
  });
});

describe("depthGridLines", () => {
  it("metric: one tick every 6m, label equals the metre value", () => {
    const ticks = depthGridLines(24, "METRIC");
    expect(ticks).toEqual([
      { m: 0, label: 0 },
      { m: 6, label: 6 },
      { m: 12, label: 12 },
      { m: 18, label: 18 },
      { m: 24, label: 24 },
    ]);
  });

  it("imperial: one tick every 10ft, positioned by its metre equivalent", () => {
    const ticks = depthGridLines(24, "IMPERIAL");
    expect(ticks.map(t => t.label)).toEqual([0, 10, 20, 30, 40, 50, 60, 70]);
    expect(ticks[1].m).toBeCloseTo(10 / 3.28084, 5); // ~3.048 m
    expect(ticks[7].m).toBeCloseTo(70 / 3.28084, 5); // ~21.335 m, still <= 24
  });

  it("imperial: does not emit a tick beyond axisMaxM", () => {
    const ticks = depthGridLines(24, "IMPERIAL");
    for (const t of ticks) expect(t.m).toBeLessThanOrEqual(24);
  });
});
