// AI-generated (Claude)
// Helpers for the Subsurface git-tree line format. Format reference: core/save-git.cpp.

export function splitKeyword(line: string): { key: string; rest: string } {
  const trimmed = line.replace(/^\s+/, "");
  const sp = trimmed.indexOf(" ");
  if (sp === -1) return { key: trimmed, rest: "" };
  return { key: trimmed.slice(0, sp), rest: trimmed.slice(sp + 1) };
}

export function unquote(s: string): string {
  const t = s.trim();
  if (t.startsWith('"') && t.endsWith('"') && t.length >= 2) {
    return t.slice(1, -1).replace(/\\(["\\])/g, "$1");
  }
  return t;
}

// "34.7m" -> 34.7, "232.0bar" -> 232, "32.0%" -> 32. Returns NaN if no number.
export function stripUnit(s: string): number {
  const m = s.match(/^-?[0-9]+(\.[0-9]+)?/);
  return m ? parseFloat(m[0]) : NaN;
}

// Parse 'a=1 b="x y" c=2.0unit' into { a:"1", b:"x y", c:"2.0unit" }.
// Splits on spaces that are not inside double quotes.
export function parseAttrs(rest: string): Record<string, string> {
  const out: Record<string, string> = {};
  const re = /(\w+)=("(?:[^"\\]|\\.)*"|\S+)/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(rest)) !== null) {
    out[m[1]] = unquote(m[2]);
  }
  return out;
}
