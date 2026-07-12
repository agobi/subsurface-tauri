// AI-generated (Claude)
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import { within } from "@testing-library/dom";
import { invoke } from "@tauri-apps/api/core";
import MobileSettingsScreen from "$lib/components/MobileSettingsScreen.svelte";
import { app } from "$lib/stores/app.svelte.ts";

describe("MobileSettingsScreen", () => {
  beforeEach(() => {
    app.reset();
    vi.mocked(invoke).mockReset().mockResolvedValue("INFO");
  });

  it("renders the Settings title", () => {
    render(MobileSettingsScreen, { props: { onBack: vi.fn() } });
    expect(screen.getByText("Settings")).toBeInTheDocument();
  });

  it("renders appearance section with theme radios", () => {
    render(MobileSettingsScreen, { props: { onBack: vi.fn() } });
    const group = screen.getByRole("radiogroup", { name: "Color Theme" });
    expect(within(group).getByLabelText("Light")).toBeInTheDocument();
    expect(within(group).getByLabelText("Dark")).toBeInTheDocument();
    expect(within(group).getByLabelText("Auto")).toBeInTheDocument();
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

  it("renders units radio group and updates app.unitsPref when changed", async () => {
    render(MobileSettingsScreen, { props: { onBack: vi.fn() } });
    expect(screen.getByLabelText("Metric")).toBeInTheDocument();
    expect(screen.getByLabelText("Imperial")).toBeInTheDocument();
    await fireEvent.change(screen.getByLabelText("Imperial"));
    expect(app.unitsPref).toBe("IMPERIAL");
  });

  it("renders logging section with level radios reflecting the backend's current level", async () => {
    vi.mocked(invoke).mockResolvedValue("DEBUG");
    render(MobileSettingsScreen, { props: { onBack: vi.fn() } });
    expect(await screen.findByLabelText("Debug")).toBeChecked();
    expect(screen.getByLabelText("Trace")).not.toBeChecked();
  });

  it("invokes set_log_level when a log level radio changes", async () => {
    render(MobileSettingsScreen, { props: { onBack: vi.fn() } });
    await fireEvent.change(screen.getByLabelText("Trace"));
    expect(invoke).toHaveBeenCalledWith("set_log_level", { level: "trace" });
  });

  describe("recents management", () => {
    beforeEach(() => {
      app.recents = [
        { kind: "Local", path: "/some/local/path" },
        { kind: "Cloud", email: "user@example.com", url: "https://ssrf-cloud-eu.subsurface-divelog.org" },
      ];
    });

    it("does not show Clear All when there are no recents", () => {
      app.recents = [];
      render(MobileSettingsScreen, { props: { onBack: vi.fn() } });
      expect(screen.queryByText("Clear All")).not.toBeInTheDocument();
    });

    it("shows a remove button per recent entry", () => {
      render(MobileSettingsScreen, { props: { onBack: vi.fn() } });
      expect(screen.getByLabelText("Remove path")).toBeInTheDocument();
      expect(screen.getByLabelText(/Remove user@example.com/)).toBeInTheDocument();
    });

    it("invokes remove_recent with the entry's index when its remove button is clicked", async () => {
      render(MobileSettingsScreen, { props: { onBack: vi.fn() } });
      vi.mocked(invoke).mockResolvedValueOnce([app.recents[1]]);
      await fireEvent.click(screen.getByLabelText("Remove path"));
      expect(invoke).toHaveBeenCalledWith("remove_recent", { index: 0 });
    });

    it("invokes clear_recents when Clear All is clicked", async () => {
      render(MobileSettingsScreen, { props: { onBack: vi.fn() } });
      vi.mocked(invoke).mockResolvedValueOnce([]);
      await fireEvent.click(screen.getByText("Clear All"));
      expect(invoke).toHaveBeenCalledWith("clear_recents");
    });
  });
});
