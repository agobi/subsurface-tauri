// AI-generated (Claude)
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import * as store from "@tauri-apps/plugin-store";
import DcDownloadDialog from "$lib/components/DcDownloadDialog.svelte";
import { app } from "$lib/stores/app.svelte.ts";

vi.mock("@tauri-apps/api/core");
vi.mock("@tauri-apps/api/event");

describe("DcDownloadDialog", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    vi.mocked(listen).mockResolvedValue(() => {});
    vi.mocked(store.load).mockResolvedValue({
      get: vi.fn().mockResolvedValue(null),
      set: vi.fn(),
      save: vi.fn(),
    } as any);
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "list_known_devices") return [];
      if (cmd === "list_dc_vendors") return ["Shearwater", "Suunto"];
      if (cmd === "list_dc_models") return [{ product: "Perdix", transports: ["BLE"] }];
      if (cmd === "list_serial_ports") return ["/dev/ttyUSB0"];
      if (cmd === "startup_logbook") return { logbook: { dives: [], trips: [], sites: [], units: "METRIC" }, displayName: "test", recents: [] };
      return null;
    });
  });

  it("goes straight to Add Device when there are no known devices", async () => {
    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });
    expect(await screen.findByText("Add Device")).toBeTruthy();
    expect(screen.queryByText("Select Device")).toBeNull();
  });

  it("calls list_dc_vendors on mount", async () => {
    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("list_dc_vendors"));
  });

  it("calls list_dc_models when vendor changes", async () => {
    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });
    const vendorSelect = await screen.findByLabelText("Vendor");
    await fireEvent.change(vendorSelect, { target: { value: "Shearwater" } });
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("list_dc_models", { vendor: "Shearwater" }));
  });

  it("calls cancel_dc_download on Cancel click during progress", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "list_known_devices") return [];
      if (cmd === "list_dc_vendors") return ["Shearwater"];
      if (cmd === "list_dc_models") return [{ product: "Perdix AI", transports: ["Serial"] }];
      if (cmd === "list_serial_ports") return ["/dev/ttyUSB0"];
      if (cmd === "start_dc_download") return new Promise(() => {}); // never resolves
      return null;
    });
    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });

    // Select vendor → loads models, defaults transport to Serial → loads serial ports
    const vendorSelect = await screen.findByLabelText("Vendor");
    await fireEvent.change(vendorSelect, { target: { value: "Shearwater" } });
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("list_dc_models", { vendor: "Shearwater" }));
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("list_serial_ports"));

    // Select the serial port so Download is enabled
    const portSelect = await screen.findByLabelText("Port");
    await fireEvent.change(portSelect, { target: { value: "/dev/ttyUSB0" } });

    // Start download → progress step
    await fireEvent.click(await screen.findByText("Download"));
    await vi.waitFor(() => expect(screen.getByText("Downloading…")).toBeTruthy());

    // Click Cancel on the progress step
    await fireEvent.click(await screen.findByText("Cancel"));

    expect(invoke).toHaveBeenCalledWith("cancel_dc_download");
  });

  it("shows the known-devices dropdown with a nickname when a remembered device has one", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "list_known_devices") return [{ vendor: "Shearwater", product: "Perdix AI", serial: "0001e240", nickname: "My Perdix" }];
      if (cmd === "list_dc_vendors") return ["Shearwater"];
      if (cmd === "list_dc_models") return [{ product: "Perdix AI", transports: ["BLE", "Serial"] }];
      return null;
    });
    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });
    expect(await screen.findByText("Select Device")).toBeTruthy();
    expect(await screen.findByText("Shearwater Perdix AI (My Perdix)")).toBeTruthy();
  });

  it("shows the serial when a remembered device has no nickname", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "list_known_devices") return [{ vendor: "Shearwater", product: "Perdix AI", serial: "0001e240", nickname: "" }];
      if (cmd === "list_dc_vendors") return ["Shearwater"];
      if (cmd === "list_dc_models") return [{ product: "Perdix AI", transports: ["BLE", "Serial"] }];
      return null;
    });
    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });
    expect(await screen.findByText("Shearwater Perdix AI (SN 0001e240)")).toBeTruthy();
  });

  it("defaults the dropdown to the most-recently-seen device (first in the list)", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "list_known_devices") return [
        { vendor: "Shearwater", product: "Perdix AI", serial: "00000002", nickname: "" },
        { vendor: "Shearwater", product: "Perdix", serial: "00000001", nickname: "" },
      ];
      if (cmd === "list_dc_vendors") return ["Shearwater"];
      if (cmd === "list_dc_models") return [{ product: "Perdix AI", transports: ["BLE"] }];
      return null;
    });
    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });
    const select = await screen.findByLabelText("Known device") as HTMLSelectElement;
    await vi.waitFor(() => expect(select.value).toBe("Shearwater Perdix AI 00000002"));
  });

  it("selecting a remembered device jumps to Connect with vendor/model fixed", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "list_known_devices") return [{ vendor: "Shearwater", product: "Perdix AI", serial: "0001e240", nickname: "" }];
      if (cmd === "list_dc_vendors") return ["Shearwater"];
      if (cmd === "list_dc_models") return [{ product: "Perdix AI", transports: ["BLE", "Serial"] }];
      return null;
    });
    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });
    await screen.findByText("Select Device");
    await fireEvent.click(await screen.findByText("Continue"));

    expect(await screen.findByText("Connect")).toBeTruthy();
    expect(screen.getByText("Shearwater Perdix AI")).toBeTruthy();
    expect(screen.queryByLabelText("Vendor")).toBeNull();
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("list_dc_models", { vendor: "Shearwater" }));
  });

  it("lets the user return to the known-devices list from a remembered device's setup screen", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "list_known_devices") return [{ vendor: "Shearwater", product: "Perdix AI", serial: "0001e240", nickname: "" }];
      if (cmd === "list_dc_vendors") return ["Shearwater"];
      if (cmd === "list_dc_models") return [{ product: "Perdix AI", transports: ["BLE", "Serial"] }];
      return null;
    });
    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });
    await screen.findByText("Select Device");
    await fireEvent.click(await screen.findByText("Continue"));
    await screen.findByText("Connect");

    await fireEvent.click(await screen.findByText("Use a different device"));
    expect(await screen.findByText("Select Device")).toBeTruthy();
  });

  it("prefills the cached transport and address for a remembered device", async () => {
    vi.mocked(store.load).mockResolvedValue({
      get: vi.fn().mockResolvedValue({
        "Shearwater Perdix AI 0001e240": { lastTransport: "Serial", addresses: { Serial: "/dev/ttyUSB0" } },
      }),
      set: vi.fn(),
      save: vi.fn(),
    } as any);
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "list_known_devices") return [{ vendor: "Shearwater", product: "Perdix AI", serial: "0001e240", nickname: "" }];
      if (cmd === "list_dc_vendors") return ["Shearwater"];
      if (cmd === "list_dc_models") return [{ product: "Perdix AI", transports: ["BLE", "Serial"] }];
      if (cmd === "list_serial_ports") return ["/dev/ttyUSB0"];
      return null;
    });

    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });
    await screen.findByText("Select Device");
    await fireEvent.click(await screen.findByText("Continue"));

    const transportSelect = await screen.findByLabelText("Transport") as HTMLSelectElement;
    await vi.waitFor(() => expect(transportSelect.value).toBe("Serial"));
    const portSelect = screen.getByLabelText("Port") as HTMLSelectElement;
    expect(portSelect.value).toBe("/dev/ttyUSB0");
  });

  it("saves the connection address keyed by serial so two devices of the same model don't collide", async () => {
    let devinfoCallback: ((e: { payload: unknown }) => void) | undefined;
    vi.mocked(listen).mockImplementation(async (event, cb) => {
      if (event === "dc-devinfo") devinfoCallback = cb as typeof devinfoCallback;
      return () => {};
    });
    const setSpy = vi.fn();
    vi.mocked(store.load).mockResolvedValue({
      get: vi.fn().mockResolvedValue(null),
      set: setSpy,
      save: vi.fn(),
    } as any);
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "list_known_devices") return [{ vendor: "Shearwater", product: "Perdix AI", serial: "0001e240", nickname: "" }];
      if (cmd === "list_dc_vendors") return ["Shearwater"];
      if (cmd === "list_dc_models") return [{ product: "Perdix AI", transports: ["Serial"] }];
      if (cmd === "list_serial_ports") return ["/dev/ttyUSB0"];
      return null;
    });

    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });
    await screen.findByText("Select Device");
    await fireEvent.click(await screen.findByText("Continue"));
    await screen.findByText("Connect");
    const portSelect = await screen.findByLabelText("Port");
    await fireEvent.change(portSelect, { target: { value: "/dev/ttyUSB0" } });
    await fireEvent.click(await screen.findByText("Download"));

    await vi.waitFor(() => expect(devinfoCallback).toBeDefined());
    devinfoCallback!({ payload: { model: 6, firmware: 1, serial: 0x0001e241 } });

    await vi.waitFor(() =>
      expect(setSpy).toHaveBeenCalledWith("dcConnections", expect.objectContaining({
        "Shearwater Perdix AI 0001e241": expect.anything(),
      })),
    );
  });

  it("shows the review step with buffered dives after dc-complete", async () => {
    let completeCb: ((e: { payload: unknown }) => void) | undefined;
    vi.mocked(listen).mockImplementation(async (event, cb) => {
      if (event === "dc-complete") completeCb = cb as typeof completeCb;
      return () => {};
    });
    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });
    await vi.waitFor(() => expect(completeCb).toBeDefined());
    completeCb!({
      payload: {
        dives: [{ date: "2026-06-14T08:00:00", durationSec: 1800, maxDepthM: 12.6 }],
        skipped: 1,
        cancelled: false,
      },
    });
    expect(await screen.findByText("Review Downloaded Dives")).toBeTruthy();
  });

  it("renders the review summary with no stray whitespace before the period", async () => {
    let completeCb: ((e: { payload: unknown }) => void) | undefined;
    vi.mocked(listen).mockImplementation(async (event, cb) => {
      if (event === "dc-complete") completeCb = cb as typeof completeCb;
      return () => {};
    });
    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });
    await vi.waitFor(() => expect(completeCb).toBeDefined());
    completeCb!({
      payload: {
        dives: [
          { date: "2026-06-14T08:00:00", durationSec: 1800, maxDepthM: 12.6 },
          { date: "2026-06-13T08:00:00", durationSec: 900, maxDepthM: 8.0 },
          { date: "2026-06-13T10:00:00", durationSec: 600, maxDepthM: 5.0 },
        ],
        skipped: 0,
        cancelled: false,
      },
    });
    await screen.findByText("Review Downloaded Dives");
    expect(document.body.textContent).toContain("3 new dives.");
    expect(document.body.textContent).not.toContain("dives .");
    expect(document.body.textContent).not.toContain("dives  ,");
  });

  it("still commits (with no dives) to advance the fingerprint cutoff when nothing new was found", async () => {
    let completeCb: ((e: { payload: unknown }) => void) | undefined;
    vi.mocked(listen).mockImplementation(async (event, cb) => {
      if (event === "dc-complete") completeCb = cb as typeof completeCb;
      return () => {};
    });
    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });
    await vi.waitFor(() => expect(completeCb).toBeDefined());
    completeCb!({
      payload: { dives: [], skipped: 50, cancelled: false },
    });
    await vi.waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("commit_dc_download", { selectedIndices: [] })
    );
  });

  it("shows the review step for dives fetched before a cancel, instead of discarding them", async () => {
    let completeCb: ((e: { payload: unknown }) => void) | undefined;
    vi.mocked(listen).mockImplementation(async (event, cb) => {
      if (event === "dc-complete") completeCb = cb as typeof completeCb;
      return () => {};
    });
    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });
    await vi.waitFor(() => expect(completeCb).toBeDefined());
    completeCb!({
      payload: {
        dives: [{ date: "2026-06-14T08:00:00", durationSec: 1800, maxDepthM: 12.6 }],
        skipped: 0,
        cancelled: true,
      },
    });
    expect(await screen.findByText("Save 1 dive to logbook")).toBeTruthy();
  });

  it("applies the reloaded logbook to the app store after saving", async () => {
    let completeCb: ((e: { payload: unknown }) => void) | undefined;
    vi.mocked(listen).mockImplementation(async (event, cb) => {
      if (event === "dc-complete") completeCb = cb as typeof completeCb;
      return () => {};
    });
    const reloadedLogbook = {
      dives: [{ number: 1, dateTime: "2026-06-14T08:00:00", durationSec: 1800 }],
      trips: [],
      sites: [],
      units: "METRIC",
    };
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "list_known_devices") return [];
      if (cmd === "list_dc_vendors") return ["Shearwater"];
      if (cmd === "list_dc_models") return [{ product: "Perdix", transports: ["BLE"] }];
      if (cmd === "commit_dc_download") return 1;
      if (cmd === "startup_logbook") {
        return { logbook: reloadedLogbook, displayName: "after-save", recents: [] };
      }
      return null;
    });

    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });
    await vi.waitFor(() => expect(completeCb).toBeDefined());
    completeCb!({
      payload: {
        dives: [{ date: "2026-06-14T08:00:00", durationSec: 1800, maxDepthM: 12.6 }],
        skipped: 0,
        cancelled: false,
      },
    });

    const saveButton = await screen.findByText("Save 1 dive to logbook");
    await fireEvent.click(saveButton);

    await vi.waitFor(() => expect(app.displayName).toBe("after-save"));
    expect(app.logbook.dives).toEqual(reloadedLogbook.dives);
  });

  it("lets the user deselect a dive and only commits the remaining selected indices", async () => {
    let completeCb: ((e: { payload: unknown }) => void) | undefined;
    vi.mocked(listen).mockImplementation(async (event, cb) => {
      if (event === "dc-complete") completeCb = cb as typeof completeCb;
      return () => {};
    });
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "list_known_devices") return [];
      if (cmd === "list_dc_vendors") return ["Shearwater"];
      if (cmd === "list_dc_models") return [{ product: "Perdix", transports: ["BLE"] }];
      if (cmd === "commit_dc_download") return 1;
      if (cmd === "startup_logbook") {
        return { logbook: { dives: [], trips: [], sites: [], units: "METRIC" }, displayName: "x", recents: [] };
      }
      return null;
    });

    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });
    await vi.waitFor(() => expect(completeCb).toBeDefined());
    completeCb!({
      payload: {
        dives: [
          { date: "2026-06-14T08:00:00", durationSec: 1800, maxDepthM: 12.6 },
          { date: "2026-06-11T16:32:00", durationSec: 199, maxDepthM: 3.6 },
        ],
        skipped: 0,
        cancelled: false,
      },
    });

    await screen.findByText("Save 2 dives to logbook");
    const checkboxes = screen.getAllByRole("checkbox");
    expect(checkboxes).toHaveLength(2);
    expect(checkboxes.every((c) => (c as HTMLInputElement).checked)).toBe(true);

    // Deselect the short, suspicious 199s segment.
    await fireEvent.click(checkboxes[1]);

    const saveButton = await screen.findByText("Save 1 dive to logbook");
    await fireEvent.click(saveButton);

    await vi.waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("commit_dc_download", { selectedIndices: [0] })
    );
  });

  it("transitions status label: Connecting… → Connected → Dive", async () => {
    let devinfoCallback: ((e: { payload: unknown }) => void) | undefined;
    let diveCallback: ((e: { payload: unknown }) => void) | undefined;

    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "list_known_devices") return [];
      if (cmd === "list_dc_vendors") return ["Shearwater"];
      if (cmd === "list_dc_models") return [{ product: "Perdix AI", transports: ["Serial"] }];
      if (cmd === "list_serial_ports") return ["/dev/ttyUSB0"];
      if (cmd === "start_dc_download") return new Promise<void>(() => {});
      return null;
    });
    vi.mocked(listen).mockImplementation(async (event, cb) => {
      if (event === "dc-devinfo") devinfoCallback = cb as typeof devinfoCallback;
      if (event === "dc-dive") diveCallback = cb as typeof diveCallback;
      return () => {};
    });

    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });

    // Select vendor → loads models, defaults transport to Serial → loads serial ports
    const vendorSelect = await screen.findByLabelText("Vendor");
    await fireEvent.change(vendorSelect, { target: { value: "Shearwater" } });
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("list_dc_models", { vendor: "Shearwater" }));
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("list_serial_ports"));

    // Select the serial port
    const portSelect = await screen.findByLabelText("Port");
    await fireEvent.change(portSelect, { target: { value: "/dev/ttyUSB0" } });

    // Start download → progress step
    await fireEvent.click(await screen.findByText("Download"));
    await vi.waitFor(() => expect(screen.getByText("Downloading…")).toBeTruthy());
    expect(screen.getByText("Connecting…")).toBeTruthy();

    // Fire dc-devinfo → status updates to "Connected: …"
    await vi.waitFor(() => expect(devinfoCallback).toBeDefined());
    devinfoCallback!({ payload: { model: 27, firmware: 1, serial: 12345 } });
    await vi.waitFor(() => expect(screen.getByText("Connected: Shearwater Perdix AI")).toBeTruthy());

    // Fire dc-dive with date → status updates to "Dive N: date"
    await vi.waitFor(() => expect(diveCallback).toBeDefined());
    diveCallback!({ payload: { diveNumber: 1, date: "2026-02-15", added: true } });
    await vi.waitFor(() => expect(screen.getByText("Dive 1: 2026-02-15")).toBeTruthy());
  });

  it("shows a BLE scan failure on the setup screen instead of swallowing it", async () => {
    let errorCallback: ((e: { payload: unknown }) => void) | undefined;

    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "list_known_devices") return [];
      if (cmd === "list_dc_vendors") return ["Shearwater"];
      if (cmd === "list_dc_models") return [{ product: "Perdix AI", transports: ["BLE"] }];
      if (cmd === "scan_ble_devices") return null;
      return null;
    });
    vi.mocked(listen).mockImplementation(async (event, cb) => {
      if (event === "dc-error") errorCallback = cb as typeof errorCallback;
      return () => {};
    });

    render(DcDownloadDialog, { props: { open: true, onClose: () => {} } });

    const vendorSelect = await screen.findByLabelText("Vendor");
    await fireEvent.change(vendorSelect, { target: { value: "Shearwater" } });
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("list_dc_models", { vendor: "Shearwater" }));

    await fireEvent.click(await screen.findByText("Scan"));
    await vi.waitFor(() => expect(errorCallback).toBeDefined());
    errorCallback!({ payload: { message: "no BLE adapter found" } });

    expect(await screen.findByText("no BLE adapter found")).toBeTruthy();
    // Must stay on the setup screen, not jump away with no context.
    expect(screen.getByText("Add Device")).toBeTruthy();
  });
});
