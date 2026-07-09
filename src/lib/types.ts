// AI-generated (Claude)
export type Units = "METRIC" | "IMPERIAL";

export interface Sample {
  timeSec: number;       // seconds from dive start
  depthM: number;        // metres (0 = surface)
  tempC?: number;        // carried forward from last reported value
  ndlSec?: number;       // no-deco limit, carried forward
  ttsSec?: number;       // time-to-surface, carried forward
  cns?: number;          // percent, carried forward
  pressureBar?: number;  // first tank sensor, carried forward
}

export interface Cylinder {
  description: string;
  volumeL?: number;
  workPressureBar?: number;
  o2Percent?: number;
  hePercent?: number;
  startBar?: number;
  endBar?: number;
  use?: string;
  depthM?: number;
}

export interface DiveEvent {
  timeSec: number;
  name: string;          // e.g. "gaschange"
  type?: number;
  flags?: number;
  value?: number;
  cylinder?: number;
  o2Percent?: number;
}

export interface DiveSummary {
  number: number;
  dateTime: string;
  durationSec: number;
  siteId?: string;
  tags: string[];
  rating?: number;
  diveGuide?: string;
  buddy?: string;
  suit?: string;
  notes?: string;
  cylinders: Cylinder[];
  dcModel?: string;
  maxDepthM?: number;
  meanDepthM?: number;
  waterTempC?: number;
  decoModel?: string;
  divemode?: string;
  totalWeightKg?: number;
  mediaCount: number;
}

export interface Dive extends DiveSummary {
  samples: Sample[];
  events: DiveEvent[];
}

export interface Trip {
  label: string;         // trip "location" line, or a derived label
  area?: string;
  notes?: string;
  diveNumbers: number[];
}

export interface Site {
  id: string;            // hex, from filename Site-XXXXXXXX
  name: string;
  description?: string;
  notes?: string;
  gps?: { lat: number; lon: number };
  country?: string;
}

export interface Logbook {
  dives: DiveSummary[];
  trips: Trip[];
  sites: Site[];
  units: Units;
}

export type RecentEntry =
  | { kind: "Local"; path: string }
  | { kind: "Cloud"; email: string; url: string };

export interface OpenResult {
  logbook: Logbook;
  displayName: string;
  recents: RecentEntry[];
}
