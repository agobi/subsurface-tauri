// AI-generated (Claude)
import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import DiveList from "$lib/components/DiveList.svelte";
import { app } from "$lib/stores/app.svelte.ts";
import type { Dive, Trip, Site } from "$lib/types.ts";

const dives: Dive[] = [
  { number: 269, dateTime: "2024-03-15T12:28:43", durationSec: 3310, siteId: "c171f112", tags: ["cave"], rating: 4, maxDepthM: 34.7, cylinders: [], samples: [], events: [] },
  { number: 270, dateTime: "2024-04-01T10:00:00", durationSec: 2400, siteId: "04782ed8", tags: ["reef"], rating: 5, maxDepthM: 18.2, cylinders: [], samples: [], events: [] },
];
const trips: Trip[] = [{ label: "March 2024", diveNumbers: [269] }];
const sites: Site[] = [
  { id: "c171f112", name: "Molnar Janos" },
  { id: "04782ed8", name: "Fenyes Forras" },
];

describe("DiveList", () => {
  beforeEach(() => app.reset());

  it("renders a row per dive with star ratings", () => {
    render(DiveList, { props: { dives, trips, sites, query: "" } });
    expect(screen.getByText("Molnar Janos")).toBeInTheDocument();
    expect(screen.getByText("Fenyes Forras")).toBeInTheDocument();
    expect(screen.getAllByLabelText(/rating/i).length).toBe(2);
  });

  it("filters by query (case-insensitive, over location and tags)", () => {
    render(DiveList, { props: { dives, trips, sites, query: "reef" } });
    expect(screen.getByText("Fenyes Forras")).toBeInTheDocument();
    expect(screen.queryByText("Molnar Janos")).not.toBeInTheDocument();
  });

  it("clicking a row selects the dive in the store", async () => {
    render(DiveList, { props: { dives, trips, sites, query: "" } });
    await fireEvent.click(screen.getByText("Molnar Janos"));
    expect(app.selectedDiveId).toBe(269);
  });

  it("renders a collapsible trip header and hides its dives when collapsed", async () => {
    render(DiveList, { props: { dives, trips, sites, query: "" } });
    expect(screen.getByText(/March 2024/)).toBeInTheDocument();
    expect(screen.getByText("Molnar Janos")).toBeInTheDocument();
    await fireEvent.click(screen.getByText(/March 2024/));
    expect(screen.queryByText("Molnar Janos")).not.toBeInTheDocument(); // collapsed hides the trip's dive
    expect(screen.getByText("Fenyes Forras")).toBeInTheDocument(); // ungrouped dive still shown
  });
});
