// AI-generated (Claude)
import { describe, it, expect } from "vitest";
import { timeToX, depthToY, depthAxisMax, ascentRateClass } from "$lib/profile/profile-scale.ts";

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

  it("rounds the depth axis up to the next 5m above max+4", () => {
    expect(depthAxisMax(34.7)).toBe(40);
    expect(depthAxisMax(18.2)).toBe(25);
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
});
