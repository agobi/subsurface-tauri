// AI-generated (Claude)
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/svelte";
import StatusBar from "$lib/components/StatusBar.svelte";

describe("StatusBar", () => {
  it("shows dive count and deco model", () => {
    render(StatusBar, { props: { diveCount: 42, decoModel: "GF 55/85", synced: true } });
    expect(screen.getByText(/42 dives/i)).toBeInTheDocument();
    expect(screen.getByText(/GF 55\/85/)).toBeInTheDocument();
    expect(screen.getByText(/synced/i)).toBeInTheDocument();
  });
});
