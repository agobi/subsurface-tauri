// AI-generated (Claude)
import { test, expect } from './fixtures.ts';
import type { Page } from '@playwright/test';

async function setupDcDownloadPage(page: Page, opts: { theme: 'light' | 'dark' }): Promise<void> {
  await page.emulateMedia({ colorScheme: opts.theme });
  await page.goto('/dc-download-harness.html');
  await page.waitForLoadState('networkidle');
}

async function driveToProgressStep(page: Page): Promise<void> {
  await expect(page.getByText('Add Device')).toBeVisible();
  await page.getByLabel('Vendor').selectOption('TestVendor');
  // "Transport"'s accessible name ends up including its selected option text
  // ("Transport Serial"), which contains "Port" as a substring — anchor to
  // the start so getByLabel('Port') doesn't match it too.
  await page.getByLabel(/^Port\b/).selectOption('/dev/ttyUSB0');
  await page.getByRole('button', { name: 'Download' }).click();
  await expect(page.getByText('Downloading…')).toBeVisible();
}

for (const theme of ['light', 'dark'] as const) {
  test(`DC download progress bar — ${theme}`, async ({ page }) => {
    await setupDcDownloadPage(page, { theme });
    await driveToProgressStep(page);
    await expect(page).toHaveScreenshot(`dc-download-progress-${theme}.png`);
  });
}
