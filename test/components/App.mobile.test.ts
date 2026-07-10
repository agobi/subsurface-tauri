// AI-generated (Claude)
import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/svelte";
import { platform } from "@tauri-apps/plugin-os";
import App from "../../src/App.svelte";
import { app } from "$lib/stores/app.svelte.ts";

describe("App — mobile branch", () => {
  beforeEach(() => {
    app.reset();
    vi.mocked(platform).mockResolvedValueOnce("android" as any);
    // jsdom doesn't implement matchMedia; App.svelte needs it in onMount
    window.matchMedia = vi.fn().mockReturnValue({
      matches: false,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    });
  });

  it("renders MobileLayout's two-row shell when platform is android", async () => {
    render(App);
    await waitFor(() => {
      expect(screen.getByTestId("mobile-panel-dives")).toBeInTheDocument();
    });
    expect(screen.getByTestId("mobile-panel-info")).toBeInTheDocument();
    expect(screen.getByTestId("mobile-panel-profile")).toBeInTheDocument();
    expect(screen.getByTestId("mobile-panel-map")).toBeInTheDocument();
  });

  it("does not render desktop Toolbar when platform is android", async () => {
    render(App);
    await waitFor(() => {
      expect(screen.getByTestId("mobile-panel-dives")).toBeInTheDocument();
    });
    expect(screen.queryByRole("button", { name: /add dive/i })).not.toBeInTheDocument();
  });
});
