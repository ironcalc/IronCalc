import { expect, test } from "@playwright/test";
import { waitForCanvas } from "./helpers";

// Visual regression tests — compare pixel-by-pixel against stored baselines.
//
// To generate or update baselines run:
//   npx playwright test --update-snapshots
// Commit the resulting files in e2e/screenshot.spec.ts-snapshots/.

test("blank workbook matches snapshot", async ({ page }) => {
  await page.goto("/");
  await waitForCanvas(page);
  // Allow fonts and styles to settle before capturing.
  await page.waitForTimeout(500);
  await expect(page).toHaveScreenshot("blank-workbook.png", {
    fullPage: true,
    // Tolerate minor anti-aliasing differences across platforms.
    maxDiffPixelRatio: 0.01,
  });
});
