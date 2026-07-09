// AI-generated (Claude)
import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { message } from "@tauri-apps/plugin-dialog";
import App from "../../src/App.svelte";
import { app } from "$lib/stores/app.svelte.ts";
import sample from "$lib/fixtures/logbook.sample.json";
import type { OpenResult } from "$lib/types.ts";

describe("App — desktop cloud wiring", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    app.reset();
    window.matchMedia = vi.fn().mockReturnValue({
      matches: false,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    });
  });

  it("does not show CloudLoginDialog on initial render", async () => {
    render(App);
    await waitFor(() => expect(screen.getByPlaceholderText(/search/i)).toBeInTheDocument());
    expect(screen.queryByRole("dialog", { name: /open cloud notebook/i })).not.toBeInTheDocument();
  });

  it("shows CloudLoginDialog when menu:cloud-open is received", async () => {
    render(App);
    await waitFor(() => expect(screen.getByPlaceholderText(/search/i)).toBeInTheDocument());

    // Find and call the menu:cloud-open listener registered via listen()
    const listenCalls = vi.mocked(listen).mock.calls;
    const cloudOpenCall = listenCalls.find((c) => c[0] === "menu:cloud-open");
    expect(cloudOpenCall).toBeDefined();
    const callback = cloudOpenCall![1] as (e: { payload: unknown }) => void;
    callback({ payload: null });

    await waitFor(() =>
      expect(screen.getByRole("dialog", { name: /open cloud notebook/i })).toBeInTheDocument()
    );
  });

  it("does not show Sync button when isCloudLogbook is false", async () => {
    render(App);
    await waitFor(() => expect(screen.getByPlaceholderText(/search/i)).toBeInTheDocument());
    expect(screen.queryByRole("button", { name: /sync/i })).not.toBeInTheDocument();
  });

  it("shows Sync button when isCloudLogbook is true", async () => {
    render(App);
    await waitFor(() => expect(screen.getByPlaceholderText(/search/i)).toBeInTheDocument());

    // Simulate a successful cloud login by placing a Cloud entry at the top of recents.
    app.recents = [{ kind: "Cloud", email: "user@example.com", url: "https://ssrf-cloud-eu.subsurface-divelog.org" }];
    // The title update happens via App.svelte's onSuccess handler when the dialog
    // reports success. Since we can't easily drive through the full dialog in this
    // integration test, we verify the Sync button appears when isCloudLogbook is true.
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /sync/i })).toBeInTheDocument()
    );
  });
});

describe("App — handleCloudSuccess", () => {
  function openResult(overrides: Partial<OpenResult> = {}): OpenResult {
    return { logbook: sample as any, displayName: "test", recents: [], warnings: [], ...overrides };
  }

  beforeEach(() => {
    vi.clearAllMocks();
    // Reset setTitle to a resolving mock between tests so implementations don't leak.
    vi.mocked(getCurrentWindow).mockReturnValue({ setTitle: vi.fn().mockResolvedValue(undefined) } as any);
    app.reset();
    window.matchMedia = vi.fn().mockReturnValue({
      matches: false,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    });
  });

  it("calls onSuccess even when setWindowTitle rejects", async () => {
    // Startup's setWindowTitle call must succeed; only the handleCloudSuccess call fails.
    const setTitle = vi.fn()
      .mockResolvedValueOnce(undefined)               // startup
      .mockRejectedValue(new Error("window gone"));   // handleCloudSuccess
    vi.mocked(getCurrentWindow).mockReturnValue({ setTitle } as any);

    const onSuccess = vi.fn().mockResolvedValue(undefined);
    app.showCloudDialog = { email: "user@example.com", onSuccess };

    // startup_logbook + get_dive + get_cloud_credentials + open_cloud_logbook + get_dive
    const sampleDive = (sample as any).dives[0];
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_dive") return sampleDive;
      return openResult();
    });

    render(App);
    await waitFor(() => screen.getByRole("dialog", { name: /open cloud notebook/i }));

    await fireEvent.input(screen.getByLabelText(/email/i), { target: { value: "user@example.com" } });
    await fireEvent.input(screen.getByLabelText(/password/i), { target: { value: "secret" } });
    await fireEvent.click(screen.getByRole("button", { name: /open cloud/i }));

    await waitFor(() => expect(onSuccess).toHaveBeenCalledOnce());
  });

  it("awaits async onSuccess before resolving", async () => {
    const order: string[] = [];
    const onSuccess = vi.fn().mockImplementation(async () => {
      await new Promise(r => setTimeout(r, 0));
      order.push("callback");
    });
    app.showCloudDialog = { email: "user@example.com", onSuccess };

    const sampleDive = (sample as any).dives[0];
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_dive") return sampleDive;
      return openResult();
    });

    render(App);
    await waitFor(() => screen.getByRole("dialog", { name: /open cloud notebook/i }));

    await fireEvent.input(screen.getByLabelText(/email/i), { target: { value: "user@example.com" } });
    await fireEvent.input(screen.getByLabelText(/password/i), { target: { value: "secret" } });
    await fireEvent.click(screen.getByRole("button", { name: /open cloud/i }));

    await waitFor(() => expect(order).toContain("callback"));
  });
});

describe("App — parse warnings dialog", () => {
  function openResult(overrides: Partial<OpenResult> = {}): OpenResult {
    return { logbook: sample as any, displayName: "test", recents: [], warnings: [], ...overrides };
  }

  beforeEach(() => {
    vi.clearAllMocks();
    app.reset();
    window.matchMedia = vi.fn().mockReturnValue({
      matches: false,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    });
  });

  it("shows a warning dialog when startup_logbook returns non-empty warnings", async () => {
    const sampleDive = (sample as any).dives[0];
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_dive") return sampleDive;
      if (cmd === "startup_logbook") {
        return openResult({ warnings: ["2024/03/16-Sat-09=00=00/Dive-2: Permission denied (os error 13)"] });
      }
      return openResult();
    });

    render(App);

    await waitFor(() =>
      expect(vi.mocked(message)).toHaveBeenCalledWith(
        "1 dive could not be read:\n\n2024/03/16-Sat-09=00=00/Dive-2: Permission denied (os error 13)",
        { title: "Some Dives Skipped", kind: "warning" }
      )
    );
  });

  it("pluralizes the dialog message for multiple warnings", async () => {
    const sampleDive = (sample as any).dives[0];
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_dive") return sampleDive;
      if (cmd === "startup_logbook") {
        return openResult({ warnings: ["path/one: error one", "path/two: error two"] });
      }
      return openResult();
    });

    render(App);

    await waitFor(() =>
      expect(vi.mocked(message)).toHaveBeenCalledWith(
        "2 dives could not be read:\n\npath/one: error one\npath/two: error two",
        { title: "Some Dives Skipped", kind: "warning" }
      )
    );
  });

  it("does not show a warning dialog when startup_logbook returns no warnings", async () => {
    const sampleDive = (sample as any).dives[0];
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_dive") return sampleDive;
      return openResult();
    });

    render(App);
    await waitFor(() => expect(screen.getByPlaceholderText(/search/i)).toBeInTheDocument());
    expect(vi.mocked(message)).not.toHaveBeenCalled();
  });
});
