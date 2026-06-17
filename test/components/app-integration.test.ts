// AI-generated (Claude)
import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/svelte";
import QuadrantGrid from "$lib/components/QuadrantGrid.svelte";
import { app } from "$lib/stores/app.svelte.ts";

describe("QuadrantGrid wired to data", () => {
  beforeEach(() => app.reset());

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
