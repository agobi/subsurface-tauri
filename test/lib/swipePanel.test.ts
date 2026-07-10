// AI-generated (Claude)
// test/lib/swipePanel.test.ts
import { describe, it, expect } from "vitest";
import { computeSnapTarget } from "$lib/swipePanel.ts";

describe("computeSnapTarget", () => {
  it("stays on the start panel when the scroll barely moves", () => {
    expect(computeSnapTarget(320, 1, 300, 3, 0.45)).toBe(1);
  });

  it("advances to the next panel once the threshold is crossed forward", () => {
    expect(computeSnapTarget(451, 1, 300, 3, 0.45)).toBe(2);
  });

  it("snaps back to the start panel when short of the threshold", () => {
    expect(computeSnapTarget(430, 1, 300, 3, 0.45)).toBe(1);
  });

  it("goes to the previous panel once the threshold is crossed backward", () => {
    expect(computeSnapTarget(149, 1, 300, 3, 0.45)).toBe(0);
  });

  it("clamps at the last panel even if scrollLeft overshoots", () => {
    expect(computeSnapTarget(900, 2, 300, 3, 0.45)).toBe(2);
  });

  it("clamps at the first panel even if scrollLeft undershoots", () => {
    expect(computeSnapTarget(-900, 0, 300, 3, 0.45)).toBe(0);
  });

  it("returns the clamped start index when containerWidth is 0 (not yet laid out)", () => {
    expect(computeSnapTarget(150, 1, 0, 3, 0.45)).toBe(1);
  });
});
