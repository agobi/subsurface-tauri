// AI-generated (Claude)
import { test, expect, setupPage } from './fixtures.ts';

for (const theme of ['light', 'dark'] as const) {
  test(`appearance tab — ${theme}`, async ({ page, platform }) => {
    await setupPage(page, { theme, path: '/prefs.html', platform });
    await expect(page).toHaveScreenshot(`prefs-${theme}.png`);
  });

  // The sidebar (and thus tab switching) is hidden below 600px, so this only
  // applies to the desktop project — see PrefsShell.svelte's media query.
  test(`logging tab — ${theme}`, async ({ page, platform }) => {
    test.skip(platform === 'android', 'sidebar hidden on narrow viewports');
    await setupPage(page, { theme, path: '/prefs.html', platform });
    await page.getByRole('button', { name: 'Logging' }).click();
    await expect(page).toHaveScreenshot(`prefs-logging-${theme}.png`);
  });

  test(`recents tab — ${theme}`, async ({ page, platform }) => {
    test.skip(platform === 'android', 'sidebar hidden on narrow viewports');
    await setupPage(page, { theme, path: '/prefs.html', platform });
    await page.getByRole('button', { name: 'Recents' }).click();
    await expect(page).toHaveScreenshot(`prefs-recents-${theme}.png`);
  });
}
