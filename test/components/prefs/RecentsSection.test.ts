// AI-generated (Claude)
import { render, screen, fireEvent } from "@testing-library/svelte";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import RecentsSection from "$lib/components/prefs/RecentsSection.svelte";
import { app } from "$lib/stores/app.svelte.ts";

const localRecents = [{ kind: "Local" as const, path: "/some/local/path" }];

describe("RecentsSection", () => {
  beforeEach(() => {
    app.reset();
    vi.mocked(invoke).mockReset();
  });

  it("loads recents on mount via get_recents", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(localRecents);
    render(RecentsSection);
    expect(await screen.findByText("path")).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith("get_recents");
  });

  it("shows an empty message when there are no recents", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([]);
    render(RecentsSection);
    expect(await screen.findByText("No recent logbooks")).toBeInTheDocument();
  });

  it("removes an entry when its remove button is clicked", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(localRecents);
    render(RecentsSection);
    await screen.findByText("path");

    vi.mocked(invoke).mockResolvedValueOnce([]);
    await fireEvent.click(screen.getByLabelText("Remove path"));
    expect(invoke).toHaveBeenCalledWith("remove_recent", { index: 0 });
  });

  it("clears all entries when Clear All is clicked", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(localRecents);
    render(RecentsSection);
    await screen.findByText("path");

    vi.mocked(invoke).mockResolvedValueOnce([]);
    await fireEvent.click(screen.getByText("Clear All"));
    expect(invoke).toHaveBeenCalledWith("clear_recents");
  });
});
