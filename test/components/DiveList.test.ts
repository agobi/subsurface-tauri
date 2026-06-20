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

function dive(overrides: Partial<Dive> = {}): Dive {
  return { number: 1, dateTime: "2024-01-01T00:00:00", durationSec: 300, tags: [], cylinders: [], samples: [], events: [], ...overrides };
}

describe("DiveList — sorting and columns", () => {
  beforeEach(() => app.reset());

  it("renders a sortable header button for each visible column", () => {
    render(DiveList, { props: { dives: [], trips: [], sites: [], query: "" } });
    expect(screen.getByRole("button", { name: /date/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /depth/i })).toBeInTheDocument();
  });

  it("clicking a column header updates sortKey in store", async () => {
    render(DiveList, { props: { dives: [], trips: [], sites: [], query: "" } });
    await fireEvent.click(screen.getByRole("button", { name: /depth/i }));
    expect(app.diveListPrefs.sortKey).toBe("depth");
    expect(app.diveListPrefs.sortDir).toBe("asc");
  });

  it("clicking the same header twice toggles sort direction", async () => {
    render(DiveList, { props: { dives: [], trips: [], sites: [], query: "" } });
    const depthBtn = () => screen.getByRole("button", { name: /depth/i });
    await fireEvent.click(depthBtn());
    await fireEvent.click(depthBtn());
    expect(app.diveListPrefs.sortDir).toBe("desc");
  });

  it("sorting by non-nr key renders flat list — trip header disappears", async () => {
    const tDives = [dive({ number: 1, siteId: "s1" }), dive({ number: 2, siteId: "s2" })];
    const tTrips = [{ label: "Trip A", diveNumbers: [1, 2] }];
    const tSites = [{ id: "s1", name: "Site One" }, { id: "s2", name: "Site Two" }];
    render(DiveList, { props: { dives: tDives, trips: tTrips, sites: tSites, query: "" } });
    expect(screen.getByText(/Trip A/)).toBeInTheDocument();
    await fireEvent.click(screen.getByRole("button", { name: /depth/i }));
    expect(screen.queryByText(/Trip A/)).not.toBeInTheDocument();
    expect(screen.getByText("Site One")).toBeInTheDocument();
    expect(screen.getByText("Site Two")).toBeInTheDocument();
  });

  it("sorting by depth asc shows shallower dive first", async () => {
    const tDives = [
      dive({ number: 1, maxDepthM: 30, siteId: "s1" }),
      dive({ number: 2, maxDepthM: 10, siteId: "s2" }),
    ];
    const tSites = [{ id: "s1", name: "Deep Site" }, { id: "s2", name: "Shallow Site" }];
    app.logbook = { dives: tDives, trips: [], sites: tSites, units: "METRIC" };
    const { container, rerender } = render(DiveList, { props: { dives: app.sortedDives, trips: [], sites: tSites, query: "" } });
    await fireEvent.click(screen.getByRole("button", { name: /depth/i }));
    await rerender({ dives: app.sortedDives });
    const rows = container.querySelectorAll("[data-testid='dive-row']");
    expect(rows[0].textContent).toContain("10.0");
    expect(rows[1].textContent).toContain("30.0");
  });

  it("opening ColumnPicker and unchecking Buddy hides the Buddy header", async () => {
    render(DiveList, { props: { dives: [], trips: [], sites: [], query: "" } });
    expect(screen.getByRole("button", { name: /buddy/i })).toBeInTheDocument();
    await fireEvent.click(screen.getByRole("button", { name: /column options/i }));
    await fireEvent.click(screen.getByRole("checkbox", { name: /buddy/i }));
    expect(screen.queryByRole("button", { name: /buddy/i })).not.toBeInTheDocument();
  });
});
