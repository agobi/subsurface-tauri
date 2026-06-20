// AI-generated (Claude)
import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import MobileLayout from "$lib/components/MobileLayout.svelte";
import { app } from "$lib/stores/app.svelte.ts";

describe("MobileLayout", () => {
  beforeEach(() => app.reset());

  it("renders all four tab buttons", () => {
    render(MobileLayout);
    expect(screen.getByRole("tab", { name: /dives/i })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: /profile/i })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: /info/i })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: /map/i })).toBeInTheDocument();
  });

  it("shows the dives panel by default", () => {
    render(MobileLayout);
    expect(screen.getByTestId("mobile-panel-dives")).toBeInTheDocument();
  });

  it("switches to profile panel when Profile tab is clicked", async () => {
    render(MobileLayout);
    await fireEvent.click(screen.getByRole("tab", { name: /profile/i }));
    expect(screen.getByTestId("mobile-panel-profile")).toBeInTheDocument();
    expect(screen.queryByTestId("mobile-panel-dives")).not.toBeInTheDocument();
  });

  it("switches to info panel when Info tab is clicked", async () => {
    render(MobileLayout);
    await fireEvent.click(screen.getByRole("tab", { name: /info/i }));
    expect(screen.getByTestId("mobile-panel-info")).toBeInTheDocument();
  });

  it("switches to map panel when Map tab is clicked", async () => {
    render(MobileLayout);
    await fireEvent.click(screen.getByRole("tab", { name: /map/i }));
    expect(screen.getByTestId("mobile-panel-map")).toBeInTheDocument();
  });

  it("marks the active tab with aria-selected=true", async () => {
    render(MobileLayout);
    const divesTab = screen.getByRole("tab", { name: /dives/i });
    expect(divesTab).toHaveAttribute("aria-selected", "true");
    await fireEvent.click(screen.getByRole("tab", { name: /profile/i }));
    expect(divesTab).toHaveAttribute("aria-selected", "false");
    expect(screen.getByRole("tab", { name: /profile/i })).toHaveAttribute("aria-selected", "true");
  });

  it("shows a gear button in the dives panel header", () => {
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
