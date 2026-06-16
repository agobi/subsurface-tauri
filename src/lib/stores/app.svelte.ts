// AI-generated (Claude)
export type PanelKey = "info" | "profile" | "list" | "map";
export type Theme = "dark" | "light";
export type VisiblePanels = Record<PanelKey, boolean>;

const ALL_VISIBLE: VisiblePanels = { info: true, profile: true, list: true, map: true };

class AppStore {
  dives = $state<unknown[]>([]);
  selectedDiveId = $state<number | null>(null);
  visiblePanels = $state<VisiblePanels>({ ...ALL_VISIBLE });
  theme = $state<Theme>("dark");

  togglePanel(key: PanelKey) {
    const next = { ...this.visiblePanels, [key]: !this.visiblePanels[key] };
    if (!Object.values(next).some(Boolean)) return; // keep >= 1 visible
    this.visiblePanels = next;
  }

  toggleTheme() {
    this.theme = this.theme === "dark" ? "light" : "dark";
  }

  reset() {
    this.dives = [];
    this.selectedDiveId = null;
    this.visiblePanels = { ...ALL_VISIBLE };
    this.theme = "dark";
  }
}

export const app = new AppStore();
