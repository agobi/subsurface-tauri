// AI-generated (Claude)
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/svelte";
import DiveProfile from "$lib/components/DiveProfile.svelte";
import type { Dive } from "$lib/types.ts";

const dive: Dive = {
  number: 1, dateTime: "2024-03-15T12:28:43", durationSec: 3300, tags: [], cylinders: [],
  maxDepthM: 34.7, samples: [
    { timeSec: 0, depthM: 0 }, { timeSec: 60, depthM: 9 }, { timeSec: 210, depthM: 30.4 },
    { timeSec: 480, depthM: 30 },
    { timeSec: 510, depthM: 24 }, // 6 m up in 30 s = 12 m/min -> fast ascent
    { timeSec: 3300, depthM: 0 },
  ], events: [],
};

describe("DiveProfile", () => {
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
});
