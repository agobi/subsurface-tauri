// AI-generated (Claude)
import { test, expect, setupPage } from './fixtures.ts';

for (const theme of ['light', 'dark'] as const) {
  test(`appearance tab — ${theme}`, async ({ page, platform }) => {
    await setupPage(page, { theme, path: '/prefs.html', platform });
    await expect(page).toHaveScreenshot(`prefs-${theme}.png`);
  });
}
