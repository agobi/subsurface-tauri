// AI-generated (Claude)
import { describe, it, expect, vi, afterEach } from "vitest";
import * as store from "@tauri-apps/plugin-store";
import { loadDcConnections, saveDcConnection, type DcConnections } from "$lib/dcConnections.ts";

describe("loadDcConnections", () => {
  afterEach(() => vi.resetAllMocks());

  it("returns an empty map when settings.json has no dcConnections key", async () => {
    vi.mocked(store.load).mockResolvedValueOnce({
      get: vi.fn().mockResolvedValue(null),
      set: vi.fn(),
      save: vi.fn(),
    } as any);
    expect(await loadDcConnections()).toEqual({});
  });

  it("returns the saved map when present", async () => {
    const saved: DcConnections = {
      "Shearwater Perdix AI": { lastTransport: "BLE", addresses: { BLE: "AA:BB:CC:DD:EE:FF" } },
    };
    vi.mocked(store.load).mockResolvedValueOnce({
      get: vi.fn().mockResolvedValue(saved),
      set: vi.fn(),
      save: vi.fn(),
    } as any);
    expect(await loadDcConnections()).toEqual(saved);
  });
});

describe("saveDcConnection", () => {
  afterEach(() => vi.resetAllMocks());

  it("creates a new entry with the transport's address and sets lastTransport", async () => {
    const mockSet = vi.fn();
    const mockSave = vi.fn().mockResolvedValue(undefined);
    vi.mocked(store.load).mockResolvedValueOnce({
      get: vi.fn().mockResolvedValue(null),
      set: mockSet,
      save: mockSave,
    } as any);

    await saveDcConnection("Shearwater Perdix AI", "BLE", "AA:BB:CC:DD:EE:FF");

    expect(mockSet).toHaveBeenCalledWith("dcConnections", {
      "Shearwater Perdix AI": { lastTransport: "BLE", addresses: { BLE: "AA:BB:CC:DD:EE:FF" } },
    });
    expect(mockSave).toHaveBeenCalled();
  });

  it("keeps a different transport's previously-cached address when saving a new one", async () => {
    const existing: DcConnections = {
      "Shearwater Perdix AI": { lastTransport: "BLE", addresses: { BLE: "AA:BB:CC:DD:EE:FF" } },
    };
    const mockSet = vi.fn();
    vi.mocked(store.load).mockResolvedValueOnce({
      get: vi.fn().mockResolvedValue(existing),
      set: mockSet,
      save: vi.fn(),
    } as any);

    await saveDcConnection("Shearwater Perdix AI", "Serial", "/dev/ttyUSB0");

    expect(mockSet).toHaveBeenCalledWith("dcConnections", {
      "Shearwater Perdix AI": {
        lastTransport: "Serial",
        addresses: { BLE: "AA:BB:CC:DD:EE:FF", Serial: "/dev/ttyUSB0" },
      },
    });
  });

  it("replaces the address for the same transport rather than duplicating", async () => {
    const existing: DcConnections = {
      "Shearwater Perdix AI": { lastTransport: "Serial", addresses: { Serial: "/dev/ttyUSB0" } },
    };
    const mockSet = vi.fn();
    vi.mocked(store.load).mockResolvedValueOnce({
      get: vi.fn().mockResolvedValue(existing),
      set: mockSet,
      save: vi.fn(),
    } as any);

    await saveDcConnection("Shearwater Perdix AI", "Serial", "/dev/ttyUSB1");

    expect(mockSet).toHaveBeenCalledWith("dcConnections", {
      "Shearwater Perdix AI": { lastTransport: "Serial", addresses: { Serial: "/dev/ttyUSB1" } },
    });
  });

  it("does not overwrite the cached address with an empty string", async () => {
    const existing: DcConnections = {
      "Shearwater Perdix AI": { lastTransport: "BLE", addresses: { BLE: "AA:BB:CC:DD:EE:FF" } },
    };
    const mockSet = vi.fn();
    vi.mocked(store.load).mockResolvedValueOnce({
      get: vi.fn().mockResolvedValue(existing),
      set: mockSet,
      save: vi.fn(),
    } as any);

    await saveDcConnection("Shearwater Perdix AI", "USBHID", "");

    expect(mockSet).toHaveBeenCalledWith("dcConnections", {
      "Shearwater Perdix AI": { lastTransport: "USBHID", addresses: { BLE: "AA:BB:CC:DD:EE:FF" } },
    });
  });
});
