// AI-generated (Claude)
// Shared display formatters.
import type { RecentEntry } from "$lib/types.ts";

// Format a duration/time in seconds as M:SS (e.g. 3310 -> "55:10").
export function fmtMinSec(sec: number): string {
  return `${Math.floor(sec / 60)}:${String(sec % 60).padStart(2, "0")}`;
}

// Label shown for a recents-list entry: basename for local paths, "email@host" for cloud.
export function fmtRecentLabel(entry: RecentEntry): string {
  if (entry.kind === "Local") {
    return entry.path.split(/[\\/]/).pop() || entry.path;
  }
  const host = entry.url.replace(/^https?:\/\//, "");
  return `${entry.email}@${host}`;
}
