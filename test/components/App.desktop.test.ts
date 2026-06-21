// AI-generated (Claude)
import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "../../src/App.svelte";
import { app } from "$lib/stores/app.svelte.ts";

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

    // Trigger menu:cloud-open
    const listenCalls = vi.mocked(listen).mock.calls;
    const cloudOpenCall = listenCalls.find((c) => c[0] === "menu:cloud-open");
    const callback = cloudOpenCall![1] as (e: { payload: unknown }) => void;
    callback({ payload: null });

    await waitFor(() =>
      expect(screen.getByRole("dialog", { name: /open cloud notebook/i })).toBeInTheDocument()
    );

    // Simulate dialog onSuccess being called (internal state change)
    // We call the dialog's success path by setting app state directly
    app.isCloudLogbook = true;
    // The title update happens via App.svelte's onSuccess handler when the dialog
    // reports success. Since we can't easily drive through the full dialog in this
    // integration test, we verify the Sync button appears when isCloudLogbook is true.
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /sync/i })).toBeInTheDocument()
    );
  });
});
