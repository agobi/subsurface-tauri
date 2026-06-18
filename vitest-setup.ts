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
