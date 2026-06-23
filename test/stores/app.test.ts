// AI-generated (Claude)
import { describe, it, expect, beforeEach, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { app } from "$lib/stores/app.svelte.ts";
import type { Dive, OpenResult } from "$lib/types.ts";
import sample from "$lib/fixtures/logbook.sample.json";

function openResult(overrides: Partial<OpenResult> = {}): OpenResult {
  return { logbook: sample as any, displayName: "test", recents: [], ...overrides };
}

const cloudRecents = [{ kind: "Cloud" as const, email: "user@example.com", url: "https://ssrf-cloud-eu.subsurface-divelog.org" }];
const localRecents = [{ kind: "Local" as const, path: "/some/local/path" }];

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

  it("toggleColumn hides a visible column (adds to hiddenCols, colOrder unchanged)", () => {
    const orderBefore = [...app.diveListPrefs.colOrder];
    app.toggleColumn("buddy");
    expect(app.diveListPrefs.hiddenCols).toContain("buddy");
    expect(app.diveListPrefs.colOrder).toEqual(orderBefore);
  });

  it("toggleColumn shows a hidden column (removes from hiddenCols, colOrder unchanged)", () => {
    // "temp" starts hidden by default
    const orderBefore = [...app.diveListPrefs.colOrder];
    app.toggleColumn("temp");
    expect(app.diveListPrefs.hiddenCols).not.toContain("temp");
    expect(app.diveListPrefs.colOrder).toEqual(orderBefore);
  });

  it("visibleCols excludes hiddenCols entries and preserves colOrder order", () => {
    app.diveListPrefs = { ...app.diveListPrefs, hiddenCols: ["depth", "buddy"] };
    const ids = app.visibleCols.map(c => c.id);
    expect(ids).not.toContain("depth");
    expect(ids).not.toContain("buddy");
    // Visible ids must appear in same relative order as colOrder
    const visibleFromOrder = app.diveListPrefs.colOrder.filter(id => !["depth", "buddy"].includes(id));
    expect(ids).toEqual(visibleFromOrder);
  });

  it("reorderColumn moves fromId to toId position", () => {
    // Set a simple known order
    app.diveListPrefs = {
      ...app.diveListPrefs,
      colOrder: ["nr", "date", "depth", "duration", "buddy"],
    };
    app.reorderColumn("depth", "nr");
    expect(app.diveListPrefs.colOrder).toEqual(["depth", "nr", "date", "duration", "buddy"]);
  });

  it("reorderColumn with same id is a no-op", () => {
    const before = [...app.diveListPrefs.colOrder];
    app.reorderColumn("nr", "nr");
    expect(app.diveListPrefs.colOrder).toEqual(before);
  });

  it("reorderColumn with unknown fromId is a no-op", () => {
    const before = [...app.diveListPrefs.colOrder];
    app.reorderColumn("unknown" as any, "nr");
    expect(app.diveListPrefs.colOrder).toEqual(before);
  });

  it("reorderColumn with unknown toId is a no-op", () => {
    const before = [...app.diveListPrefs.colOrder];
    app.reorderColumn("nr", "unknown" as any);
    expect(app.diveListPrefs.colOrder).toEqual(before);
  });
});

describe("sortedDives", () => {
  beforeEach(() => app.reset());

  it("sorts by dive number ascending when sortKey is nr (default)", () => {
    app.logbook = {
      dives: [makeDive({ number: 3 }), makeDive({ number: 1 }), makeDive({ number: 2 })],
      trips: [], sites: [], units: "METRIC",
    };
    expect(app.sortedDives.map(d => d.number)).toEqual([1, 2, 3]);
  });

  it("sorts by dive number descending when sortKey is nr and sortDir is desc", () => {
    app.logbook = {
      dives: [makeDive({ number: 3 }), makeDive({ number: 1 }), makeDive({ number: 2 })],
      trips: [], sites: [], units: "METRIC",
    };
    app.setSortCol("nr"); // toggles asc→desc since "nr" is already the default sortKey
    expect(app.sortedDives.map(d => d.number)).toEqual([3, 2, 1]);
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
    app.recents = cloudRecents;
    app.reset();
    expect(app.isCloudLogbook).toBe(false);
  });

  it("openCloud() invokes open_cloud_logbook and sets isCloudLogbook to true", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(openResult({ recents: cloudRecents }));
    await app.openCloud("user@example.com", "secret");
    expect(invoke).toHaveBeenCalledWith("open_cloud_logbook", {
      email: "user@example.com",
      password: "secret",
    });
    expect(app.isCloudLogbook).toBe(true);
    expect(app.logbook.dives.length).toBeGreaterThan(0);
  });

  it("openCloud() selects the first dive", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(openResult());
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
    vi.mocked(invoke).mockResolvedValueOnce(openResult());
    await app.openCloud("user@example.com", "secret");
    const firstId = app.selectedDiveId;

    vi.mocked(invoke).mockResolvedValueOnce(openResult());
    await app.syncCloud();
    expect(app.selectedDiveId).toBe(firstId);
  });

  it("syncCloud() resets selectedDiveId to first dive when previous dive no longer exists", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(openResult());
    await app.openCloud("user@example.com", "secret");
    app.selectedDiveId = 999999; // a dive that won't exist after sync

    vi.mocked(invoke).mockResolvedValueOnce(openResult());
    await app.syncCloud();
    expect(app.selectedDiveId).toBe(app.logbook.dives[0]?.number ?? null);
  });

  it("open() sets isCloudLogbook to false", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(openResult({ recents: cloudRecents }));
    await app.openCloud("user@example.com", "secret");
    expect(app.isCloudLogbook).toBe(true);

    vi.mocked(invoke).mockResolvedValueOnce(openResult({ recents: localRecents }));
    await app.open("/some/local/path");
    expect(app.isCloudLogbook).toBe(false);
  });

  it("newLogbook() sets isCloudLogbook to false", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(openResult({ recents: cloudRecents }));
    await app.openCloud("user@example.com", "secret");

    vi.mocked(invoke).mockResolvedValueOnce(openResult({ recents: localRecents }));
    await app.newLogbook("/some/new/path");
    expect(app.isCloudLogbook).toBe(false);
  });

  it("openRecent() with Cloud entry invokes open_recent_cloud_logbook (not showCloudDialog)", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(openResult({ recents: cloudRecents }));
    await app.openRecent({ kind: "Cloud", email: "user@example.com", url: "https://ssrf-cloud-eu.subsurface-divelog.org" });
    expect(invoke).toHaveBeenCalledWith("open_recent_cloud_logbook", { email: "user@example.com" });
    expect(app.showCloudDialog).toBe(false);
  });

  it("openRecent() with Local entry invokes open_logbook", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(openResult({ recents: localRecents }));
    await app.openRecent({ kind: "Local", path: "/some/local/path" });
    expect(invoke).toHaveBeenCalledWith("open_logbook", { root: "/some/local/path" });
  });
});
