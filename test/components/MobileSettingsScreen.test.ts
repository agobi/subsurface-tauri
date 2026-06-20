// AI-generated (Claude)
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import MobileSettingsScreen from "$lib/components/MobileSettingsScreen.svelte";
import { app } from "$lib/stores/app.svelte.ts";

describe("MobileSettingsScreen", () => {
  beforeEach(() => app.reset());

  it("renders the Settings title", () => {
    render(MobileSettingsScreen, { props: { onBack: vi.fn() } });
    expect(screen.getByText("Settings")).toBeInTheDocument();
  });

  it("renders appearance section with theme radios", () => {
    render(MobileSettingsScreen, { props: { onBack: vi.fn() } });
    expect(screen.getByLabelText("Light")).toBeInTheDocument();
    expect(screen.getByLabelText("Dark")).toBeInTheDocument();
    expect(screen.getByLabelText("Auto")).toBeInTheDocument();
  });

  it("calls onBack when back button is clicked", async () => {
    const onBack = vi.fn();
    render(MobileSettingsScreen, { props: { onBack } });
    await fireEvent.click(screen.getByRole("button", { name: /back/i }));
    expect(onBack).toHaveBeenCalledOnce();
  });

  it("updates app.theme when a theme radio changes", async () => {
    render(MobileSettingsScreen, { props: { onBack: vi.fn() } });
    await fireEvent.change(screen.getByLabelText("Light"));
    expect(app.theme).toBe("light");
  });
});
