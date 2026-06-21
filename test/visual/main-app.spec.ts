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
    await page.locator('[data-testid="dive-row"]').nth(1).click();
    await page.waitForTimeout(200); // let Svelte reactivity flush
    await expect(page).toHaveScreenshot(`sample-second-${theme}.png`);
  });

  test(`dive list scrolled to country column — ${theme}`, async ({ page, platform }) => {
    test.skip(platform === 'android', 'no quad-list panel on mobile');
    await setupPage(page, { logbook: sampleLogbook, theme, platform });
    await page.locator('[data-testid="quad-list"]').locator('.body').evaluate(
      (el: HTMLElement) => { el.scrollLeft = 300; },
    );
    await page.waitForTimeout(100);
    await expect(page.locator('[data-testid="quad-list"]')).toHaveScreenshot(`dive-list-scrolled-${theme}.png`);
  });
}
