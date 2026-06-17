// AI-generated (Claude)
import { describe, it, expect } from "vitest";
import { parseLogbook } from "$lib/ssrf-git/index.ts";
import path from "node:path";

const FIXTURE = path.resolve(__dirname, "../fixtures/git-tree");

describe("parseLogbook (golden)", () => {
  const lb = parseLogbook(FIXTURE);

  it("reads units and sites", () => {
    expect(lb.units).toBe("METRIC");
    expect(lb.sites).toHaveLength(1);
    expect(lb.sites[0]).toMatchObject({ id: "04782ed8", name: "Test Spring", gps: { lat: 47.668408, lon: 18.307076 } });
  });

  it("assembles a dive from its directory, Dive and Divecomputer files", () => {
    expect(lb.dives).toHaveLength(1);
    const d = lb.dives[0];
    expect(d.number).toBe(1);
    expect(d.dateTime).toBe("2024-03-15T12:28:43");
    expect(d.durationSec).toBe(300);
    expect(d.rating).toBe(4);
    expect(d.tags).toEqual(["cave"]);
    expect(d.siteId).toBe("04782ed8");
    expect(d.dcModel).toBe("Shearwater Perdix AI");
    expect(d.maxDepthM).toBe(34.7);
    expect(d.decoModel).toBe("GF 55/85");
    expect(d.cylinders[0].description).toBe("D12 232 bar");
    expect(d.samples).toHaveLength(5);
    expect(d.samples[2]).toMatchObject({ timeSec: 150, depthM: 20, tempC: 24, ndlSec: 55 * 60 });
    expect(d.events[0]).toMatchObject({ name: "gaschange", cylinder: 0 });
  });

  it("sorts dives by datetime", () => {
    const times = lb.dives.map((d) => d.dateTime);
    expect([...times].sort()).toEqual(times);
  });
});
