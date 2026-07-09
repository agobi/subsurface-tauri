// AI-generated (Claude)
import { describe, it, expect, beforeEach, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { app } from "$lib/stores/app.svelte.ts";
import type { OpenResult } from "$lib/types.ts";
import sample from "$lib/fixtures/logbook.sample.json";

function openResult(): OpenResult {
  return { logbook: sample as any, displayName: "test", recents: [], warnings: [] };
}

describe("logbook loading via IPC", () => {
  beforeEach(() => app.reset());

  it("startup() calls startup_logbook and populates the store", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(openResult());
    await app.startup();
    expect(invoke).toHaveBeenCalledWith("startup_logbook");
    expect(app.logbook.dives.length).toBeGreaterThan(0);
    expect(app.selectedDiveId).toBe(app.logbook.dives[0].number);
  });

  it("open() calls open_logbook with the path", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(openResult());
    await app.open("/some/path");
    expect(invoke).toHaveBeenCalledWith("open_logbook", { root: "/some/path" });
    expect(app.logbook.dives.length).toBeGreaterThan(0);
  });

  it("starts with an empty logbook before startup()", () => {
    expect(app.logbook.dives).toHaveLength(0);
    expect(app.selectedDiveId).toBeNull();
  });

  it("selectedDive is populated after startup() calls selectDive()", async () => {
    const sampleDive = (sample as any).dives[0];
    vi.mocked(invoke)
      .mockResolvedValueOnce(openResult())  // startup_logbook
      .mockResolvedValueOnce(sampleDive);   // get_dive called by selectDive
    await app.startup();
    expect(app.selectedDive?.number).toBe(sampleDive.number);
  });
});
