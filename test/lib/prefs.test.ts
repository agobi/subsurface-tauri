// AI-generated (Claude)
import { describe, it, expect, vi, afterEach } from "vitest";
import * as store from "@tauri-apps/plugin-store";
import * as event from "@tauri-apps/api/event";
import {
  resolveTheme,
  applyTheme,
  loadAppearancePrefs,
  saveAndEmitAppearance,
} from "$lib/prefs.ts";

describe("resolveTheme", () => {
  it("returns 'dark' directly", () => {
    expect(resolveTheme("dark")).toBe("dark");
  });

  it("returns 'light' directly", () => {
    expect(resolveTheme("light")).toBe("light");
  });

  it("resolves 'auto' to dark when prefers-color-scheme is dark", () => {
    window.matchMedia = vi.fn().mockReturnValue({ matches: true });
    expect(resolveTheme("auto")).toBe("dark");
  });

  it("resolves 'auto' to light when prefers-color-scheme is light", () => {
    window.matchMedia = vi.fn().mockReturnValue({ matches: false });
    expect(resolveTheme("auto")).toBe("light");
  });
});

describe("applyTheme", () => {
  it("sets data-theme on documentElement", () => {
    applyTheme("light");
    expect(document.documentElement.dataset.theme).toBe("light");
  });

  it("never writes 'auto' to data-theme", () => {
    window.matchMedia = vi.fn().mockReturnValue({ matches: true });
    applyTheme("auto");
    expect(document.documentElement.dataset.theme).not.toBe("auto");
    expect(document.documentElement.dataset.theme).toBe("dark");
  });
});

describe("loadAppearancePrefs", () => {
  afterEach(() => {
    vi.resetAllMocks();
  });

  it("returns default dark theme when settings.json has no appearance key", async () => {
    const mockGet = vi.fn().mockResolvedValue(null);
    vi.mocked(store.load).mockResolvedValueOnce({
      get: mockGet,
      set: vi.fn(),
      save: vi.fn(),
    } as any);
    const prefs = await loadAppearancePrefs();
    expect(prefs).toEqual({ theme: "dark" });
  });

  it("returns saved theme when present", async () => {
    const mockGet = vi.fn().mockResolvedValue({ theme: "light" });
    vi.mocked(store.load).mockResolvedValueOnce({
      get: mockGet,
      set: vi.fn(),
      save: vi.fn(),
    } as any);
    const prefs = await loadAppearancePrefs();
    expect(prefs).toEqual({ theme: "light" });
  });
});

describe("saveAndEmitAppearance", () => {
  afterEach(() => {
    vi.resetAllMocks();
  });

  it("writes appearance key and saves, then emits event", async () => {
    const mockSet = vi.fn();
    const mockSave = vi.fn().mockResolvedValue(undefined);
    vi.mocked(store.load).mockResolvedValueOnce({
      get: vi.fn(),
      set: mockSet,
      save: mockSave,
    } as any);

    await saveAndEmitAppearance({ theme: "auto" });

    expect(mockSet).toHaveBeenCalledWith("appearance", { theme: "auto" });
    expect(mockSave).toHaveBeenCalled();
    expect(vi.mocked(event.emit)).toHaveBeenCalledWith("prefs:appearance-changed", { theme: "auto" });
  });
});
