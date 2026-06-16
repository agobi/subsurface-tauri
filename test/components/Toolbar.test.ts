// AI-generated (Claude)
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
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
});
