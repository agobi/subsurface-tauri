// AI-generated (Claude)
import { describe, it, expect, beforeEach } from "vitest";
import { app } from "$lib/stores/app.svelte.ts";
import type { Dive } from "$lib/types.ts";

describe("app store", () => {
  beforeEach(() => app.reset());

  it("starts with all four panels visible and auto theme", () => {
    expect(app.visiblePanels).toEqual({ info: true, profile: true, list: true, map: true });
    expect(app.theme).toBe("auto");
  });

  it("toggles a panel", () => {
    app.togglePanel("map");
    expect(app.visiblePanels.map).toBe(false);
  });

  it("never hides the last visible panel", () => {
    app.togglePanel("info");
    app.togglePanel("profile");
    app.togglePanel("list");
    app.togglePanel("map");
    expect(app.visiblePanels.map).toBe(true);
  });

  it("sets theme to light", () => {
    app.setTheme("light");
    expect(app.theme).toBe("light");
  });

  it("sets theme to auto", () => {
    app.setTheme("auto");
    expect(app.theme).toBe("auto");
  });

  describe("platform", () => {
    it("defaults to desktop", () => {
      expect(app.platform).toBe("desktop");
      expect(app.isMobile).toBe(false);
    });

    it("setPlatform('mobile') sets isMobile to true", () => {
      app.setPlatform("mobile");
      expect(app.platform).toBe("mobile");
      expect(app.isMobile).toBe(true);
    });

    it("reset() restores platform to desktop", () => {
      app.setPlatform("mobile");
      app.reset();
      expect(app.platform).toBe("desktop");
    });
  });
});

function makeDive(overrides: Partial<Dive> = {}): Dive {
  return { number: 1, dateTime: "2024-01-01T00:00:00", durationSec: 300, tags: [], cylinders: [], samples: [], events: [], ...overrides };
}

describe("diveListPrefs", () => {
  beforeEach(() => app.reset());

  it("starts with sortKey nr and sortDir asc", () => {
    expect(app.diveListPrefs.sortKey).toBe("nr");
    expect(app.diveListPrefs.sortDir).toBe("asc");
  });

  it("setSortCol changes key and defaults to asc", () => {
    app.setSortCol("depth");
    expect(app.diveListPrefs.sortKey).toBe("depth");
    expect(app.diveListPrefs.sortDir).toBe("asc");
  });

  it("setSortCol toggles sortDir when same key clicked twice", () => {
    app.setSortCol("depth");
    app.setSortCol("depth");
    expect(app.diveListPrefs.sortDir).toBe("desc");
  });

  it("setSortCol resets sortDir to asc when switching keys", () => {
    app.setSortCol("depth");
    app.setSortCol("depth"); // now desc
    app.setSortCol("date");  // different key → back to asc
    expect(app.diveListPrefs.sortDir).toBe("asc");
  });

  it("toggleColumn removes a visible column from colOrder", () => {
    expect(app.diveListPrefs.colOrder).toContain("buddy");
    app.toggleColumn("buddy");
    expect(app.diveListPrefs.colOrder).not.toContain("buddy");
  });

  it("toggleColumn adds a hidden column to end of colOrder", () => {
    expect(app.diveListPrefs.colOrder).not.toContain("temp");
    app.toggleColumn("temp");
    const order = app.diveListPrefs.colOrder;
    expect(order).toContain("temp");
    expect(order[order.length - 1]).toBe("temp");
  });

  it("visibleCols returns ColDefs matching colOrder ids", () => {
    expect(app.visibleCols.map(c => c.id)).toEqual(app.diveListPrefs.colOrder);
  });
});

describe("sortedDives", () => {
  beforeEach(() => app.reset());

  it("returns logbook dives unchanged when sortKey is nr", () => {
    app.logbook = {
      dives: [makeDive({ number: 2, dateTime: "2024-02-01T00:00:00" }), makeDive({ number: 1, dateTime: "2024-01-01T00:00:00" })],
      trips: [], sites: [], units: "METRIC",
    };
    expect(app.sortedDives[0].number).toBe(2);
    expect(app.sortedDives[1].number).toBe(1);
  });

  it("sorts by depth ascending", () => {
    app.logbook = {
      dives: [makeDive({ number: 1, maxDepthM: 30 }), makeDive({ number: 2, maxDepthM: 10 })],
      trips: [], sites: [], units: "METRIC",
    };
    app.setSortCol("depth");
    expect(app.sortedDives[0].number).toBe(2);
  });

  it("sorts by depth descending after toggle", () => {
    app.logbook = {
      dives: [makeDive({ number: 1, maxDepthM: 10 }), makeDive({ number: 2, maxDepthM: 30 })],
      trips: [], sites: [], units: "METRIC",
    };
    app.setSortCol("depth");
    app.setSortCol("depth"); // desc
    expect(app.sortedDives[0].number).toBe(2);
  });

  it("dives with missing sort field sort last in both directions", () => {
    app.logbook = {
      dives: [
        makeDive({ number: 1, maxDepthM: 20 }),
        makeDive({ number: 2 }),               // no depth
        makeDive({ number: 3, maxDepthM: 10 }),
      ],
      trips: [], sites: [], units: "METRIC",
    };
    app.setSortCol("depth"); // asc
    expect(app.sortedDives[2].number).toBe(2);
    app.setSortCol("depth"); // desc
    expect(app.sortedDives[2].number).toBe(2);
  });
});
