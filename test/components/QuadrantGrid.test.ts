// AI-generated (Claude)
import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import QuadrantGrid from "$lib/components/QuadrantGrid.svelte";
import { app } from "$lib/stores/app.svelte.ts";

describe("QuadrantGrid", () => {
  beforeEach(() => app.reset());

  it("renders all four quadrant regions when all visible", () => {
    render(QuadrantGrid);
    expect(screen.getByTestId("quad-info")).toBeInTheDocument();
    expect(screen.getByTestId("quad-profile")).toBeInTheDocument();
    expect(screen.getByTestId("quad-list")).toBeInTheDocument();
    expect(screen.getByTestId("quad-map")).toBeInTheDocument();
  });

  it("shows only the active panel in single-panel mode", () => {
    app.visiblePanels = { list: true, profile: false, info: false, map: false };
    render(QuadrantGrid);
    expect(screen.getByTestId("quad-list")).toBeInTheDocument();
    expect(screen.queryByTestId("quad-info")).not.toBeInTheDocument();
    expect(screen.queryByTestId("quad-profile")).not.toBeInTheDocument();
    expect(screen.queryByTestId("quad-map")).not.toBeInTheDocument();
  });

  it("exposes a vertical splitter that updates the column template", async () => {
    app.reset();
    const { container } = render(QuadrantGrid);
    const splitter = screen.getByTestId("splitter-v");
    await fireEvent.mouseDown(splitter, { clientX: 500 });
    await fireEvent.mouseMove(window, { clientX: 600 });
    await fireEvent.mouseUp(window);
    const grid = container.querySelector(".quad-grid") as HTMLElement;
    expect(grid.style.gridTemplateColumns).toMatch(/fr/);
  });
});
