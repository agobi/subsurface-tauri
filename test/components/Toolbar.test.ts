// AI-generated (Claude)
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import Toolbar from "$lib/components/Toolbar.svelte";

describe("Toolbar", () => {
  it("renders primary actions", () => {
    render(Toolbar, { props: { onSearch: () => {} } });
    expect(screen.getByRole("button", { name: /add dive/i })).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/search/i)).toBeInTheDocument();
  });

  it("calls onSearch as the query changes", async () => {
    const onSearch = vi.fn();
    render(Toolbar, { props: { onSearch } });
    await fireEvent.input(screen.getByPlaceholderText(/search/i), { target: { value: "cave" } });
    expect(onSearch).toHaveBeenCalledWith("cave");
  });

  it("does not show Sync button when isCloud is false", () => {
    render(Toolbar, { props: { onSearch: () => {}, isCloud: false } });
    expect(screen.queryByRole("button", { name: /sync/i })).not.toBeInTheDocument();
  });

  it("shows Sync button when isCloud is true", () => {
    render(Toolbar, { props: { onSearch: () => {}, isCloud: true, onSync: vi.fn().mockResolvedValue(undefined) } });
    expect(screen.getByRole("button", { name: /sync/i })).toBeInTheDocument();
  });

  it("calls onSync when Sync button is clicked", async () => {
    const onSync = vi.fn().mockResolvedValue(undefined);
    render(Toolbar, { props: { onSearch: () => {}, isCloud: true, onSync } });
    await fireEvent.click(screen.getByRole("button", { name: /sync/i }));
    await waitFor(() => expect(onSync).toHaveBeenCalledOnce());
  });

  it("shows error banner when onSync rejects", async () => {
    const onSync = vi.fn().mockRejectedValue("Could not reach Subsurface Cloud. Check your connection.");
    render(Toolbar, { props: { onSearch: () => {}, isCloud: true, onSync } });
    await fireEvent.click(screen.getByRole("button", { name: /sync/i }));
    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent(/Could not reach Subsurface Cloud/i)
    );
  });

  it("dismisses error banner when × is clicked", async () => {
    const onSync = vi.fn().mockRejectedValue("Network error");
    render(Toolbar, { props: { onSearch: () => {}, isCloud: true, onSync } });
    await fireEvent.click(screen.getByRole("button", { name: /sync/i }));
    await waitFor(() => expect(screen.getByRole("alert")).toBeInTheDocument());
    await fireEvent.click(screen.getByRole("button", { name: /dismiss/i }));
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });
});
