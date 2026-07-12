// AI-generated (Claude)
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import { describe, it, expect, vi, beforeEach } from "vitest";
import * as store from "@tauri-apps/plugin-store";
import DcDownloadSection from "$lib/components/prefs/DcDownloadSection.svelte";

vi.mock("@tauri-apps/plugin-store");

describe("DcDownloadSection", () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it("shows the default 15-minute gap when nothing is saved yet", async () => {
    vi.mocked(store.load).mockResolvedValue({
      get: vi.fn().mockResolvedValue(null),
      set: vi.fn(),
      save: vi.fn(),
    } as any);
    render(DcDownloadSection);
    const input = await screen.findByLabelText("Merge segments within");
    await waitFor(() => {
      expect((input as HTMLInputElement).value).toBe("15");
    });
  });

  it("shows the saved gap when present", async () => {
    vi.mocked(store.load).mockResolvedValue({
      get: vi.fn().mockResolvedValue({ mergeGapMinutes: 30 }),
      set: vi.fn(),
      save: vi.fn(),
    } as any);
    render(DcDownloadSection);
    const input = await screen.findByLabelText("Merge segments within");
    await waitFor(() => {
      expect((input as HTMLInputElement).value).toBe("30");
    });
  });

  it("saves the new value when changed", async () => {
    const mockSet = vi.fn();
    const mockSave = vi.fn().mockResolvedValue(undefined);
    vi.mocked(store.load).mockResolvedValue({
      get: vi.fn().mockResolvedValue(null),
      set: mockSet,
      save: mockSave,
    } as any);
    render(DcDownloadSection);
    const input = await screen.findByLabelText("Merge segments within");
    await fireEvent.change(input, { target: { value: "45" } });
    expect(mockSet).toHaveBeenCalledWith("dcDownload", { mergeGapMinutes: 45 });
    expect(mockSave).toHaveBeenCalled();
  });
});
