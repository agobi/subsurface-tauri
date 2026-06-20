// AI-generated (Claude)
import type { Dive, Site, Cylinder } from "$lib/types.ts";
import { fmtMinSec } from "$lib/format.ts";

export type ColId =
  | "nr" | "date" | "rating" | "depth" | "duration"
  | "buddy" | "guide" | "country" | "location"
  | "temp" | "suit" | "cylinder" | "sac" | "tags"
  | "notes" | "divemode" | "weight";

export interface RenderCtx {
  sites: Site[];
}

export interface ColDef {
  id: ColId;
  label: string;
  width: string;
  defaultVisible: boolean;
  render: (d: Dive, ctx: RenderCtx) => string;
  // missing values sort last in both asc and desc — enforced by AppStore.sortedDives
  compare: (a: Dive, b: Dive, ctx: RenderCtx) => number;
}

export const DEFAULT_COL_ORDER: ColId[] = [
  "nr", "date", "rating", "depth", "duration",
  "buddy", "guide", "country", "location",
];

function gasMix(c?: Cylinder): string {
  if (!c) return "—";
  const o2 = c.o2Percent;
  const he = c.hePercent;
  if (he != null && he > 0) return `${Math.round(o2 ?? 21)}/${Math.round(he)}`;
  if (o2 == null || Math.abs(o2 - 21) < 0.5) return "Air";
  return `EAN${Math.round(o2)}`;
}

function sacRaw(d: Dive): number | null {
  const cyl = d.cylinders[0];
  if (!cyl || cyl.startBar == null || cyl.endBar == null || cyl.volumeL == null || d.meanDepthM == null) return null;
  const durationMin = d.durationSec / 60;
  if (durationMin <= 0) return null;
  return (cyl.startBar - cyl.endBar) * cyl.volumeL / durationMin / (d.meanDepthM / 10 + 1);
}

function lookupSiteName(d: Dive, sites: Site[]): string {
  return sites.find(s => s.id === d.siteId)?.name ?? "—";
}

function lookupCountry(d: Dive, sites: Site[]): string {
  return sites.find(s => s.id === d.siteId)?.country ?? "—";
}

export const ALL_COLS: ColDef[] = [
  {
    id: "nr", label: "#", width: "40px", defaultVisible: true,
    render: (d) => String(d.number),
    compare: (a, b) => a.number - b.number,
  },
  {
    id: "date", label: "Date", width: "90px", defaultVisible: true,
    render: (d) => d.dateTime.slice(0, 10),
    compare: (a, b) => a.dateTime.localeCompare(b.dateTime),
  },
  {
    id: "rating", label: "Rating", width: "72px", defaultVisible: true,
    render: (d) => d.rating != null ? "★".repeat(d.rating) + "☆".repeat(5 - d.rating) : "—",
    compare: (a, b) => (a.rating ?? 0) - (b.rating ?? 0),
  },
  {
    id: "depth", label: "Depth", width: "56px", defaultVisible: true,
    render: (d) => d.maxDepthM != null ? d.maxDepthM.toFixed(1) : "—",
    compare: (a, b) => (a.maxDepthM ?? 0) - (b.maxDepthM ?? 0),
  },
  {
    id: "duration", label: "Duration", width: "64px", defaultVisible: true,
    render: (d) => fmtMinSec(d.durationSec),
    compare: (a, b) => a.durationSec - b.durationSec,
  },
  {
    id: "buddy", label: "Buddy", width: "1fr", defaultVisible: true,
    render: (d) => d.buddy ?? "—",
    compare: (a, b) => (a.buddy ?? "").localeCompare(b.buddy ?? ""),
  },
  {
    id: "guide", label: "Dive guide", width: "1fr", defaultVisible: true,
    render: (d) => d.diveGuide ?? "—",
    compare: (a, b) => (a.diveGuide ?? "").localeCompare(b.diveGuide ?? ""),
  },
  {
    id: "country", label: "Country", width: "90px", defaultVisible: true,
    render: (d, ctx) => lookupCountry(d, ctx.sites),
    compare: (a, b, ctx) => lookupCountry(a, ctx.sites).localeCompare(lookupCountry(b, ctx.sites)),
  },
  {
    id: "location", label: "Location", width: "1fr", defaultVisible: true,
    render: (d, ctx) => lookupSiteName(d, ctx.sites),
    compare: (a, b, ctx) => lookupSiteName(a, ctx.sites).localeCompare(lookupSiteName(b, ctx.sites)),
  },
  {
    id: "temp", label: "Temp.", width: "56px", defaultVisible: false,
    render: (d) => d.waterTempC != null ? d.waterTempC.toFixed(1) : "—",
    compare: (a, b) => (a.waterTempC ?? 0) - (b.waterTempC ?? 0),
  },
  {
    id: "suit", label: "Suit", width: "88px", defaultVisible: false,
    render: (d) => d.suit ?? "—",
    compare: (a, b) => (a.suit ?? "").localeCompare(b.suit ?? ""),
  },
  {
    id: "cylinder", label: "Cylinder", width: "72px", defaultVisible: false,
    render: (d) => gasMix(d.cylinders[0]),
    compare: (a, b) => gasMix(a.cylinders[0]).localeCompare(gasMix(b.cylinders[0])),
  },
  {
    id: "sac", label: "SAC", width: "64px", defaultVisible: false,
    render: (d) => { const v = sacRaw(d); return v != null ? v.toFixed(1) : "—"; },
    compare: (a, b) => (sacRaw(a) ?? 0) - (sacRaw(b) ?? 0),
  },
  {
    id: "tags", label: "Tags", width: "1fr", defaultVisible: false,
    render: (d) => d.tags.length ? d.tags.join(", ") : "—",
    compare: (a, b) => a.tags.join(", ").localeCompare(b.tags.join(", ")),
  },
  {
    id: "notes", label: "Notes", width: "2fr", defaultVisible: false,
    render: (d) => {
      if (!d.notes) return "—";
      return d.notes.length > 60 ? d.notes.slice(0, 60) + "…" : d.notes;
    },
    compare: (a, b) => (a.notes ?? "").localeCompare(b.notes ?? ""),
  },
  {
    id: "divemode", label: "Divemode", width: "72px", defaultVisible: false,
    render: (d) => d.divemode ?? "—",
    compare: (a, b) => (a.divemode ?? "").localeCompare(b.divemode ?? ""),
  },
  {
    id: "weight", label: "Weight", width: "56px", defaultVisible: false,
    render: (d) => d.totalWeightKg != null ? d.totalWeightKg.toFixed(2) : "—",
    compare: (a, b) => (a.totalWeightKg ?? 0) - (b.totalWeightKg ?? 0),
  },
];
