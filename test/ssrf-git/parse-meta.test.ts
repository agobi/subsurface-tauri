// AI-generated (Claude)
import { describe, it, expect } from "vitest";
import { parseHeader } from "$lib/ssrf-git/parse-header.ts";
import { parseSite } from "$lib/ssrf-git/parse-site.ts";
import { parseTrip } from "$lib/ssrf-git/parse-trip.ts";

describe("parse meta files", () => {
  it("reads units from 00-Subsurface", () => {
    expect(parseHeader("version 3\nautogroup\nunits METRIC\nprefs DCCEILING\n").units).toBe("METRIC");
    expect(parseHeader("units IMPERIAL\n").units).toBe("IMPERIAL");
    expect(parseHeader("version 3\n").units).toBe("METRIC");
  });

  it("parses a divesite file", () => {
    const text = 'name "Fenyes Forras"\ndescription ""\nnotes ""\ngps 47.668408 18.307076\ngeo cat 2 origin 2 "Hungary"\n';
    const site = parseSite("Site-04782ed8", text);
    expect(site).toEqual({
      id: "04782ed8", name: "Fenyes Forras", description: "", notes: "",
      gps: { lat: 47.668408, lon: 18.307076 },
    });
  });

  it("parses a trip file", () => {
    const text = 'date 2024-03-15\ntime 12:28:43\nlocation "Tapolca"\nnotes "cave week"\n';
    const trip = parseTrip(text, [269, 270]);
    expect(trip.label).toBe("Tapolca");
    expect(trip.notes).toBe("cave week");
    expect(trip.diveNumbers).toEqual([269, 270]);
  });
});
