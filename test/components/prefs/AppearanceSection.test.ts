// AI-generated (Claude)
import { render, fireEvent } from "@testing-library/svelte";
import { describe, it, expect, vi } from "vitest";
import AppearanceSection from "$lib/components/prefs/AppearanceSection.svelte";

describe("AppearanceSection", () => {
  it("renders Light, Dark, and Auto radio options", () => {
    const { getByLabelText } = render(AppearanceSection, {
      props: { currentTheme: "dark", onThemeChange: vi.fn() },
    });
    expect(getByLabelText("Light")).toBeInTheDocument();
    expect(getByLabelText("Dark")).toBeInTheDocument();
    expect(getByLabelText("Auto")).toBeInTheDocument();
  });

  it("checks the radio matching currentTheme", () => {
    const { getByLabelText } = render(AppearanceSection, {
      props: { currentTheme: "light", onThemeChange: vi.fn() },
    });
    expect(getByLabelText("Light")).toBeChecked();
    expect(getByLabelText("Dark")).not.toBeChecked();
    expect(getByLabelText("Auto")).not.toBeChecked();
  });

  it("calls onThemeChange with the selected value when a radio changes", async () => {
    const onThemeChange = vi.fn();
    const { getByLabelText } = render(AppearanceSection, {
      props: { currentTheme: "dark", onThemeChange },
    });
    await fireEvent.change(getByLabelText("Auto"));
    expect(onThemeChange).toHaveBeenCalledWith("auto");
  });

  it("calls onThemeChange with 'light' when Light radio changes", async () => {
    const onThemeChange = vi.fn();
    const { getByLabelText } = render(AppearanceSection, {
      props: { currentTheme: "dark", onThemeChange },
    });
    await fireEvent.change(getByLabelText("Light"));
    expect(onThemeChange).toHaveBeenCalledWith("light");
  });
});
