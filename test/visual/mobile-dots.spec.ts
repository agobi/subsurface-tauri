// AI-generated (Claude)
import { test, expect, setupPage, sampleLogbook } from './fixtures.ts';

// Android-only: the swipeable top-row/dot-indicator layout (issue #87) only exists
// on the mobile two-row layout.
test.beforeEach(({ platform }) => {
  test.skip(platform !== 'android', 'android-only layout');
});

test.describe('mobile panel-selector dots', () => {
  test('dot indicator does not shift position when the active panel changes', async ({ page, platform }) => {
    await setupPage(page, { logbook: sampleLogbook, theme: 'light', platform });

    const dots = page.locator('.dots-row .dots');
    const label = page.getByTestId('mobile-active-panel-label');

    // Panel labels are different lengths ("Info" / "Profile" / "Map"); the dots
    // must stay put regardless of which label is currently showing next to them.
    await page.getByTestId('mobile-dot-info').click();
    await expect(label).toHaveText('Info');
    const infoBox = await dots.boundingBox();

    await page.getByTestId('mobile-dot-profile').click();
    await expect(label).toHaveText('Profile');
    const profileBox = await dots.boundingBox();

    await page.getByTestId('mobile-dot-map').click();
    await expect(label).toHaveText('Map');
    const mapBox = await dots.boundingBox();

    expect(infoBox).not.toBeNull();
    expect(profileBox).not.toBeNull();
    expect(mapBox).not.toBeNull();

    // Sub-pixel layout jitter is fine; the ~10px shift from the old centered-row
    // layout (dots + label centered together) is the regression this guards against.
    expect(profileBox!.x).toBeCloseTo(infoBox!.x, 0);
    expect(mapBox!.x).toBeCloseTo(infoBox!.x, 0);
    expect(profileBox!.y).toBeCloseTo(infoBox!.y, 0);
    expect(mapBox!.y).toBeCloseTo(infoBox!.y, 0);
  });

  test('dot indicator does not shift position when swiping the panel via scroll', async ({ page, platform }) => {
    await setupPage(page, { logbook: sampleLogbook, theme: 'light', platform });

    const dots = page.locator('.dots-row .dots');
    const swipe = page.getByTestId('mobile-swipe');
    const label = page.getByTestId('mobile-active-panel-label');

    await expect(label).toHaveText('Profile'); // default
    const profileBox = await dots.boundingBox();

    await swipe.evaluate((el: HTMLElement) => {
      el.scrollTo({ left: 0 }); // scroll to Info (index 0)
    });
    await expect(label).toHaveText('Info');
    const infoBox = await dots.boundingBox();

    expect(infoBox).not.toBeNull();
    expect(profileBox).not.toBeNull();
    expect(infoBox!.x).toBeCloseTo(profileBox!.x, 0);
    expect(infoBox!.y).toBeCloseTo(profileBox!.y, 0);
  });

  for (const theme of ['light', 'dark'] as const) {
    for (const panel of ['info', 'profile', 'map'] as const) {
      test(`dots row appearance — ${panel} active — ${theme}`, async ({ page, platform }) => {
        await setupPage(page, { logbook: sampleLogbook, theme, platform });
        await page.getByTestId(`mobile-dot-${panel}`).click();
        await page.waitForTimeout(100);
        await expect(page.locator('.dots-row')).toHaveScreenshot(`mobile-dots-row-${panel}-${theme}.png`);
      });
    }
  }
});
