// AI-generated (Claude)
import type { Cylinder } from "$lib/types.ts";
import { splitKeyword, unquote, parseAttrs, stripUnit } from "./tokenize.ts";

export interface DiveOverview {
  durationSec: number;
  rating?: number;
  tags: string[];
  siteId?: string;
  diveGuide?: string;
  buddy?: string;
  suit?: string;
  notes?: string;
  cylinders: Cylinder[];
}

function parseDuration(rest: string): number {
  const m = rest.match(/(\d+):(\d+)/);
  return m ? parseInt(m[1]) * 60 + parseInt(m[2]) : 0;
}

function parseCylinder(rest: string): Cylinder {
  const a = parseAttrs(rest);
  const cyl: Cylinder = { description: a.description ?? "" };
  if (a.vol != null) cyl.volumeL = stripUnit(a.vol);
  if (a.workpressure != null) cyl.workPressureBar = stripUnit(a.workpressure);
  if (a.o2 != null) cyl.o2Percent = stripUnit(a.o2);
  if (a.he != null) cyl.hePercent = stripUnit(a.he);
  if (a.start != null) cyl.startBar = stripUnit(a.start);
  if (a.end != null) cyl.endBar = stripUnit(a.end);
  if (a.use != null) cyl.use = a.use;
  if (a.depth != null) cyl.depthM = stripUnit(a.depth);
  return cyl;
}

export function parseDive(text: string): DiveOverview {
  const d: DiveOverview = { durationSec: 0, tags: [], cylinders: [] };
  for (const line of text.split("\n")) {
    const { key, rest } = splitKeyword(line);
    switch (key) {
      case "duration": d.durationSec = parseDuration(rest); break;
      case "rating": d.rating = parseInt(rest); break;
      case "tags": d.tags = rest.split(",").map((t) => unquote(t.trim())).filter(Boolean); break;
      case "divesiteid": d.siteId = rest.trim(); break;
      case "divemaster": d.diveGuide = unquote(rest); break;
      case "buddy": d.buddy = unquote(rest); break;
      case "suit": d.suit = unquote(rest); break;
      case "notes": d.notes = unquote(rest); break;
      case "cylinder": d.cylinders.push(parseCylinder(rest)); break;
      default: break;
    }
  }
  return d;
}
