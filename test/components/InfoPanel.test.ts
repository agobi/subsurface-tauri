// AI-generated (Claude)
import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import InfoPanel from "$lib/components/InfoPanel.svelte";
import type { Dive } from "$lib/types.ts";

const dive: Dive = {
  number: 269, dateTime: "2024-03-15T12:28:43", durationSec: 3310, tags: ["cave"], rating: 4,
  diveGuide: "Attila Gobi", maxDepthM: 34.7, meanDepthM: 14.8, waterTempC: 19, decoModel: "GF 55/85",
  cylinders: [{ description: "D12 232 bar", volumeL: 24, workPressureBar: 232, o2Percent: 32 }],
  notes: "Great cave dive", mediaCount: 0, samples: [], events: [],
};

describe("InfoPanel", () => {
  it("defaults to the Notes tab and shows notes", () => {
    render(InfoPanel, { props: { dive } });
    expect(screen.getByText("Great cave dive")).toBeInTheDocument();
  });

  it("switches to Equipment and shows the cylinder", async () => {
    render(InfoPanel, { props: { dive } });
    await fireEvent.click(screen.getByRole("tab", { name: /equipment/i }));
    expect(screen.getByText("D12 232 bar")).toBeInTheDocument();
  });

  it("switches to Information and shows computed depth/temp", async () => {
    render(InfoPanel, { props: { dive } });
    await fireEvent.click(screen.getByRole("tab", { name: /information/i }));
    expect(screen.getByText(/34.7/)).toBeInTheDocument();
  });
});
