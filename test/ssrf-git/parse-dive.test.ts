// AI-generated (Claude)
import { describe, it, expect } from "vitest";
import { parseDive } from "$lib/ssrf-git/parse-dive.ts";

const text = [
  "duration 55:10 min",
  "rating 4",
  "notrip",
  'tags "cave", "deep"',
  "divesiteid c171f112",
  'divemaster "Attila Gobi"',
  'suit "drysuit"',
  'notes "Great cave dive"',
  'cylinder vol=24.0l workpressure=232.0bar description="D12 232 bar" o2=32.0% depth=39.845m',
].join("\n");

describe("parseDive", () => {
  it("parses overview fields and cylinders", () => {
    const d = parseDive(text);
    expect(d.durationSec).toBe(55 * 60 + 10);
    expect(d.rating).toBe(4);
    expect(d.tags).toEqual(["cave", "deep"]);
    expect(d.siteId).toBe("c171f112");
    expect(d.diveGuide).toBe("Attila Gobi");
    expect(d.suit).toBe("drysuit");
    expect(d.notes).toBe("Great cave dive");
    expect(d.cylinders).toHaveLength(1);
    expect(d.cylinders[0]).toEqual({
      description: "D12 232 bar", volumeL: 24, workPressureBar: 232, o2Percent: 32, depthM: 39.845,
    });
  });

  it("ignores unknown lines instead of failing", () => {
    const d = parseDive("duration 1:00 min\nsomethingnew foo bar\n");
    expect(d.durationSec).toBe(60);
  });
});
