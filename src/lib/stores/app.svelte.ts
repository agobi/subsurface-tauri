// AI-generated (Claude)
import { invoke } from "@tauri-apps/api/core";
import type { Logbook, Dive } from "$lib/types.ts";
import type { DiveListPrefs } from "$lib/prefs.ts";
import { DEFAULT_DIVE_LIST_PREFS, loadDiveListPrefs, saveDiveListPrefs } from "$lib/prefs.ts";
import type { ColId, ColDef, RenderCtx } from "$lib/diveListColumns.ts";
import { ALL_COLS } from "$lib/diveListColumns.ts";

export type PanelKey = "info" | "profile" | "list" | "map";
export type Theme = "dark" | "light" | "auto";
export type VisiblePanels = Record<PanelKey, boolean>;
export type Platform = "desktop" | "mobile";

const ALL_VISIBLE: VisiblePanels = { info: true, profile: true, list: true, map: true };
const EMPTY_LOGBOOK: Logbook = { dives: [], trips: [], sites: [], units: "METRIC" };

class AppStore {
  logbook = $state<Logbook>({ ...EMPTY_LOGBOOK });
  selectedDiveId = $state<number | null>(null);
  visiblePanels = $state<VisiblePanels>({ ...ALL_VISIBLE });
  theme = $state<Theme>("auto");
  platform = $state<Platform>("desktop");
  diveListPrefs = $state<DiveListPrefs>({
    ...DEFAULT_DIVE_LIST_PREFS,
    colOrder: [...DEFAULT_DIVE_LIST_PREFS.colOrder],
  });

  get dives(): Dive[] { return this.logbook.dives; }

  get selectedDive(): Dive | undefined {
    return this.logbook.dives.find((d) => d.number === this.selectedDiveId);
  }

  get isMobile(): boolean { return this.platform === "mobile"; }

  get visibleCols(): ColDef[] {
    return this.diveListPrefs.colOrder
      .map(id => ALL_COLS.find(c => c.id === id))
      .filter((c): c is ColDef => c != null);
  }

  get sortedDives(): Dive[] {
    const { sortKey, sortDir } = this.diveListPrefs;
    if (sortKey === "nr") return this.logbook.dives;
    const col = ALL_COLS.find(c => c.id === sortKey);
    if (!col) return this.logbook.dives;
    const ctx: RenderCtx = { sites: this.logbook.sites };
    return [...this.logbook.dives].sort((a, b) => {
      const ae = col.render(a, ctx) === "—";
      const be = col.render(b, ctx) === "—";
      if (ae && be) return 0;
      if (ae) return 1;
      if (be) return -1;
      const cmp = col.compare(a, b, ctx);
      return sortDir === "asc" ? cmp : -cmp;
    });
  }

  async startup(): Promise<void> {
    this.logbook = await invoke<Logbook>("startup_logbook");
    this.selectedDiveId = this.logbook.dives[0]?.number ?? null;
    this.diveListPrefs = await loadDiveListPrefs();
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

  setTheme(t: Theme) { this.theme = t; }

  setPlatform(p: Platform) { this.platform = p; }

  setSortCol(id: ColId) {
    const { sortKey, sortDir } = this.diveListPrefs;
    const newDir = id === sortKey ? (sortDir === "asc" ? "desc" : "asc") : "asc";
    this.diveListPrefs = { ...this.diveListPrefs, sortKey: id, sortDir: newDir };
    saveDiveListPrefs(this.diveListPrefs);
  }

  toggleColumn(id: ColId) {
    const { colOrder } = this.diveListPrefs;
    const next = colOrder.includes(id)
      ? colOrder.filter(c => c !== id)
      : [...colOrder, id];
    this.diveListPrefs = { ...this.diveListPrefs, colOrder: next };
    saveDiveListPrefs(this.diveListPrefs);
  }

  reset() {
    this.logbook = { ...EMPTY_LOGBOOK };
    this.selectedDiveId = null;
    this.visiblePanels = { ...ALL_VISIBLE };
    this.theme = "auto";
    this.platform = "desktop";
    this.diveListPrefs = { ...DEFAULT_DIVE_LIST_PREFS, colOrder: [...DEFAULT_DIVE_LIST_PREFS.colOrder] };
  }
}

export const app = new AppStore();
