// AI-generated (Claude)
import { describe, it, expect } from "vitest";
import { ALL_COLS, DEFAULT_COL_ORDER } from "$lib/diveListColumns.ts";
import type { Dive, Site } from "$lib/types.ts";

const noSites: Site[] = [];
const sites: Site[] = [{ id: "abc", name: "Test Spring", country: "Hungary" }];

function makeDive(overrides: Partial<Dive> = {}): Dive {
  return { number: 1, dateTime: "2024-03-15T12:28:43", durationSec: 3310, tags: [], cylinders: [], mediaCount: 0, samples: [], events: [], ...overrides };
}

describe("DEFAULT_COL_ORDER", () => {
  it("contains 20 entries", () => expect(DEFAULT_COL_ORDER.length).toBe(20));
  it("starts with nr and date", () => {
    expect(DEFAULT_COL_ORDER[0]).toBe("nr");
    expect(DEFAULT_COL_ORDER[1]).toBe("date");
  });
  it("contains all ColIds from ALL_COLS", () => {
    const allIds = ALL_COLS.map(c => c.id);
    expect(DEFAULT_COL_ORDER).toEqual(expect.arrayContaining(allIds));
    expect(DEFAULT_COL_ORDER.length).toBe(allIds.length);
  });
});

describe("ALL_COLS", () => {
  it("contains 20 entries", () => expect(ALL_COLS.length).toBe(20));
  it("defaultVisible cols appear before non-defaultVisible cols in DEFAULT_COL_ORDER", () => {
    const orders = DEFAULT_COL_ORDER.map(id => ALL_COLS.find(c => c.id === id)!.defaultVisible);
    const firstHidden = orders.indexOf(false);
    const lastVisible = orders.lastIndexOf(true);
    expect(firstHidden).toBeGreaterThan(lastVisible);
  });
});

describe("render", () => {
  const col = (id: string) => ALL_COLS.find(c => c.id === id)!;

  it("nr renders dive number as string", () => {
    expect(col("nr").render(makeDive({ number: 42 }), { sites: noSites })).toBe("42");
  });
  it("date renders ISO date portion only", () => {
    expect(col("date").render(makeDive(), { sites: noSites })).toBe("2024-03-15");
  });
  it("rating renders star string for rating 3", () => {
    expect(col("rating").render(makeDive({ rating: 3 }), { sites: noSites })).toBe("★★★☆☆");
  });
  it("rating renders em-dash when missing", () => {
    expect(col("rating").render(makeDive(), { sites: noSites })).toBe("—");
  });
  it("depth renders toFixed(1) when present", () => {
    expect(col("depth").render(makeDive({ maxDepthM: 34.7 }), { sites: noSites })).toBe("34.7");
  });
  it("depth renders em-dash when missing", () => {
    expect(col("depth").render(makeDive(), { sites: noSites })).toBe("—");
  });
  it("duration renders M:SS", () => {
    expect(col("duration").render(makeDive({ durationSec: 3310 }), { sites: noSites })).toBe("55:10");
  });
  it("buddy renders em-dash when absent", () => {
    expect(col("buddy").render(makeDive(), { sites: noSites })).toBe("—");
  });
  it("location resolves site name", () => {
    expect(col("location").render(makeDive({ siteId: "abc" }), { sites })).toBe("Test Spring");
  });
  it("country resolves from site", () => {
    expect(col("country").render(makeDive({ siteId: "abc" }), { sites })).toBe("Hungary");
  });
  it("country renders em-dash when site has no country", () => {
    const noCountry: Site[] = [{ id: "abc", name: "Test Spring" }];
    expect(col("country").render(makeDive({ siteId: "abc" }), { sites: noCountry })).toBe("—");
  });
  it("tags joins with comma", () => {
    expect(col("tags").render(makeDive({ tags: ["cave", "night"] }), { sites: noSites })).toBe("cave, night");
  });
  it("tags renders em-dash when empty", () => {
    expect(col("tags").render(makeDive({ tags: [] }), { sites: noSites })).toBe("—");
  });
  it("notes truncates at 60 chars with ellipsis", () => {
    const long = "a".repeat(65);
    const result = col("notes").render(makeDive({ notes: long }), { sites: noSites });
    expect(result).toHaveLength(61);
    expect(result.endsWith("…")).toBe(true);
  });
  it("notes shows full text under 60 chars", () => {
    expect(col("notes").render(makeDive({ notes: "short" }), { sites: noSites })).toBe("short");
  });
  it("divemode renders string directly", () => {
    expect(col("divemode").render(makeDive({ divemode: "CCR" }), { sites: noSites })).toBe("CCR");
  });
  it("weight renders toFixed(2)", () => {
    expect(col("weight").render(makeDive({ totalWeightKg: 3.5 }), { sites: noSites })).toBe("3.50");
  });
  it("media renders count as string", () => {
    expect(col("media").render(makeDive({ mediaCount: 3 }), { sites: noSites })).toBe("3");
  });
  it("media renders em-dash when zero", () => {
    expect(col("media").render(makeDive({ mediaCount: 0 }), { sites: noSites })).toBe("—");
  });
  it("otu renders integer value", () => {
    expect(col("otu").render(makeDive({ otu: 42 }), { sites: noSites })).toBe("42");
  });
  it("otu renders em-dash when missing", () => {
    expect(col("otu").render(makeDive(), { sites: noSites })).toBe("—");
  });
  it("maxcns renders rounded percent", () => {
    expect(col("maxcns").render(makeDive({ maxCns: 19.6 }), { sites: noSites })).toBe("20%");
  });
  it("maxcns renders em-dash when missing", () => {
    expect(col("maxcns").render(makeDive(), { sites: noSites })).toBe("—");
  });
});

describe("gas mix (cylinder column)", () => {
  const col = ALL_COLS.find(c => c.id === "cylinder")!;
  const ctx = { sites: noSites };

  it("Air when o2 ≈ 21% with no he", () => {
    expect(col.render(makeDive({ cylinders: [{ description: "AL80", o2Percent: 21 }] }), ctx)).toBe("Air");
  });
  it("EAN32 for 32% o2 no he", () => {
    expect(col.render(makeDive({ cylinders: [{ description: "AL80", o2Percent: 32 }] }), ctx)).toBe("EAN32");
  });
  it("Trimix as o2/he percentages", () => {
    expect(col.render(makeDive({ cylinders: [{ description: "12L", o2Percent: 21, hePercent: 35 }] }), ctx)).toBe("21/35");
  });
  it("em-dash when no cylinders", () => {
    expect(col.render(makeDive({ cylinders: [] }), ctx)).toBe("—");
  });
  it("Air when o2 absent", () => {
    expect(col.render(makeDive({ cylinders: [{ description: "AL80" }] }), ctx)).toBe("Air");
  });
});

describe("SAC column", () => {
  const col = ALL_COLS.find(c => c.id === "sac")!;
  const ctx = { sites: noSites };

  it("computes SAC when all fields present", () => {
    const d = makeDive({ durationSec: 3310, meanDepthM: 15, cylinders: [{ description: "12L", volumeL: 12, startBar: 200, endBar: 50 }] });
    const result = parseFloat(col.render(d, ctx));
    expect(result).toBeGreaterThan(10);
    expect(result).toBeLessThan(20);
  });
  it("em-dash when startBar missing", () => {
    const d = makeDive({ durationSec: 3310, meanDepthM: 15, cylinders: [{ description: "12L", volumeL: 12, endBar: 50 }] });
    expect(col.render(d, ctx)).toBe("—");
  });
  it("em-dash when no cylinders", () => {
    expect(col.render(makeDive({ cylinders: [], meanDepthM: 15 }), ctx)).toBe("—");
  });
});

describe("compare", () => {
  const col = (id: string) => ALL_COLS.find(c => c.id === id)!;
  const ctx = { sites: noSites };

  it("nr: ascending by number", () => {
    expect(col("nr").compare(makeDive({ number: 5 }), makeDive({ number: 3 }), ctx)).toBeGreaterThan(0);
    expect(col("nr").compare(makeDive({ number: 3 }), makeDive({ number: 5 }), ctx)).toBeLessThan(0);
  });
  it("depth: ascending by maxDepthM", () => {
    expect(col("depth").compare(makeDive({ maxDepthM: 10 }), makeDive({ maxDepthM: 30 }), ctx)).toBeLessThan(0);
  });
  it("media: ascending by mediaCount", () => {
    expect(col("media").compare(makeDive({ mediaCount: 1 }), makeDive({ mediaCount: 5 }), ctx)).toBeLessThan(0);
  });
  it("date: lexicographic dateTime", () => {
    expect(col("date").compare(makeDive({ dateTime: "2024-01-01T00:00:00" }), makeDive({ dateTime: "2024-06-01T00:00:00" }), ctx)).toBeLessThan(0);
  });
  it("otu: ascending by otu", () => {
    expect(col("otu").compare(makeDive({ otu: 10 }), makeDive({ otu: 30 }), ctx)).toBeLessThan(0);
  });
  it("maxcns: ascending by maxCns", () => {
    expect(col("maxcns").compare(makeDive({ maxCns: 10 }), makeDive({ maxCns: 30 }), ctx)).toBeLessThan(0);
  });
});
