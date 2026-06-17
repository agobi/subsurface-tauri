// AI-generated (Claude)
import fs from "node:fs";
import path from "node:path";
import type { Logbook, Dive, Site, Trip } from "$lib/types.ts";
import { parseHeader } from "./parse-header.ts";
import { parseSite } from "./parse-site.ts";
import { parseDive } from "./parse-dive.ts";
import { parseDivecomputer, type DivecomputerData } from "./parse-divecomputer.ts";

const DIVE_DIR_RE = /^(\d{2})-\w{3}-(\d{2})=(\d{2})=(\d{2})$/; // DD-Day-HH=MM=SS
const YEAR_RE = /^\d{4}$/;
const MONTH_RE = /^\d{2}$/;

function read(p: string): string { return fs.readFileSync(p, "utf8"); }
function exists(p: string): boolean { return fs.existsSync(p); }

function parseSites(root: string): Site[] {
  const dir = path.join(root, "01-Divesites");
  if (!exists(dir)) return [];
  return fs.readdirSync(dir)
    .filter((f) => f.startsWith("Site-"))
    .map((f) => parseSite(f, read(path.join(dir, f))));
}

function parseDiveDir(dirPath: string, year: string, month: string, dirName: string): Dive | null {
  const m = dirName.match(DIVE_DIR_RE);
  if (!m) return null;
  const [, day, hh, mm, ss] = m;
  const diveFile = fs.readdirSync(dirPath).find((f) => /^Dive-\d+/.test(f));
  if (!diveFile) return null;
  const number = parseInt(diveFile.replace(/^Dive-/, ""));
  const overview = parseDive(read(path.join(dirPath, diveFile)));
  const dcPath = path.join(dirPath, "Divecomputer");
  const emptyDc: DivecomputerData = { events: [], samples: [] };
  const dc = exists(dcPath) ? parseDivecomputer(read(dcPath)) : emptyDc;

  return {
    number,
    dateTime: `${year}-${month}-${day}T${hh}:${mm}:${ss}`,
    durationSec: overview.durationSec,
    siteId: overview.siteId,
    tags: overview.tags,
    rating: overview.rating,
    diveGuide: overview.diveGuide,
    buddy: overview.buddy,
    suit: overview.suit,
    notes: overview.notes,
    cylinders: overview.cylinders,
    dcModel: dc.model,
    maxDepthM: dc.maxDepthM,
    meanDepthM: dc.meanDepthM,
    waterTempC: dc.waterTempC,
    decoModel: dc.decoModel,
    samples: dc.samples,
    events: dc.events,
  };
}

export function parseLogbook(root: string): Logbook {
  const headerPath = path.join(root, "00-Subsurface");
  const units = exists(headerPath) ? parseHeader(read(headerPath)).units : "METRIC";
  const sites = parseSites(root);
  const dives: Dive[] = [];
  const trips: Trip[] = [];

  for (const year of fs.readdirSync(root).filter((d) => YEAR_RE.test(d))) {
    const yearDir = path.join(root, year);
    for (const month of fs.readdirSync(yearDir).filter((d) => MONTH_RE.test(d))) {
      const monthDir = path.join(yearDir, month);
      for (const entry of fs.readdirSync(monthDir)) {
        const full = path.join(monthDir, entry);
        if (!fs.statSync(full).isDirectory()) continue;
        if (entry.endsWith("trip") || entry.startsWith("-")) continue; // trip-grouping dirs: best-effort, skipped in prototype
        const dive = parseDiveDir(full, year, month, entry);
        if (dive) dives.push(dive);
      }
    }
  }

  dives.sort((a, b) => a.dateTime.localeCompare(b.dateTime));
  return { dives, trips, sites, units };
}
