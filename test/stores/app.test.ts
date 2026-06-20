// AI-generated (Claude)
import { describe, it, expect, beforeEach } from "vitest";
import { app } from "$lib/stores/app.svelte.ts";

describe("app store", () => {
  beforeEach(() => app.reset());

  it("starts with all four panels visible and auto theme", () => {
    expect(app.visiblePanels).toEqual({ info: true, profile: true, list: true, map: true });
    expect(app.theme).toBe("auto");
  });

  it("toggles a panel", () => {
    app.togglePanel("map");
    expect(app.visiblePanels.map).toBe(false);
  });

  it("never hides the last visible panel", () => {
    app.togglePanel("info");
    app.togglePanel("profile");
    app.togglePanel("list");
    app.togglePanel("map");
    expect(app.visiblePanels.map).toBe(true);
  });

  it("sets theme to light", () => {
    app.setTheme("light");
    expect(app.theme).toBe("light");
  });

  it("sets theme to auto", () => {
    app.setTheme("auto");
    expect(app.theme).toBe("auto");
  });

  describe("platform", () => {
    it("defaults to desktop", () => {
      expect(app.platform).toBe("desktop");
      expect(app.isMobile).toBe(false);
    });

    it("setPlatform('mobile') sets isMobile to true", () => {
      app.setPlatform("mobile");
      expect(app.platform).toBe("mobile");
      expect(app.isMobile).toBe(true);
    });

    it("reset() restores platform to desktop", () => {
      app.setPlatform("mobile");
      app.reset();
      expect(app.platform).toBe("desktop");
    });
  });
});
