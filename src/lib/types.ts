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

export interface Dive {
  number: number;        // from Dive-NNN filename
  dateTime: string;      // ISO 8601, from the dive directory name
  durationSec: number;
  siteId?: string;       // divesiteid (hex, no 0x)
  tags: string[];
  rating?: number;       // 0..5
  diveGuide?: string;    // "divemaster" line
  buddy?: string;
  suit?: string;
  notes?: string;
  cylinders: Cylinder[];
  dcModel?: string;
  maxDepthM?: number;
  meanDepthM?: number;
  waterTempC?: number;
  decoModel?: string;    // keyvalue "Deco model", e.g. "GF 55/85"
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
}

export interface Logbook {
  dives: Dive[];
  trips: Trip[];
  sites: Site[];
  units: Units;
}
