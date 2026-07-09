// AI-generated (Claude)
// Vite alias target for all @tauri-apps/* modules when VITE_VISUAL_TEST=1.
// Playwright seeds window.__playwright_fixtures__ via page.addInitScript() before page load.
import type { Logbook, OpenResult } from '../../src/lib/types.ts';

const EMPTY: Logbook = { dives: [], trips: [], sites: [], units: 'METRIC' };

function fixtureLogbook(): Logbook {
  return (window as any).__playwright_fixtures__?.logbook ?? EMPTY;
}

function fixtureResult(): OpenResult {
  return { logbook: fixtureLogbook(), displayName: 'Test Logbook', recents: [], warnings: [] };
}

type DcFixtures = {
  vendors?: string[];
  models?: { product: string; transports: string[] }[];
  serialPorts?: string[];
  progress?: { current: number; maximum: number };
  devinfo?: { model: number; firmware: number; serial: number };
};

function fixtureDc(): DcFixtures {
  return (window as any).__playwright_fixtures__?.dc ?? {};
}

// Populated by the `listen` mock below; `start_dc_download` replays fixture
// events through these so a harness page can drive DcDownloadDialog past its
// "setup" step without a real backend.
const eventListeners = new Map<string, ((e: { payload: unknown }) => void)[]>();

function emitToListeners(event: string, payload: unknown): void {
  for (const handler of eventListeners.get(event) ?? []) handler({ payload });
}

// @tauri-apps/api/core
export async function invoke<T>(cmd: string, args?: unknown): Promise<T> {
  switch (cmd) {
    case 'startup_logbook':
    case 'open_logbook':
    case 'open_recent_cloud_logbook':
    case 'open_cloud_logbook':
    case 'sync_cloud_logbook':
      return fixtureResult() as T;
    case 'new_logbook':
      return { logbook: EMPTY, displayName: '', recents: [], warnings: [] } as T;
    case 'get_dive': {
      const a = args as { number: number } | undefined;
      const dive = fixtureLogbook().dives.find(d => d.number === a?.number);
      return (dive ?? null) as T;
    }
    case 'list_known_devices':
      return [] as T;
    case 'list_dc_vendors':
      return (fixtureDc().vendors ?? ['TestVendor']) as T;
    case 'list_dc_models':
      return (fixtureDc().models ?? [{ product: 'TestModel', transports: ['Serial'] }]) as T;
    case 'list_serial_ports':
      return (fixtureDc().serialPorts ?? ['/dev/ttyUSB0']) as T;
    case 'get_log_level':
      return 'INFO' as T;
    case 'start_dc_download': {
      const dc = fixtureDc();
      if (dc.devinfo) emitToListeners('dc-devinfo', dc.devinfo);
      emitToListeners('dc-progress', dc.progress ?? { current: 524288, maximum: 1048576 });
      // Never resolves — parks the dialog on the "progress" step so
      // Playwright can screenshot it mid-download, same as the Toolbar
      // sync-in-progress harness.
      return new Promise<T>(() => {});
    }
    default:
      return null as T;
  }
}

// @tauri-apps/api/event
export async function listen(event: string, handler: (e: { payload: unknown }) => void): Promise<() => void> {
  const handlers = eventListeners.get(event) ?? [];
  handlers.push(handler);
  eventListeners.set(event, handlers);
  return () => {
    eventListeners.set(event, (eventListeners.get(event) ?? []).filter(h => h !== handler));
  };
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
export async function message(_msg: string, _opts?: unknown): Promise<void> {}

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
