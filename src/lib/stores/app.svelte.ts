// AI-generated (Claude)
import type { Logbook, Dive } from "$lib/types.ts";
import sample from "$lib/fixtures/logbook.sample.json";

// Use the generated (git-ignored) logbook.json if present; otherwise the committed sample.
const generated = import.meta.glob("../fixtures/logbook.json", { eager: true, import: "default" });
const initialLogbook = (Object.values(generated)[0] as Logbook | undefined) ?? (sample as Logbook);

export type PanelKey = "info" | "profile" | "list" | "map";
export type Theme = "dark" | "light";
export type VisiblePanels = Record<PanelKey, boolean>;

const ALL_VISIBLE: VisiblePanels = { info: true, profile: true, list: true, map: true };

class AppStore {
  logbook = $state<Logbook>(initialLogbook);
  selectedDiveId = $state<number | null>(initialLogbook.dives[0]?.number ?? null);
  visiblePanels = $state<VisiblePanels>({ ...ALL_VISIBLE });
  theme = $state<Theme>("dark");

  get dives(): Dive[] { return this.logbook.dives; }
  get selectedDive(): Dive | undefined { return this.logbook.dives.find((d) => d.number === this.selectedDiveId); }

  togglePanel(key: PanelKey) {
    const next = { ...this.visiblePanels, [key]: !this.visiblePanels[key] };
    if (!Object.values(next).some(Boolean)) return; // keep >= 1 visible
    this.visiblePanels = next;
  }

  toggleTheme() {
    this.theme = this.theme === "dark" ? "light" : "dark";
  }

  reset() {
    this.logbook = initialLogbook;
    this.selectedDiveId = initialLogbook.dives[0]?.number ?? null;
    this.visiblePanels = { ...ALL_VISIBLE };
    this.theme = "dark";
  }
}

export const app = new AppStore();
