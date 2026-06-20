// AI-generated (Claude)
import { test, expect, setupPage, sampleLogbook } from './fixtures.ts';

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
    await page.getByText('Fenyes Forras').click();
    await page.waitForTimeout(200); // let Svelte reactivity flush
    await expect(page).toHaveScreenshot(`sample-second-${theme}.png`);
  });
}
