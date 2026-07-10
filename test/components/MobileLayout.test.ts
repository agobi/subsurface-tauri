// AI-generated (Claude)
// test/components/MobileLayout.test.ts
import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import MobileLayout from "$lib/components/MobileLayout.svelte";
import { app } from "$lib/stores/app.svelte.ts";

describe("MobileLayout", () => {
  beforeEach(() => {
    app.reset();
    Element.prototype.scrollTo = vi.fn();
  });

  it("always renders the bottom dive-list row, with no tab bar", () => {
    render(MobileLayout);
    expect(screen.getByTestId("mobile-panel-dives")).toBeInTheDocument();
    expect(screen.queryByRole("tablist")).not.toBeInTheDocument();
  });

  it("shows the empty state in the bottom row when there are no dives", () => {
    render(MobileLayout);
    expect(screen.getByTestId("mobile-empty-state")).toBeInTheDocument();
  });

  it("renders three swipeable top panels: info, profile, map", () => {
    render(MobileLayout);
    expect(screen.getByTestId("mobile-panel-info")).toBeInTheDocument();
    expect(screen.getByTestId("mobile-panel-profile")).toBeInTheDocument();
    expect(screen.getByTestId("mobile-panel-map")).toBeInTheDocument();
  });

  it("defaults to Profile as the active top panel", () => {
    render(MobileLayout);
    expect(screen.getByTestId("mobile-active-panel-label")).toHaveTextContent("Profile");
    expect(screen.getByTestId("mobile-dot-profile")).toHaveClass("active");
  });

  it("jumps to a panel when its dot is clicked", async () => {
    render(MobileLayout);
    await fireEvent.click(screen.getByTestId("mobile-dot-info"));
    expect(screen.getByTestId("mobile-active-panel-label")).toHaveTextContent("Info");
    expect(screen.getByTestId("mobile-dot-info")).toHaveClass("active");
    expect(screen.getByTestId("mobile-dot-profile")).not.toHaveClass("active");
  });

  it("shows a settings gear button in the persistent header", () => {
    render(MobileLayout);
    expect(screen.getByRole("button", { name: /settings/i })).toBeInTheDocument();
  });

  it("shows settings screen when gear button is clicked", async () => {
    render(MobileLayout);
    await fireEvent.click(screen.getByRole("button", { name: /settings/i }));
    expect(screen.getByTestId("mobile-settings-screen")).toBeInTheDocument();
  });

  it("returns to main view when back button is clicked in settings", async () => {
    render(MobileLayout);
    await fireEvent.click(screen.getByRole("button", { name: /settings/i }));
    await fireEvent.click(screen.getByRole("button", { name: /back/i }));
    expect(screen.queryByTestId("mobile-settings-screen")).not.toBeInTheDocument();
    expect(screen.getByTestId("mobile-panel-dives")).toBeInTheDocument();
  });
});
