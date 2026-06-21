// AI-generated (Claude)
import { test, expect } from './fixtures.ts';
import type { Page } from '@playwright/test';

// Toolbar is desktop-only; MobileLayout is used on Android.
test.skip(({ platform }) => platform === 'android', 'desktop-only component');

async function setupToolbarPage(
  page: Page,
  opts: { theme: 'light' | 'dark'; isCloud?: boolean },
): Promise<void> {
  await page.addInitScript(([isCloud]) => {
    (window as any).__playwright_fixtures__ = { isCloud };
  }, [opts.isCloud ?? false] as [boolean]);
  await page.emulateMedia({ colorScheme: opts.theme });
  await page.goto('/toolbar-harness.html');
  await page.waitForLoadState('networkidle');
}

for (const theme of ['light', 'dark'] as const) {
  test(`toolbar default — ${theme}`, async ({ page }) => {
    await setupToolbarPage(page, { theme });
    await expect(page).toHaveScreenshot(`toolbar-default-${theme}.png`);
  });

  test(`toolbar cloud idle — ${theme}`, async ({ page }) => {
    await setupToolbarPage(page, { theme, isCloud: true });
    await expect(page).toHaveScreenshot(`toolbar-cloud-idle-${theme}.png`);
  });

  test(`toolbar cloud syncing — ${theme}`, async ({ page }) => {
    await setupToolbarPage(page, { theme, isCloud: true });
    await page.getByRole('button', { name: 'Sync' }).click();
    await page.waitForTimeout(50);
    await expect(page).toHaveScreenshot(`toolbar-cloud-syncing-${theme}.png`);
  });
}
