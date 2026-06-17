// AI-generated (Claude)
import { describe, it, expect } from "vitest";
import { splitKeyword, parseAttrs, unquote, stripUnit } from "$lib/ssrf-git/tokenize.ts";

describe("tokenize", () => {
  it("splits the leading keyword from the rest", () => {
    expect(splitKeyword('divemaster "Attila Gobi"')).toEqual({ key: "divemaster", rest: '"Attila Gobi"' });
    expect(splitKeyword("maxdepth 34.7m")).toEqual({ key: "maxdepth", rest: "34.7m" });
  });

  it("unquotes a quoted value", () => {
    expect(unquote('"D12 232 bar"')).toBe("D12 232 bar");
    expect(unquote("plain")).toBe("plain");
  });

  it("strips a trailing unit, returning the number", () => {
    expect(stripUnit("34.7m")).toBe(34.7);
    expect(stripUnit("232.0bar")).toBe(232);
    expect(stripUnit("24.0l")).toBe(24);
    expect(stripUnit("32.0%")).toBe(32);
  });

  it("parses an attribute list with quotes and units", () => {
    expect(parseAttrs('vol=24.0l workpressure=232.0bar description="D12 232 bar" o2=32.0% depth=39.845m'))
      .toEqual({ vol: "24.0l", workpressure: "232.0bar", description: "D12 232 bar", o2: "32.0%", depth: "39.845m" });
  });
});
