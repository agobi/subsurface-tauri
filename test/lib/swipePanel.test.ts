// AI-generated (Claude)
// test/lib/swipePanel.test.ts
import { describe, it, expect } from "vitest";
import { computeActiveIndex } from "$lib/swipePanel.ts";

describe("computeActiveIndex", () => {
  it("returns 0 when scrolled to the start", () => {
    expect(computeActiveIndex(0, 300, 3)).toBe(0);
  });

  it("returns 1 when scrolled exactly one panel width", () => {
    expect(computeActiveIndex(300, 300, 3)).toBe(1);
  });

  it("rounds to the nearest panel during a partial scroll", () => {
    expect(computeActiveIndex(460, 300, 3)).toBe(2);
  });

  it("clamps to the last panel index even if scrollLeft overshoots", () => {
    expect(computeActiveIndex(900, 300, 3)).toBe(2);
  });

  it("returns 0 when containerWidth is 0 (not yet laid out)", () => {
    expect(computeActiveIndex(150, 0, 3)).toBe(0);
  });
});
