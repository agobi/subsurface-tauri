// AI-generated (Claude)
import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import MenuBar from "$lib/components/MenuBar.svelte";
import { app } from "$lib/stores/app.svelte.ts";

describe("MenuBar", () => {
  beforeEach(() => app.reset());

  it("renders the top-level menus", () => {
    render(MenuBar);
    for (const m of ["File", "Edit", "Import", "Log", "View", "Help"]) {
      expect(screen.getByRole("button", { name: m })).toBeInTheDocument();
    }
  });

  it("View menu toggles a panel in the store", async () => {
    render(MenuBar);
    await fireEvent.click(screen.getByRole("button", { name: "View" }));
    await fireEvent.click(screen.getByRole("menuitem", { name: /map/i }));
    expect(app.visiblePanels.map).toBe(false);
  });
});
