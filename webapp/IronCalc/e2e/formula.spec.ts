import { expect, test } from "@playwright/test";
import {
  clickCell,
  getFormulaBarValue,
  waitForCanvas,
} from "./helpers";

test("entering =1+1 in A1 shows result 2", async ({ page }) => {
  await page.goto("/");
  await waitForCanvas(page);

  // Select A1 and type a formula.
  await clickCell(page, 1, 1);
  await page.keyboard.type("=1+1");
  await page.keyboard.press("Enter");

  // Re-select A1.
  await clickCell(page, 1, 1);

  // Formula bar shows the formula (standard spreadsheet behaviour).
  expect(await getFormulaBarValue(page)).toBe("=1+1");

  // The canvas renders the computed value: verify visually.
  await expect(page.locator(".ic-worksheet-sheet-canvas")).toHaveScreenshot("formula-result.png", {
    maxDiffPixels: 0
  });
});

test("entering text in B2 persists after navigating away and back", async ({
  page,
}) => {
  await page.goto("/");
  await waitForCanvas(page);

  await clickCell(page, 2, 2); // B2
  await page.keyboard.type("hello");
  await page.keyboard.press("Enter");

  // Move away then come back.
  await clickCell(page, 1, 1);
  await clickCell(page, 2, 2);
  expect(await getFormulaBarValue(page)).toBe("hello");
});

test("pressing enter goes to cell below", async ({ page }) => {
  await page.goto("/");
  await waitForCanvas(page);

  await clickCell(page, 2, 2); // B2
  await page.keyboard.press("Enter");

  // Visually check that C2 is selected
  await expect(page.locator(".ic-worksheet-sheet-canvas")).toHaveScreenshot("enter.png", {
    maxDiffPixels: 0
  });
});
