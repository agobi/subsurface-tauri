// AI-generated (Claude)
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/svelte";
import MapPanel from "$lib/components/MapPanel.svelte";

describe("MapPanel", () => {
  it("shows the site name and a marker", () => {
    render(MapPanel, { props: { siteName: "Fenyes Forras", gps: { lat: 47.66, lon: 18.3 } } });
    expect(screen.getByText("Fenyes Forras")).toBeInTheDocument();
    expect(screen.getByTestId("map-marker")).toBeInTheDocument();
  });

  it("shows an empty state with no site", () => {
    render(MapPanel, { props: { siteName: undefined, gps: undefined } });
    expect(screen.getByText(/no site/i)).toBeInTheDocument();
  });
});
