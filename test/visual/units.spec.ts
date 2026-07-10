// AI-generated (Claude)
import { test, expect, setupPage, sampleLogbook } from './fixtures.ts';
import type { Logbook } from '../../src/lib/types.ts';

// Extend the first sample dive with the fields the units preference affects
// (water temp, total weight, cylinder volume/pressures) so metric-vs-imperial
// conversions actually render non-trivial numbers in these screenshots.
const sampleLogbookWithUnits: Logbook = {
  ...sampleLogbook,
  dives: [
    {
      ...sampleLogbook.dives[0],
      meanDepthM: 14.8,
      waterTempC: 19,
      totalWeightKg: 3.5,
      cylinders: [
        { description: 'D12 232 bar', volumeL: 12, workPressureBar: 232, startBar: 220, endBar: 60, o2Percent: 32 },
      ],
    },
    ...sampleLogbook.dives.slice(1),
  ],
};

for (const units of ['METRIC', 'IMPERIAL'] as const) {
  const suffix = units === 'METRIC' ? 'metric' : 'imperial';

  test(`dive list depth column — ${suffix}`, async ({ page, platform }) => {
    test.skip(platform === 'android', 'desktop-only per scoped coverage');
    await setupPage(page, { logbook: sampleLogbookWithUnits, theme: 'dark', platform });
    await page.evaluate(async (u) => {
      const { app } = await import('/src/lib/stores/app.svelte.ts');
      app.setUnitsPref(u);
    }, units);
    await page.waitForTimeout(100);
    await expect(page.locator('[data-testid="quad-list"]')).toHaveScreenshot(`dive-list-${suffix}-dark.png`);
  });

  test(`information tab depth/temp — ${suffix}`, async ({ page, platform }) => {
    test.skip(platform === 'android', 'desktop-only per scoped coverage');
    await setupPage(page, { logbook: sampleLogbookWithUnits, theme: 'dark', platform });
    await page.evaluate(async (u) => {
      const { app } = await import('/src/lib/stores/app.svelte.ts');
      app.setUnitsPref(u);
    }, units);
    await page.getByRole('tab', { name: 'Information' }).click();
    await page.waitForTimeout(100);
    await expect(page.locator('[data-testid="quad-info"]')).toHaveScreenshot(`info-tab-${suffix}-dark.png`);
  });

  test(`equipment tab headers/values — ${suffix}`, async ({ page, platform }) => {
    test.skip(platform === 'android', 'desktop-only per scoped coverage');
    await setupPage(page, { logbook: sampleLogbookWithUnits, theme: 'dark', platform });
    await page.evaluate(async (u) => {
      const { app } = await import('/src/lib/stores/app.svelte.ts');
      app.setUnitsPref(u);
    }, units);
    await page.getByRole('tab', { name: 'Equipment' }).click();
    await page.waitForTimeout(100);
    await expect(page.locator('[data-testid="quad-info"]')).toHaveScreenshot(`equipment-tab-${suffix}-dark.png`);
  });
}
