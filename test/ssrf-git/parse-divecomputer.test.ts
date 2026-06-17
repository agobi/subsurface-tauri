// AI-generated (Claude)
import { describe, it, expect } from "vitest";
import { parseDivecomputer } from "$lib/ssrf-git/parse-divecomputer.ts";

const text = [
  'model "Shearwater Perdix AI"',
  "deviceid a790cf6c",
  "diveid 76b9bc25",
  "maxdepth 34.7m",
  "meandepth 14.793m",
  "watertemp 19.0°C",
  "salinity 1020g/l",
  'keyvalue "Deco model" "GF 55/85"',
  'event 0:05 type=25 flags=1 name="gaschange" cylinder=0 o2=32.0%',
  "  0:05 1.6m 25.0°C cns=5%",
  "  0:10 1.9m",
  "  0:15 2.5m ndl=99:00",
  "  4:55 15.0m 24.0°C",
].join("\n");

describe("parseDivecomputer", () => {
  it("parses dc meta", () => {
    const dc = parseDivecomputer(text);
    expect(dc.model).toBe("Shearwater Perdix AI");
    expect(dc.maxDepthM).toBe(34.7);
    expect(dc.meanDepthM).toBe(14.793);
    expect(dc.waterTempC).toBe(19);
    expect(dc.decoModel).toBe("GF 55/85");
  });

  it("parses the gaschange event", () => {
    const dc = parseDivecomputer(text);
    expect(dc.events[0]).toMatchObject({ timeSec: 5, name: "gaschange", cylinder: 0, o2Percent: 32 });
  });

  it("parses samples and carries temp/ndl forward", () => {
    const dc = parseDivecomputer(text);
    expect(dc.samples[0]).toMatchObject({ timeSec: 5, depthM: 1.6, tempC: 25, cns: 5 });
    expect(dc.samples[1]).toMatchObject({ timeSec: 10, depthM: 1.9, tempC: 25 });
    expect(dc.samples[2]).toMatchObject({ timeSec: 15, depthM: 2.5, tempC: 25, ndlSec: 99 * 60 });
    expect(dc.samples[3]).toMatchObject({ timeSec: 295, depthM: 15, tempC: 24, ndlSec: 99 * 60 });
  });
});
