// AI-generated (Claude)
import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { render, screen } from "@testing-library/svelte";
import DiveProfile from "$lib/components/DiveProfile.svelte";
import { app } from "$lib/stores/app.svelte.ts";
import type { Dive } from "$lib/types.ts";

const dive: Dive = {
  number: 1, dateTime: "2024-03-15T12:28:43", durationSec: 3300, tags: [], cylinders: [], mediaCount: 0,
  maxDepthM: 34.7, samples: [
    { timeSec: 0, depthM: 0 }, { timeSec: 60, depthM: 9 }, { timeSec: 210, depthM: 30.4 },
    { timeSec: 480, depthM: 30 },
    { timeSec: 510, depthM: 24 }, // 6 m up in 30 s = 12 m/min -> fast ascent
    { timeSec: 3300, depthM: 0 },
  ], events: [],
};

const diveNoSamples: Dive = {
  number: 2, dateTime: "2024-03-16T10:00:00", durationSec: 1800, tags: [], cylinders: [], mediaCount: 0,
  maxDepthM: 18, samples: [], events: [],
};

const diveHighPressure: Dive = {
  number: 3, dateTime: "2024-03-17T09:00:00", durationSec: 60, tags: [], cylinders: [], mediaCount: 0,
  maxDepthM: 10, samples: [
    { timeSec: 0, depthM: 10, pressureBar: 300 },
    { timeSec: 60, depthM: 0, pressureBar: 280 },
  ], events: [],
};

describe("DiveProfile", () => {
  beforeEach(() => app.reset());

  it("renders without crash when samples is empty", () => {
    const { container } = render(DiveProfile, { props: { dive: diveNoSamples } });
    expect(container.querySelector("svg")).toBeInTheDocument();
  });

  it("renders an svg with a depth path", () => {
    const { container } = render(DiveProfile, { props: { dive } });
    expect(container.querySelector("svg")).toBeInTheDocument();
    expect(container.querySelectorAll("path,line,polyline").length).toBeGreaterThan(0);
  });

  it("renders curve toggle buttons", () => {
    render(DiveProfile, { props: { dive } });
    expect(screen.getByRole("button", { name: /temperature/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /pO.?2/i })).toBeInTheDocument();
  });

  it("marks fast-ascent segments with a non-color (dashed) cue", () => {
    const { container } = render(DiveProfile, { props: { dive } });
    // On render (no cursor) the only dashed line is the fast-ascent depth segment.
    const dashed = container.querySelector("line[stroke-dasharray]");
    expect(dashed).toBeInTheDocument();
    // The cue is both color (rate-fast red) and non-color (dashed) together.
    expect(dashed?.getAttribute("stroke")).toContain("rate-fast");
  });

  it("does not flat-line the tank-pressure trace above the old 250 bar cap", () => {
    const { container } = render(DiveProfile, { props: { dive: diveHighPressure } });
    const tankLine = container.querySelector("polyline[stroke='var(--amber)']");
    expect(tankLine).toBeInTheDocument();
    const points = tankLine!.getAttribute("points")!.trim().split(" ").map((p) => p.split(",").map(Number));
    // 300 bar and 280 bar would both clip to the same y (top) under the old fixed
    // /250 normalization; with dynamic rescaling they must land at different heights.
    expect(points[0][1]).not.toBeCloseTo(points[1][1]);
  });

  it("renders metric depth-axis gridline labels every 6m by default", () => {
    const { container } = render(DiveProfile, { props: { dive } });
    const labels = Array.from(container.querySelectorAll("text")).map(t => t.textContent);
    expect(labels).toContain("0m");
    expect(labels).toContain("6m");
    expect(labels).not.toContain("10ft");
  });
});

describe("DiveProfile — imperial units", () => {
  beforeEach(() => app.reset());
  afterEach(() => app.reset());

  it("renders depth-axis gridline labels every 10ft instead of 6m", () => {
    app.setUnitsPref("IMPERIAL");
    const { container } = render(DiveProfile, { props: { dive } });
    const labels = Array.from(container.querySelectorAll("text")).map(t => t.textContent);
    expect(labels).toContain("0ft");
    expect(labels).toContain("10ft");
    expect(labels).not.toContain("6m");
  });
});
