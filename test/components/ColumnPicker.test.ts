// AI-generated (Claude)
import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import ColumnPicker from "$lib/components/ColumnPicker.svelte";
import { app } from "$lib/stores/app.svelte.ts";
import { ALL_COLS } from "$lib/diveListColumns.ts";

describe("ColumnPicker", () => {
  beforeEach(() => app.reset());

  it("renders all 17 columns in colOrder sequence", () => {
    render(ColumnPicker, { props: { open: true } });
    const rows = document.querySelectorAll("[data-testid^='col-row-']");
    expect(rows.length).toBe(ALL_COLS.length);
    // First row matches first id in colOrder
    expect(rows[0].getAttribute("data-testid")).toBe(`col-row-${app.diveListPrefs.colOrder[0]}`);
  });

  it("hidden columns are present in DOM but carry the 'hidden' CSS class", () => {
    // "temp" is hidden by default
    render(ColumnPicker, { props: { open: true } });
    const tempRow = document.querySelector("[data-testid='col-row-temp']");
    expect(tempRow).not.toBeNull();
    expect(tempRow!.classList.contains("hidden")).toBe(true);
  });

  it("visible columns do not have the 'hidden' CSS class", () => {
    render(ColumnPicker, { props: { open: true } });
    const nrRow = document.querySelector("[data-testid='col-row-nr']");
    expect(nrRow!.classList.contains("hidden")).toBe(false);
  });

  it("clicking checkbox for a visible column adds it to hiddenCols", async () => {
    render(ColumnPicker, { props: { open: true } });
    const checkbox = screen.getByRole("checkbox", { name: /buddy/i });
    await fireEvent.click(checkbox);
    expect(app.diveListPrefs.hiddenCols).toContain("buddy");
  });

  it("clicking checkbox for a hidden column removes it from hiddenCols", async () => {
    render(ColumnPicker, { props: { open: true } });
    // "temp" is hidden by default; checkbox is unchecked
    const checkbox = screen.getByRole("checkbox", { name: /temp/i });
    await fireEvent.click(checkbox);
    expect(app.diveListPrefs.hiddenCols).not.toContain("temp");
  });

  it("dragstart on a row then drop on another row calls reorderColumn", async () => {
    // Set a simple known order
    app.diveListPrefs = {
      ...app.diveListPrefs,
      colOrder: ["nr", "date", "depth", "duration", "buddy", "guide", "country", "location", "rating", "temp", "suit", "cylinder", "sac", "tags", "notes", "divemode", "weight"],
    };
    render(ColumnPicker, { props: { open: true } });
    const depthRow = document.querySelector("[data-testid='col-row-depth']")!;
    const nrRow = document.querySelector("[data-testid='col-row-nr']")!;
    await fireEvent.dragStart(depthRow);
    await fireEvent.drop(nrRow);
    // "depth" should now be at index 0, displacing "nr" to index 1
    expect(app.diveListPrefs.colOrder[0]).toBe("depth");
    expect(app.diveListPrefs.colOrder[1]).toBe("nr");
  });

  it("drop on the same row as dragstart is a no-op", async () => {
    const before = [...app.diveListPrefs.colOrder];
    render(ColumnPicker, { props: { open: true } });
    const nrRow = document.querySelector("[data-testid='col-row-nr']")!;
    await fireEvent.dragStart(nrRow);
    await fireEvent.drop(nrRow);
    expect(app.diveListPrefs.colOrder).toEqual(before);
  });
});
