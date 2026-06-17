// AI-generated (Claude)
import type { Site } from "$lib/types.ts";
import { splitKeyword, unquote } from "./tokenize.ts";

export function parseSite(fileName: string, text: string): Site {
  const id = fileName.replace(/^Site-/, "");
  const site: Site = { id, name: "" };
  for (const line of text.split("\n")) {
    const { key, rest } = splitKeyword(line);
    if (key === "name") site.name = unquote(rest);
    else if (key === "description") site.description = unquote(rest);
    else if (key === "notes") site.notes = unquote(rest);
    else if (key === "gps") {
      const [lat, lon] = rest.trim().split(/\s+/).map(parseFloat);
      if (!Number.isNaN(lat) && !Number.isNaN(lon)) site.gps = { lat, lon };
    }
  }
  return site;
}
