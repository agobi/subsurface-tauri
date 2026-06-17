// AI-generated (Claude)
import type { Trip } from "$lib/types.ts";
import { splitKeyword, unquote } from "./tokenize.ts";

export function parseTrip(text: string, diveNumbers: number[]): Trip {
  let label = "Trip";
  let notes: string | undefined;
  for (const line of text.split("\n")) {
    const { key, rest } = splitKeyword(line);
    if (key === "location") label = unquote(rest) || label;
    else if (key === "notes") notes = unquote(rest) || undefined;
  }
  return { label, notes, diveNumbers };
}
