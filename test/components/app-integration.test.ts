// AI-generated (Claude)
import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import QuadrantGrid from "$lib/components/QuadrantGrid.svelte";
import { app } from "$lib/stores/app.svelte.ts";
import { invoke } from "@tauri-apps/api/core";
import type { OpenResult } from "$lib/types.ts";
import sample from "$lib/fixtures/logbook.sample.json";

describe("QuadrantGrid wired to data", () => {
  beforeEach(async () => {
    app.reset();
    vi.mocked(invoke)
      .mockResolvedValueOnce({ logbook: sample, displayName: "test", recents: [] } as unknown as OpenResult)
      .mockResolvedValueOnce((sample as any).dives[0]); // get_dive called by selectDive
    await app.startup();
  });

  it("renders the selected dive's profile (svg) when a dive is selected", () => {
    render(QuadrantGrid, { props: { query: "" } });
    expect(document.querySelector("svg")).toBeInTheDocument();
  });

  it("renders dive-list rows", () => {
    render(QuadrantGrid, { props: { query: "" } });
    const rows = screen.getAllByRole("button").filter((b) => b.classList.contains("dl-row"));
    expect(rows.length).toBeGreaterThan(0);
  });
});
