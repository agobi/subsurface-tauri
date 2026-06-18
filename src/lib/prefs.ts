// AI-generated (Claude)
import { load } from "@tauri-apps/plugin-store";
import { emit } from "@tauri-apps/api/event";
import type { Theme } from "$lib/stores/app.svelte.ts";

export interface AppearancePrefs {
  theme: Theme;
}

export function resolveTheme(theme: Theme): "dark" | "light" {
  if (theme !== "auto") return theme;
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

export function applyTheme(theme: Theme): void {
  document.documentElement.dataset.theme = resolveTheme(theme);
}

export async function loadAppearancePrefs(): Promise<AppearancePrefs> {
  const store = await load("settings.json");
  const saved = await store.get<AppearancePrefs>("appearance");
  return saved ?? { theme: "dark" };
}

export async function saveAndEmitAppearance(prefs: AppearancePrefs): Promise<void> {
  const store = await load("settings.json");
  await store.set("appearance", prefs);
  await store.save();
  await emit("prefs:appearance-changed", prefs);
}
