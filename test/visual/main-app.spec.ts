// AI-generated (Claude)
import { test, expect, setupPage, sampleLogbook } from './fixtures.ts';
import type { Logbook } from '../../src/lib/types.ts';

// Extend the sample with GPS so the Leaflet map renders for the first selected dive.
const sampleLogbookWithGps: Logbook = {
  ...sampleLogbook,
  sites: [
    { id: 'c171f112', name: 'Molnar Janos', country: 'Hungary', gps: { lat: 47.5, lon: 19.0 } },
    ...sampleLogbook.sites.slice(1),
  ],
};

for (const theme of ['light', 'dark'] as const) {
  test(`empty logbook — ${theme}`, async ({ page, platform }) => {
    await setupPage(page, { logbook: null, theme, platform });
    await expect(page).toHaveScreenshot(`empty-${theme}.png`);
  });

  test(`sample data first dive selected — ${theme}`, async ({ page, platform }) => {
    await setupPage(page, { logbook: sampleLogbook, theme, platform });
    await expect(page).toHaveScreenshot(`sample-selected-${theme}.png`);
  });

  test(`sample data second dive clicked — ${theme}`, async ({ page, platform }) => {
    await setupPage(page, { logbook: sampleLogbook, theme, platform });
    await page.locator('[data-testid="dive-row"]').nth(1).click();
    await page.waitForTimeout(200); // let Svelte reactivity flush
    await expect(page).toHaveScreenshot(`sample-second-${theme}.png`);
  });

  // Regression: cloud dialog must sit above Leaflet's zoom controls (z-index 1000).
  test(`cloud dialog above map — ${theme}`, async ({ page, platform }) => {
    test.skip(platform === 'android', 'desktop-only');
    await setupPage(page, { logbook: sampleLogbookWithGps, theme, platform });
    await page.evaluate(async () => {
      const { app } = await import('/src/lib/stores/app.svelte.ts');
      app.showCloudDialog = { email: '' };
    });
    await page.waitForTimeout(100);
    await expect(page).toHaveScreenshot(`cloud-dialog-${theme}.png`);
  });

  // Regression: DC download dialog must sit above Leaflet's zoom controls (z-index 1000).
  test(`dc download dialog above map — ${theme}`, async ({ page, platform }) => {
    test.skip(platform === 'android', 'desktop-only');
    await setupPage(page, { logbook: sampleLogbookWithGps, theme, platform });
    await page.evaluate(async () => {
      const { app } = await import('/src/lib/stores/app.svelte.ts');
      app.showDcDialog = true;
    });
    await page.waitForTimeout(100);
    await expect(page).toHaveScreenshot(`dc-dialog-${theme}.png`);
  });

  // Android has no menu bar, so Preferences (prefs.html) is unreachable there —
  // MobileSettingsScreen is the actual settings UI on Android.
  test(`mobile settings screen — ${theme}`, async ({ page, platform }) => {
    test.skip(platform !== 'android', 'android-only screen');
    await setupPage(page, { logbook: sampleLogbook, theme, platform });
    await page.getByLabel('Settings').click();
    await page.waitForTimeout(100);
    await expect(page).toHaveScreenshot(`mobile-settings-${theme}.png`);
  });

  test(`dive list scrolled to country column — ${theme}`, async ({ page, platform }) => {
    await setupPage(page, { logbook: sampleLogbook, theme, platform });
    const isAndroid = platform === 'android';
    const scrollContainer = isAndroid
      ? page.locator('[data-testid="mobile-panel-dives"]')
      : page.locator('[data-testid="quad-list"]').locator('.body');
    const screenshotTarget = isAndroid
      ? page.locator('[data-testid="mobile-panel-dives"]')
      : page.locator('[data-testid="quad-list"]');
    await scrollContainer.evaluate((el: HTMLElement) => { el.scrollLeft = 300; });
    await page.waitForTimeout(100);
    await expect(screenshotTarget).toHaveScreenshot(`dive-list-scrolled-${theme}.png`);
  });
}
