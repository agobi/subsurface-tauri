// AI-generated (Claude)
import "@testing-library/jest-dom/vitest";
import { vi } from "vitest";

const emptyLogbook = { dives: [], trips: [], sites: [], units: "METRIC" };

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(emptyLogbook),
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
