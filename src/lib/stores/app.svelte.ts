// AI-generated (Claude)
import { invoke } from "@tauri-apps/api/core";
import type { Logbook, Dive, DiveSummary, OpenResult, RecentEntry } from "$lib/types.ts";
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
  selectedDive = $state<Dive | null>(null);
  selectedDiveLoading = $state(false);
  visiblePanels = $state<VisiblePanels>({ ...ALL_VISIBLE });
  theme = $state<Theme>("auto");
  platform = $state<Platform>("desktop");
  get isCloudLogbook(): boolean { return this.recents[0]?.kind === "Cloud"; }
  displayName = $state("");
  recents = $state<RecentEntry[]>([]);
  showCloudDialog = $state<{ email: string; message?: string; onSuccess?: () => void } | false>(false);
  showDcDialog = $state(false);
  diveListPrefs = $state<DiveListPrefs>({
    ...DEFAULT_DIVE_LIST_PREFS,
    colOrder: [...DEFAULT_DIVE_LIST_PREFS.colOrder],
    hiddenCols: [...DEFAULT_DIVE_LIST_PREFS.hiddenCols],
  });

  get dives(): DiveSummary[] { return this.logbook.dives; }

  get isMobile(): boolean { return this.platform === "mobile"; }

  get visibleCols(): ColDef[] {
    const hidden = new Set(this.diveListPrefs.hiddenCols);
    return this.diveListPrefs.colOrder
      .filter(id => !hidden.has(id))
      .map(id => ALL_COLS.find(c => c.id === id))
      .filter((c): c is ColDef => c != null);
  }

  get sortedDives(): DiveSummary[] {
    const { sortKey, sortDir } = this.diveListPrefs;
    if (sortKey === "nr") {
      return [...this.logbook.dives].sort((a, b) => sortDir === "asc" ? a.number - b.number : b.number - a.number);
    }
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

  async selectDive(number: number | null): Promise<void> {
    this.selectedDiveId = number;
    this.selectedDive = null;
    if (number === null) return;
    this.selectedDiveLoading = true;
    try {
      this.selectedDive = await invoke<Dive>("get_dive", { number });
    } finally {
      this.selectedDiveLoading = false;
    }
  }

  async startup(): Promise<void> {
    const result = await invoke<OpenResult>("startup_logbook");
    this.logbook = result.logbook;
    this.displayName = result.displayName;
    this.recents = result.recents;
    this.diveListPrefs = await loadDiveListPrefs();
    await this.selectDive(result.logbook.dives[0]?.number ?? null);
  }

  async open(root: string): Promise<void> {
    const result = await invoke<OpenResult>("open_logbook", { root });
    this.logbook = result.logbook;
    this.displayName = result.displayName;
    this.recents = result.recents;
    await this.selectDive(result.logbook.dives[0]?.number ?? null);
  }

  async newLogbook(root: string): Promise<void> {
    const result = await invoke<OpenResult>("new_logbook", { root });
    this.logbook = result.logbook;
    this.displayName = result.displayName;
    this.recents = result.recents;
    await this.selectDive(result.logbook.dives[0]?.number ?? null);
  }

  async openRecentCloud(email: string): Promise<void> {
    const result = await invoke<OpenResult>("open_recent_cloud_logbook", { email });
    this.logbook = result.logbook;
    this.displayName = result.displayName;
    this.recents = result.recents;
    await this.selectDive(result.logbook.dives[0]?.number ?? null);
  }

  async openCloud(email: string, password: string): Promise<void> {
    const result = await invoke<OpenResult>("open_cloud_logbook", { email, password });
    this.logbook = result.logbook;
    this.displayName = result.displayName;
    this.recents = result.recents;
    await this.selectDive(result.logbook.dives[0]?.number ?? null);
  }

  async syncCloud(): Promise<void> {
    const result = await invoke<OpenResult>("sync_cloud_logbook");
    const currentId = this.selectedDiveId;
    this.logbook = result.logbook;
    this.displayName = result.displayName;
    this.recents = result.recents;
    const stillExists = result.logbook.dives.some((d) => d.number === currentId);
    await this.selectDive(stillExists ? currentId : (result.logbook.dives[0]?.number ?? null));
  }

  async openRecent(entry: RecentEntry): Promise<void> {
    if (entry.kind === "Local") {
      await this.open(entry.path);
    } else {
      await this.openRecentCloud(entry.email);
    }
  }

  togglePanel(key: PanelKey) {
    const next = { ...this.visiblePanels, [key]: !this.visiblePanels[key] };
    if (!Object.values(next).some(Boolean)) return;
    this.visiblePanels = next;
  }

  setTheme(t: Theme) { this.theme = t; }

  setPlatform(p: Platform) { this.platform = p; }

  async setSortCol(id: ColId) {
    const { sortKey, sortDir } = this.diveListPrefs;
    const newDir = id === sortKey ? (sortDir === "asc" ? "desc" : "asc") : "asc";
    this.diveListPrefs = { ...this.diveListPrefs, sortKey: id, sortDir: newDir };
    try {
      await saveDiveListPrefs(this.diveListPrefs);
    } catch (e) {
      console.error("Failed to persist sort preference:", e);
    }
  }

  async toggleColumn(id: ColId) {
    const { hiddenCols } = this.diveListPrefs;
    const next = hiddenCols.includes(id)
      ? hiddenCols.filter(c => c !== id)
      : [...hiddenCols, id];
    this.diveListPrefs = { ...this.diveListPrefs, hiddenCols: next };
    try {
      await saveDiveListPrefs(this.diveListPrefs);
    } catch (e) {
      console.error("Failed to persist column preference:", e);
    }
  }

  async reorderColumn(fromId: ColId, toId: ColId) {
    if (fromId === toId) return;
    const { colOrder } = this.diveListPrefs;
    const fromIdx = colOrder.indexOf(fromId);
    const toIdx = colOrder.indexOf(toId);
    if (fromIdx === -1 || toIdx === -1) return;
    const next = [...colOrder];
    next.splice(fromIdx, 1);
    next.splice(toIdx, 0, fromId);
    this.diveListPrefs = { ...this.diveListPrefs, colOrder: next };
    try {
      await saveDiveListPrefs(this.diveListPrefs);
    } catch (e) {
      console.error("Failed to persist column preference:", e);
    }
  }

  reset() {
    this.logbook = { ...EMPTY_LOGBOOK };
    this.selectedDiveId = null;
    this.selectedDive = null;
    this.selectedDiveLoading = false;
    this.visiblePanels = { ...ALL_VISIBLE };
    this.theme = "auto";
    this.platform = "desktop";
    this.displayName = "";
    this.recents = [];
    this.showCloudDialog = false;
    this.showDcDialog = false;
    this.diveListPrefs = {
      ...DEFAULT_DIVE_LIST_PREFS,
      colOrder: [...DEFAULT_DIVE_LIST_PREFS.colOrder],
      hiddenCols: [...DEFAULT_DIVE_LIST_PREFS.hiddenCols],
    };
  }
}

export const app = new AppStore();
