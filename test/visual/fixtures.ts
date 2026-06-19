// AI-generated (Claude)
import type { Page } from '@playwright/test';
import type { Logbook } from '../../src/lib/types.ts';

export const sampleLogbook: Logbook = {
  units: 'METRIC',
  dives: [
    {
      number: 269,
      dateTime: '2024-03-15T12:28:43',
      durationSec: 3310,
      siteId: 'c171f112',
      tags: ['cave'],
      rating: 4,
      maxDepthM: 34.7,
      cylinders: [],
      samples: [
        { timeSec: 0, depthM: 0 },
        { timeSec: 300, depthM: 20.0 },
        { timeSec: 1200, depthM: 34.7 },
        { timeSec: 3000, depthM: 5.0 },
        { timeSec: 3310, depthM: 0 },
      ],
      events: [],
    },
    {
      number: 270,
      dateTime: '2024-04-01T10:00:00',
      durationSec: 2400,
      siteId: '04782ed8',
      tags: ['reef'],
      rating: 5,
      maxDepthM: 18.2,
      cylinders: [],
      samples: [],
      events: [],
    },
  ],
  trips: [{ label: 'March 2024', diveNumbers: [269] }],
  sites: [
    { id: 'c171f112', name: 'Molnar Janos' },
    { id: '04782ed8', name: 'Fenyes Forras' },
  ],
};

export async function setupPage(
  page: Page,
  opts: { logbook?: Logbook | null; theme: 'light' | 'dark'; path?: string },
): Promise<void> {
  await page.addInitScript((lb) => {
    (window as any).__playwright_fixtures__ = { logbook: lb };
  }, opts.logbook ?? null);
  await page.emulateMedia({ colorScheme: opts.theme });
  await page.goto(opts.path ?? '/');
  await page.waitForLoadState('networkidle');
}
