// AI-generated (Claude)
// Vite alias target for all @tauri-apps/* modules when VITE_VISUAL_TEST=1.
// Playwright seeds window.__playwright_fixtures__ via page.addInitScript() before page load.
import type { Logbook, OpenResult } from '../../src/lib/types.ts';

const EMPTY: Logbook = { dives: [], trips: [], sites: [], units: 'METRIC' };

function fixtureLogbook(): Logbook {
  return (window as any).__playwright_fixtures__?.logbook ?? EMPTY;
}

function fixtureResult(): OpenResult {
  return { logbook: fixtureLogbook(), displayName: 'Test Logbook', recents: [] };
}

// @tauri-apps/api/core
export async function invoke<T>(cmd: string, _args?: unknown): Promise<T> {
  switch (cmd) {
    case 'startup_logbook':
    case 'open_logbook':
    case 'open_recent_cloud_logbook':
    case 'open_cloud_logbook':
    case 'sync_cloud_logbook':
      return fixtureResult() as T;
    case 'new_logbook':
      return { logbook: EMPTY, displayName: '', recents: [] } as T;
    default:
      return null as T;
  }
}

// @tauri-apps/api/event
export async function listen(_event: string, _handler: unknown): Promise<() => void> {
  return () => {};
}
export async function emit(_event: string, _payload?: unknown): Promise<void> {}

// @tauri-apps/api/window
export function getCurrentWindow() {
  return { setTitle: async (_title: string): Promise<void> => {} };
}

// @tauri-apps/plugin-dialog
export async function open(_opts?: unknown): Promise<null> {
  return null;
}

// @tauri-apps/plugin-os
export async function platform(): Promise<string> {
  return (window as any).__playwright_fixtures__?.platform ?? 'macos';
}

// @tauri-apps/plugin-store
export async function load(_path: string) {
  return {
    get: async (_key: string) => null,
    set: async (_key: string, _value: unknown): Promise<void> => {},
    save: async (): Promise<void> => {},
  };
}
