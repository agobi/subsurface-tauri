// AI-generated (Claude)
import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import TitleBar from "$lib/components/TitleBar.svelte";
import { app } from "$lib/stores/app.svelte.ts";

describe("TitleBar", () => {
  it("shows the document name", () => {
    render(TitleBar, { props: { fileName: "logbook.ssrf" } });
    expect(screen.getByText("logbook.ssrf")).toBeInTheDocument();
  });

  it("theme toggle flips the store theme", async () => {
    app.reset();
    render(TitleBar, { props: { fileName: "x" } });
    await fireEvent.click(screen.getByRole("button", { name: /theme/i }));
    expect(app.theme).toBe("light");
  });
});
