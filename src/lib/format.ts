// AI-generated (Claude)
// Shared display formatters.

// Format a duration/time in seconds as M:SS (e.g. 3310 -> "55:10").
export function fmtMinSec(sec: number): string {
  return `${Math.floor(sec / 60)}:${String(sec % 60).padStart(2, "0")}`;
}
