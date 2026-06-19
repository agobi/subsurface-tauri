// AI-generated (Claude)
import { test, expect } from '@playwright/test';
import { setupPage } from './fixtures.ts';

for (const theme of ['light', 'dark'] as const) {
  test(`appearance tab — ${theme}`, async ({ page }) => {
    await setupPage(page, { theme, path: '/prefs.html' });
    await expect(page).toHaveScreenshot(`prefs-${theme}.png`);
  });
}
