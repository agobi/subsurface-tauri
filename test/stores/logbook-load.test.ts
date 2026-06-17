// AI-generated (Claude)
import { describe, it, expect } from "vitest";
import { app } from "$lib/stores/app.svelte.ts";

describe("logbook loading", () => {
  it("loads a logbook by default (dives present)", () => {
    expect(app.logbook.units).toBe("METRIC");
    expect(app.logbook.dives.length).toBeGreaterThan(0);
    expect(app.logbook.sites.length).toBeGreaterThan(0);
  });

  it("selectedDive returns the dive matching selectedDiveId", () => {
    const first = app.logbook.dives[0];
    app.selectedDiveId = first.number;
    expect(app.selectedDive?.number).toBe(first.number);
  });
});
