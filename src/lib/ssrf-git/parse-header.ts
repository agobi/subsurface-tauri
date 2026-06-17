// AI-generated (Claude)
import type { Units } from "$lib/types.ts";
import { splitKeyword } from "./tokenize.ts";

export function parseHeader(text: string): { units: Units } {
  let units: Units = "METRIC";
  for (const line of text.split("\n")) {
    const { key, rest } = splitKeyword(line);
    if (key === "units") units = rest.trim().startsWith("IMPERIAL") ? "IMPERIAL" : "METRIC";
  }
  return { units };
}
