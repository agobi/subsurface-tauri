// AI-generated (Claude)
import { describe, it, expect, vi, afterEach } from "vitest";
import * as store from "@tauri-apps/plugin-store";
import * as event from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import {
  resolveTheme,
  applyTheme,
  loadAppearancePrefs,
  saveAndEmitAppearance,
  loadDiveListPrefs,
  saveDiveListPrefs,
  loadLoggingPrefs,
  applyLogLevel,
  DEFAULT_DIVE_LIST_PREFS,
  type DiveListPrefs,
} from "$lib/prefs.ts";
import { ALL_COLS } from "$lib/diveListColumns.ts";

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

  it("returns default auto theme when settings.json has no appearance key", async () => {
    const mockGet = vi.fn().mockResolvedValue(null);
    vi.mocked(store.load).mockResolvedValueOnce({
      get: mockGet,
      set: vi.fn(),
      save: vi.fn(),
    } as any);
    const prefs = await loadAppearancePrefs();
    expect(prefs).toEqual({ theme: "auto" });
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

describe("loadDiveListPrefs", () => {
  afterEach(() => vi.resetAllMocks());

  it("returns defaults with hiddenCols when settings.json has no diveList key", async () => {
    vi.mocked(store.load).mockResolvedValueOnce({
      get: vi.fn().mockResolvedValue(null),
      set: vi.fn(),
      save: vi.fn(),
    } as any);
    const prefs = await loadDiveListPrefs();
    expect(prefs.hiddenCols).toEqual(expect.arrayContaining(["temp", "suit", "cylinder"]));
    expect(prefs.colOrder.length).toBe(20);
  });

  it("migrates old format (no hiddenCols): infers hiddenCols from absent ids", async () => {
    const oldFormat = { sortKey: "nr" as const, sortDir: "asc" as const, colOrder: ["nr", "date", "depth"] };
    vi.mocked(store.load).mockResolvedValueOnce({
      get: vi.fn().mockResolvedValue(oldFormat),
      set: vi.fn(),
      save: vi.fn(),
    } as any);
    const prefs = await loadDiveListPrefs();
    // All 17 ids must be present in colOrder after migration
    expect(prefs.colOrder.length).toBe(ALL_COLS.length);
    // The 3 visible ones must come first
    expect(prefs.colOrder.slice(0, 3)).toEqual(["nr", "date", "depth"]);
    // hiddenCols = every id that wasn't in the saved colOrder
    const expectedHidden = ALL_COLS.map(c => c.id).filter(id => !oldFormat.colOrder.includes(id));
    expect(prefs.hiddenCols.sort()).toEqual(expectedHidden.sort());
  });

  it("returns saved prefs unchanged when hiddenCols is already present", async () => {
    const saved: DiveListPrefs = {
      sortKey: "depth", sortDir: "desc",
      colOrder: ["nr", "depth"],
      hiddenCols: ["date"],
    };
    vi.mocked(store.load).mockResolvedValueOnce({
      get: vi.fn().mockResolvedValue(saved),
      set: vi.fn(),
      save: vi.fn(),
    } as any);
    const prefs = await loadDiveListPrefs();
    expect(prefs).toEqual(saved);
  });
});

describe("saveDiveListPrefs", () => {
  afterEach(() => vi.resetAllMocks());

  it("writes diveList key and saves", async () => {
    const mockSet = vi.fn();
    const mockSave = vi.fn().mockResolvedValue(undefined);
    vi.mocked(store.load).mockResolvedValueOnce({
      get: vi.fn(),
      set: mockSet,
      save: mockSave,
    } as any);
    const prefs: DiveListPrefs = {
      sortKey: "depth", sortDir: "asc",
      colOrder: ["nr", "depth"],
      hiddenCols: ["date"],
    };
    await saveDiveListPrefs(prefs);
    expect(mockSet).toHaveBeenCalledWith("diveList", prefs);
    expect(mockSave).toHaveBeenCalled();
  });
});

describe("loadLoggingPrefs", () => {
  afterEach(() => vi.resetAllMocks());

  it("returns the lowercased level reported by the backend", async () => {
    vi.mocked(invoke).mockResolvedValueOnce("DEBUG");
    const level = await loadLoggingPrefs();
    expect(level).toBe("debug");
    expect(invoke).toHaveBeenCalledWith("get_log_level");
  });
});

describe("applyLogLevel", () => {
  afterEach(() => vi.resetAllMocks());

  it("invokes set_log_level with the chosen level", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(undefined);
    await applyLogLevel("trace");
    expect(invoke).toHaveBeenCalledWith("set_log_level", { level: "trace" });
  });
});
