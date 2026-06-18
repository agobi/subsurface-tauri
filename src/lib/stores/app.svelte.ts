// AI-generated (Claude)
import { invoke } from "@tauri-apps/api/core";
import type { Logbook, Dive } from "$lib/types.ts";

export type PanelKey = "info" | "profile" | "list" | "map";
export type Theme = "dark" | "light";
export type VisiblePanels = Record<PanelKey, boolean>;

const ALL_VISIBLE: VisiblePanels = { info: true, profile: true, list: true, map: true };
const EMPTY_LOGBOOK: Logbook = { dives: [], trips: [], sites: [], units: "METRIC" };

class AppStore {
  logbook = $state<Logbook>({ ...EMPTY_LOGBOOK });
  selectedDiveId = $state<number | null>(null);
  visiblePanels = $state<VisiblePanels>({ ...ALL_VISIBLE });
  theme = $state<Theme>("dark");

  get dives(): Dive[] { return this.logbook.dives; }
  get selectedDive(): Dive | undefined {
    return this.logbook.dives.find((d) => d.number === this.selectedDiveId);
  }

  async startup(): Promise<void> {
    this.logbook = await invoke<Logbook>("startup_logbook");
    this.selectedDiveId = this.logbook.dives[0]?.number ?? null;
  }

  async open(root: string): Promise<void> {
    this.logbook = await invoke<Logbook>("open_logbook", { root });
    this.selectedDiveId = this.logbook.dives[0]?.number ?? null;
  }

  async newLogbook(root: string): Promise<void> {
    this.logbook = await invoke<Logbook>("new_logbook", { root });
    this.selectedDiveId = this.logbook.dives[0]?.number ?? null;
  }

  togglePanel(key: PanelKey) {
    const next = { ...this.visiblePanels, [key]: !this.visiblePanels[key] };
    if (!Object.values(next).some(Boolean)) return;
    this.visiblePanels = next;
  }

  setPanelVisible(key: PanelKey, visible: boolean) {
    this.visiblePanels = { ...this.visiblePanels, [key]: visible };
  }

  toggleTheme() {
    this.theme = this.theme === "dark" ? "light" : "dark";
  }

  reset() {
    this.logbook = { ...EMPTY_LOGBOOK };
    this.selectedDiveId = null;
    this.visiblePanels = { ...ALL_VISIBLE };
    this.theme = "dark";
  }
}

export const app = new AppStore();
