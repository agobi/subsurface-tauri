// AI-generated (Claude)
import { describe, it, expect, beforeEach, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { app } from "$lib/stores/app.svelte.ts";
import type { Dive, Logbook } from "$lib/types.ts";
import sample from "$lib/fixtures/logbook.sample.json";

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

describe("cloud logbook", () => {
  beforeEach(() => app.reset());

  it("isCloudLogbook starts false", () => {
    expect(app.isCloudLogbook).toBe(false);
  });

  it("reset() sets isCloudLogbook to false", () => {
    app.isCloudLogbook = true; // force it
    app.reset();
    expect(app.isCloudLogbook).toBe(false);
  });

  it("openCloud() invokes open_cloud_logbook and sets isCloudLogbook to true", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(sample as unknown as Logbook);
    await app.openCloud("user@example.com", "secret");
    expect(invoke).toHaveBeenCalledWith("open_cloud_logbook", {
      email: "user@example.com",
      password: "secret",
    });
    expect(app.isCloudLogbook).toBe(true);
    expect(app.logbook.dives.length).toBeGreaterThan(0);
  });

  it("openCloud() selects the first dive", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(sample as unknown as Logbook);
    await app.openCloud("user@example.com", "secret");
    expect(app.selectedDiveId).toBe(app.logbook.dives[0]?.number ?? null);
  });

  it("openCloud() propagates errors without setting isCloudLogbook", async () => {
    vi.mocked(invoke).mockRejectedValueOnce("Authentication failed. Check your email and password.");
    await expect(app.openCloud("user@example.com", "wrong")).rejects.toBe(
      "Authentication failed. Check your email and password."
    );
    expect(app.isCloudLogbook).toBe(false);
  });

  it("syncCloud() updates logbook and retains selectedDiveId when dive still exists", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(sample as unknown as Logbook);
    await app.openCloud("user@example.com", "secret");
    const firstId = app.selectedDiveId;

    vi.mocked(invoke).mockResolvedValueOnce(sample as unknown as Logbook);
    await app.syncCloud();
    expect(app.selectedDiveId).toBe(firstId);
  });

  it("syncCloud() resets selectedDiveId to first dive when previous dive no longer exists", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(sample as unknown as Logbook);
    await app.openCloud("user@example.com", "secret");
    app.selectedDiveId = 999999; // a dive that won't exist after sync

    vi.mocked(invoke).mockResolvedValueOnce(sample as unknown as Logbook);
    await app.syncCloud();
    expect(app.selectedDiveId).toBe(app.logbook.dives[0]?.number ?? null);
  });

  it("open() sets isCloudLogbook to false", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(sample as unknown as Logbook);
    await app.openCloud("user@example.com", "secret");
    expect(app.isCloudLogbook).toBe(true);

    vi.mocked(invoke).mockResolvedValueOnce(sample as unknown as Logbook);
    await app.open("/some/local/path");
    expect(app.isCloudLogbook).toBe(false);
  });

  it("newLogbook() sets isCloudLogbook to false", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(sample as unknown as Logbook);
    await app.openCloud("user@example.com", "secret");

    vi.mocked(invoke).mockResolvedValueOnce(sample as unknown as Logbook);
    await app.newLogbook("/some/new/path");
    expect(app.isCloudLogbook).toBe(false);
  });
});
