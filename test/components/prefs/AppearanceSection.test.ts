// AI-generated (Claude)
import { render, fireEvent } from "@testing-library/svelte";
import { within } from "@testing-library/dom";
import { describe, it, expect, vi } from "vitest";
import AppearanceSection from "$lib/components/prefs/AppearanceSection.svelte";

function renderSection(overrides: { currentTheme?: string; currentUnits?: string } = {}) {
  const onThemeChange = vi.fn();
  const onUnitsChange = vi.fn();
  const utils = render(AppearanceSection, {
    props: {
      currentTheme: (overrides.currentTheme ?? "dark") as any,
      onThemeChange,
      currentUnits: (overrides.currentUnits ?? "auto") as any,
      onUnitsChange,
    },
  });
  return { ...utils, onThemeChange, onUnitsChange };
}

describe("AppearanceSection — theme", () => {
  it("renders Light, Dark, and Auto radio options", () => {
    const { getByRole } = renderSection();
    const group = getByRole("radiogroup", { name: "Color Theme" });
    expect(within(group).getByLabelText("Light")).toBeInTheDocument();
    expect(within(group).getByLabelText("Dark")).toBeInTheDocument();
    expect(within(group).getByLabelText("Auto")).toBeInTheDocument();
  });

  it("checks the radio matching currentTheme", () => {
    const { getByRole } = renderSection({ currentTheme: "light" });
    const group = getByRole("radiogroup", { name: "Color Theme" });
    expect(within(group).getByLabelText("Light")).toBeChecked();
    expect(within(group).getByLabelText("Dark")).not.toBeChecked();
    expect(within(group).getByLabelText("Auto")).not.toBeChecked();
  });

  it("calls onThemeChange with the selected value when a radio changes", async () => {
    const { getByRole, onThemeChange } = renderSection({ currentTheme: "dark" });
    const group = getByRole("radiogroup", { name: "Color Theme" });
    await fireEvent.change(within(group).getByLabelText("Auto"));
    expect(onThemeChange).toHaveBeenCalledWith("auto");
  });

  it("calls onThemeChange with 'light' when Light radio changes", async () => {
    const { getByRole, onThemeChange } = renderSection({ currentTheme: "dark" });
    const group = getByRole("radiogroup", { name: "Color Theme" });
    await fireEvent.change(within(group).getByLabelText("Light"));
    expect(onThemeChange).toHaveBeenCalledWith("light");
  });
});

describe("AppearanceSection — units", () => {
  it("renders Auto, Metric, and Imperial radio options", () => {
    const { getByRole } = renderSection();
    const group = getByRole("radiogroup", { name: "Units" });
    expect(within(group).getByLabelText("Auto")).toBeInTheDocument();
    expect(within(group).getByLabelText("Metric")).toBeInTheDocument();
    expect(within(group).getByLabelText("Imperial")).toBeInTheDocument();
  });

  it("checks the radio matching currentUnits", () => {
    const { getByRole } = renderSection({ currentUnits: "IMPERIAL" });
    const group = getByRole("radiogroup", { name: "Units" });
    expect(within(group).getByLabelText("Imperial")).toBeChecked();
    expect(within(group).getByLabelText("Metric")).not.toBeChecked();
    expect(within(group).getByLabelText("Auto")).not.toBeChecked();
  });

  it("calls onUnitsChange with 'METRIC' when Metric radio changes", async () => {
    const { getByRole, onUnitsChange } = renderSection({ currentUnits: "auto" });
    const group = getByRole("radiogroup", { name: "Units" });
    await fireEvent.change(within(group).getByLabelText("Metric"));
    expect(onUnitsChange).toHaveBeenCalledWith("METRIC");
  });

  it("calls onUnitsChange with 'IMPERIAL' when Imperial radio changes", async () => {
    const { getByRole, onUnitsChange } = renderSection({ currentUnits: "auto" });
    const group = getByRole("radiogroup", { name: "Units" });
    await fireEvent.change(within(group).getByLabelText("Imperial"));
    expect(onUnitsChange).toHaveBeenCalledWith("IMPERIAL");
  });
});
