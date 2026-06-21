// AI-generated (Claude)
import "@testing-library/jest-dom/vitest";
import { vi } from "vitest";

const emptyOpenResult = {
  logbook: { dives: [], trips: [], sites: [], units: "METRIC" },
  displayName: "",
  recents: [],
};

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(emptyOpenResult),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn().mockResolvedValue(null),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: vi.fn().mockReturnValue({ setTitle: vi.fn() }),
}));

vi.mock("@tauri-apps/plugin-store", () => {
  const mockGet = vi.fn().mockResolvedValue(null);
  const mockSet = vi.fn().mockResolvedValue(undefined);
  const mockSave = vi.fn().mockResolvedValue(undefined);

  return {
    load: vi.fn().mockResolvedValue({
      get: mockGet,
      set: mockSet,
      save: mockSave,
    }),
  };
});

vi.mock("@tauri-apps/plugin-os", () => ({
  platform: vi.fn().mockResolvedValue("macos"),
}));
