// AI-generated (Claude)
import { render, fireEvent } from "@testing-library/svelte";
import { describe, it, expect, vi } from "vitest";
import LoggingSection from "$lib/components/prefs/LoggingSection.svelte";

describe("LoggingSection", () => {
  it("renders all five level options", () => {
    const { getByLabelText } = render(LoggingSection, {
      props: { currentLevel: "info", onLevelChange: vi.fn() },
    });
    for (const label of ["Error", "Warning", "Info", "Debug", "Trace"]) {
      expect(getByLabelText(label)).toBeInTheDocument();
    }
  });

  it("checks the radio matching currentLevel", () => {
    const { getByLabelText } = render(LoggingSection, {
      props: { currentLevel: "debug", onLevelChange: vi.fn() },
    });
    expect(getByLabelText("Debug")).toBeChecked();
    expect(getByLabelText("Info")).not.toBeChecked();
  });

  it("calls onLevelChange with the selected value", async () => {
    const onLevelChange = vi.fn();
    const { getByLabelText } = render(LoggingSection, {
      props: { currentLevel: "info", onLevelChange },
    });
    await fireEvent.change(getByLabelText("Trace"));
    expect(onLevelChange).toHaveBeenCalledWith("trace");
  });
});
