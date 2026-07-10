// AI-generated (Claude)
import { describe, it, expect } from "vitest";
import { fmtDepth, fmtPressure, fmtTemp, fmtVolume, fmtWeight, fmtCylinderSize } from "$lib/units.ts";

describe("fmtDepth", () => {
  it("metric: one decimal place with 'm' suffix", () => {
    expect(fmtDepth(34.7, "METRIC")).toBe("34.7 m");
  });
  it("metric: suffix:false omits the unit", () => {
    expect(fmtDepth(34.7, "METRIC", { suffix: false })).toBe("34.7");
  });
  it("imperial: converts to rounded feet with 'ft' suffix", () => {
    expect(fmtDepth(34.7, "IMPERIAL")).toBe("114 ft");
  });
  it("imperial: suffix:false omits the unit", () => {
    expect(fmtDepth(34.7, "IMPERIAL", { suffix: false })).toBe("114");
  });
  it("imperial: rounds to the nearest foot", () => {
    expect(fmtDepth(1, "IMPERIAL")).toBe("3 ft");   // 3.28084 -> 3
    expect(fmtDepth(1.5, "IMPERIAL")).toBe("5 ft"); // 4.92126 -> 5
  });
});

describe("fmtPressure", () => {
  it("metric: raw passthrough with 'bar' suffix", () => {
    expect(fmtPressure(232, "METRIC")).toBe("232 bar");
  });
  it("metric: suffix:false omits the unit", () => {
    expect(fmtPressure(232, "METRIC", { suffix: false })).toBe("232");
  });
  it("imperial: converts to rounded psi with 'psi' suffix", () => {
    expect(fmtPressure(232, "IMPERIAL")).toBe("3365 psi");
  });
  it("imperial: suffix:false omits the unit", () => {
    expect(fmtPressure(232, "IMPERIAL", { suffix: false })).toBe("3365");
  });
});

describe("fmtTemp", () => {
  it("metric: raw passthrough with '°C' suffix", () => {
    expect(fmtTemp(19, "METRIC")).toBe("19 °C");
  });
  it("metric: suffix:false omits the unit", () => {
    expect(fmtTemp(19, "METRIC", { suffix: false })).toBe("19");
  });
  it("imperial: converts to rounded fahrenheit with '°F' suffix", () => {
    expect(fmtTemp(19, "IMPERIAL")).toBe("66 °F");
  });
  it("imperial: suffix:false omits the unit", () => {
    expect(fmtTemp(19, "IMPERIAL", { suffix: false })).toBe("66");
  });
  it("imperial: handles freezing point exactly", () => {
    expect(fmtTemp(0, "IMPERIAL")).toBe("32 °F");
  });
});

describe("fmtVolume", () => {
  it("metric: raw passthrough with 'L' suffix", () => {
    expect(fmtVolume(24, "METRIC")).toBe("24 L");
  });
  it("metric: suffix:false omits the unit", () => {
    expect(fmtVolume(24, "METRIC", { suffix: false })).toBe("24");
  });
  it("imperial: converts to cuft with two decimal places and 'cuft' suffix", () => {
    expect(fmtVolume(24, "IMPERIAL")).toBe("0.85 cuft");
  });
  it("imperial: suffix:false omits the unit", () => {
    expect(fmtVolume(24, "IMPERIAL", { suffix: false })).toBe("0.85");
  });
});

describe("fmtCylinderSize", () => {
  it("metric: raw litres regardless of working pressure", () => {
    expect(fmtCylinderSize(24, 232, "METRIC")).toBe("24 L");
    expect(fmtCylinderSize(24, undefined, "METRIC")).toBe("24 L");
  });
  it("imperial: converts to cuft as physical-volume-times-working-pressure-in-atm (not a plain L->cuft conversion)", () => {
    // (24 L / 28.3168466) * (232 bar / 1.01325) ~= 194 cuft — matches Qt Subsurface's
    // get_cylinder_string(), which rates a cylinder's cuft by its free-gas capacity at
    // working pressure, not the physical tank volume alone.
    expect(fmtCylinderSize(24, 232, "IMPERIAL")).toBe("194 cuft");
  });
  it("imperial: suffix:false omits the unit", () => {
    expect(fmtCylinderSize(24, 232, "IMPERIAL", { suffix: false })).toBe("194");
  });
  it("imperial: falls back to litres when working pressure is unknown", () => {
    // Matches Qt Subsurface: without a working pressure we cannot derive a
    // meaningful cuft rating, so the cylinder size is shown in litres even
    // when the display units are IMPERIAL.
    expect(fmtCylinderSize(24, undefined, "IMPERIAL")).toBe("24 L");
  });
  it("imperial: uses 1 decimal place for values between 2 and 20 cuft", () => {
    expect(fmtCylinderSize(0.5, 200, "IMPERIAL")).toBe("3.5 cuft");
  });
  it("imperial: uses 2 decimal places for values at or below 2 cuft", () => {
    expect(fmtCylinderSize(0.2, 200, "IMPERIAL")).toBe("1.39 cuft");
  });
});

describe("fmtWeight", () => {
  it("metric: two decimal places with 'kg' suffix", () => {
    expect(fmtWeight(3.5, "METRIC")).toBe("3.50 kg");
  });
  it("metric: suffix:false omits the unit", () => {
    expect(fmtWeight(3.5, "METRIC", { suffix: false })).toBe("3.50");
  });
  it("imperial: converts to rounded pounds with 'lbs' suffix", () => {
    expect(fmtWeight(3.5, "IMPERIAL")).toBe("8 lbs");
  });
  it("imperial: suffix:false omits the unit", () => {
    expect(fmtWeight(3.5, "IMPERIAL", { suffix: false })).toBe("8");
  });
});
