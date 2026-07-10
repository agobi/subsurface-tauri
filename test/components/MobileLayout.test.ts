// AI-generated (Claude)
// test/components/MobileLayout.test.ts
import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import MobileLayout from "$lib/components/MobileLayout.svelte";
import { app } from "$lib/stores/app.svelte.ts";
import { invoke } from "@tauri-apps/api/core";
import type { Dive, DiveSummary, Logbook, OpenResult } from "$lib/types.ts";

describe("MobileLayout", () => {
  beforeEach(() => {
    app.reset();
    Element.prototype.scrollTo = vi.fn();
  });

  it("always renders the bottom dive-list row, with no tab bar", () => {
    render(MobileLayout);
    expect(screen.getByTestId("mobile-panel-dives")).toBeInTheDocument();
    expect(screen.queryByRole("tablist")).not.toBeInTheDocument();
  });

  it("shows the empty state in the bottom row when there are no dives", () => {
    render(MobileLayout);
    expect(screen.getByTestId("mobile-empty-state")).toBeInTheDocument();
  });

  it("renders three swipeable top panels: info, profile, map", () => {
    render(MobileLayout);
    expect(screen.getByTestId("mobile-panel-info")).toBeInTheDocument();
    expect(screen.getByTestId("mobile-panel-profile")).toBeInTheDocument();
    expect(screen.getByTestId("mobile-panel-map")).toBeInTheDocument();
  });

  it("defaults to Profile as the active top panel", () => {
    render(MobileLayout);
    expect(screen.getByTestId("mobile-active-panel-label")).toHaveTextContent("Profile");
    expect(screen.getByTestId("mobile-dot-profile")).toHaveClass("active");
  });

  it("jumps to a panel when its dot is clicked", async () => {
    render(MobileLayout);
    await fireEvent.click(screen.getByTestId("mobile-dot-info"));
    expect(screen.getByTestId("mobile-active-panel-label")).toHaveTextContent("Info");
    expect(screen.getByTestId("mobile-dot-info")).toHaveClass("active");
    expect(screen.getByTestId("mobile-dot-profile")).not.toHaveClass("active");
  });

  it("shows a settings gear button in the persistent header", () => {
    render(MobileLayout);
    expect(screen.getByRole("button", { name: /settings/i })).toBeInTheDocument();
  });

  it("shows settings screen when gear button is clicked", async () => {
    render(MobileLayout);
    await fireEvent.click(screen.getByRole("button", { name: /settings/i }));
    expect(screen.getByTestId("mobile-settings-screen")).toBeInTheDocument();
  });

  it("returns to main view when back button is clicked in settings", async () => {
    render(MobileLayout);
    await fireEvent.click(screen.getByRole("button", { name: /settings/i }));
    await fireEvent.click(screen.getByRole("button", { name: /back/i }));
    expect(screen.queryByTestId("mobile-settings-screen")).not.toBeInTheDocument();
    expect(screen.getByTestId("mobile-panel-dives")).toBeInTheDocument();
  });

  it("exposes a splitter that resizes the top row, clamped to [0.2, 0.8]", async () => {
    const { container } = render(MobileLayout);
    const splitter = screen.getByTestId("mobile-splitter");
    expect(splitter).toHaveAttribute("role", "separator");

    // jsdom's getBoundingClientRect() returns an all-zero rect, so any positive
    // clientY drives the computed fraction to +Infinity, which clamps to the max (0.8).
    // This mirrors the existing loose-assertion pattern in QuadrantGrid.test.ts, which
    // works around the same jsdom limitation.
    await fireEvent.pointerDown(splitter, { pointerId: 1, clientY: 100 });
    await fireEvent.pointerMove(splitter, { pointerId: 1, clientY: 900 });
    await fireEvent.pointerUp(splitter, { pointerId: 1, clientY: 900 });

    const topRow = container.querySelector(".top-row") as HTMLElement;
    expect(topRow.style.flex).toBe("0 0 80%");
  });

  // jsdom does not propagate exceptions thrown inside event listeners back through
  // dispatchEvent synchronously — it reports them via a `window` "error" event instead.
  // So these regression tests listen for that event rather than wrapping fireEvent in
  // try/catch (which would report as "not thrown" even against buggy code).
  //
  // jsdom also has no `releasePointerCapture` on Element, so the crash site
  // (`target.releasePointerCapture?.(ev.pointerId)`) short-circuits via optional
  // chaining and never evaluates `ev.pointerId` unless the method actually exists.
  // Stubbing it makes the argument evaluation (and the bug) observable.
  function captureWindowErrors() {
    const errors: unknown[] = [];
    const handler = (e: ErrorEvent) => errors.push(e.error ?? e.message);
    window.addEventListener("error", handler);
    return { errors, stop: () => window.removeEventListener("error", handler) };
  }

  it("does not crash when a second pointerdown interrupts an in-progress drag", async () => {
    Element.prototype.releasePointerCapture = vi.fn();
    render(MobileLayout);
    const splitter = screen.getByTestId("mobile-splitter");
    const { errors, stop } = captureWindowErrors();

    await fireEvent.pointerDown(splitter, { pointerId: 1, clientY: 100 });
    // A stray second pointerdown (e.g. multi-touch) mid-drag calls the previous
    // drag's cleanup with no event — this used to throw trying to read ev.pointerId.
    await fireEvent.pointerDown(splitter, { pointerId: 2, clientY: 150 });

    stop();
    expect(errors).toEqual([]);
  });

  it("does not crash when the component unmounts mid-drag", async () => {
    Element.prototype.releasePointerCapture = vi.fn();
    const { unmount } = render(MobileLayout);
    const splitter = screen.getByTestId("mobile-splitter");
    const { errors, stop } = captureWindowErrors();
    // Svelte's effect-teardown error handling surfaces a synchronous throw inside
    // onDestroy as a Node-level unhandled promise rejection rather than a synchronous
    // throw here or a browser `window` "unhandledrejection" event (jsdom doesn't wire
    // that up), so listen on `process` instead.
    const rejections: unknown[] = [];
    const rejectionHandler = (reason: unknown) => rejections.push(reason);
    process.on("unhandledRejection", rejectionHandler);

    await fireEvent.pointerDown(splitter, { pointerId: 1, clientY: 100 });
    unmount();
    // Node only flags a promise as an unhandled rejection after the current macrotask
    // (not just microtasks) completes, so wait a real tick before asserting.
    await new Promise((resolve) => setTimeout(resolve, 0));

    stop();
    process.off("unhandledRejection", rejectionHandler);
    expect([...errors, ...rejections]).toEqual([]);
  });
});

describe("MobileLayout wired to data", () => {
  function makeDiveSummary(number: number, dateTime: string): DiveSummary {
    return { number, dateTime, durationSec: 1800, tags: [], cylinders: [], mediaCount: 0 };
  }
  function makeDive(number: number, dateTime: string): Dive {
    return { ...makeDiveSummary(number, dateTime), samples: [], events: [] };
  }

  const logbook: Logbook = {
    dives: [makeDiveSummary(1, "2024-01-01T08:00:00"), makeDiveSummary(2, "2024-02-02T09:00:00")],
    trips: [],
    sites: [],
    units: "METRIC",
  };

  beforeEach(async () => {
    app.reset();
    Element.prototype.scrollTo = vi.fn();
    vi.mocked(invoke)
      .mockResolvedValueOnce({ logbook, displayName: "test", recents: [], warnings: [] } as unknown as OpenResult)
      .mockResolvedValueOnce(makeDive(1, "2024-01-01T08:00:00"));
    await app.startup();
  });

  it("updates the (always-mounted) info panel when a different dive is tapped in the list", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(makeDive(2, "2024-02-02T09:00:00"));
    render(MobileLayout);
    expect(await screen.findByText("2024-01-01 08:00:00")).toBeInTheDocument();

    const rows = screen.getAllByTestId("dive-row");
    await fireEvent.click(rows[1]); // sorted by number asc by default: rows[1] is dive 2

    expect(await screen.findByText("2024-02-02 09:00:00")).toBeInTheDocument();
  });
});
