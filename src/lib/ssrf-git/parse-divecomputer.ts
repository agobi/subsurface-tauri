// AI-generated (Claude)
import type { Sample, DiveEvent } from "$lib/types.ts";
import { splitKeyword, unquote, parseAttrs, stripUnit } from "./tokenize.ts";

export interface DivecomputerData {
  model?: string;
  maxDepthM?: number;
  meanDepthM?: number;
  waterTempC?: number;
  decoModel?: string;
  events: DiveEvent[];
  samples: Sample[];
}

function mmss(s: string): number {
  const m = s.match(/(\d+):(\d+)/);
  return m ? parseInt(m[1]) * 60 + parseInt(m[2]) : 0;
}

// A sample line begins with whitespace then "M:SS". Example:
//   "  0:15 2.5m 24.0°C ndl=99:00 cns=5%"
function parseSampleLine(line: string, carry: Partial<Sample>): Sample {
  const tokens = line.trim().split(/\s+/);
  const s: Sample = { timeSec: mmss(tokens[0]), depthM: 0 };
  for (let i = 1; i < tokens.length; i++) {
    const tok = tokens[i];
    if (tok.endsWith("m") && !tok.includes("=")) s.depthM = stripUnit(tok);
    else if (tok.endsWith("°C")) carry.tempC = stripUnit(tok);
    else if (tok.startsWith("ndl=")) carry.ndlSec = mmss(tok.slice(4));
    else if (tok.startsWith("tts=")) carry.ttsSec = mmss(tok.slice(4));
    else if (tok.startsWith("cns=")) carry.cns = stripUnit(tok.slice(4));
    else if (tok.endsWith("bar") && !tok.includes("=")) carry.pressureBar = stripUnit(tok.split(":")[0]);
  }
  if (carry.tempC != null) s.tempC = carry.tempC;
  if (carry.ndlSec != null) s.ndlSec = carry.ndlSec;
  if (carry.ttsSec != null) s.ttsSec = carry.ttsSec;
  if (carry.cns != null) s.cns = carry.cns;
  if (carry.pressureBar != null) s.pressureBar = carry.pressureBar;
  return s;
}

function parseEvent(rest: string): DiveEvent {
  const timeTok = rest.split(/\s+/)[0];
  const a = parseAttrs(rest);
  const ev: DiveEvent = { timeSec: mmss(timeTok), name: a.name ?? "" };
  if (a.type != null) ev.type = parseInt(a.type);
  if (a.flags != null) ev.flags = parseInt(a.flags);
  if (a.value != null) ev.value = parseInt(a.value);
  if (a.cylinder != null) ev.cylinder = parseInt(a.cylinder);
  if (a.o2 != null) ev.o2Percent = stripUnit(a.o2);
  return ev;
}

export function parseDivecomputer(text: string): DivecomputerData {
  const dc: DivecomputerData = { events: [], samples: [] };
  const carry: Partial<Sample> = {};
  for (const line of text.split("\n")) {
    if (/^\s+\d+:\d+/.test(line)) { dc.samples.push(parseSampleLine(line, carry)); continue; }
    const { key, rest } = splitKeyword(line);
    switch (key) {
      case "model": dc.model = unquote(rest); break;
      case "maxdepth": dc.maxDepthM = stripUnit(rest); break;
      case "meandepth": dc.meanDepthM = stripUnit(rest); break;
      case "watertemp": dc.waterTempC = stripUnit(rest); break;
      case "event": dc.events.push(parseEvent(rest)); break;
      case "keyvalue": {
        const m = rest.match(/^"([^"]*)"\s+"([^"]*)"/);
        if (m && m[1] === "Deco model") dc.decoModel = m[2];
        break;
      }
      default: break;
    }
  }
  return dc;
}
