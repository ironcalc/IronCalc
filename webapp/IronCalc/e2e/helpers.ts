import type { Page } from "@playwright/test";

// From worksheetCanvas.ts
const HEADER_ROW_HEIGHT = 28;
const HEADER_COLUMN_WIDTH = 30;

// From base/src/constants.rs (DEFAULT_COLUMN_WIDTH, DEFAULT_ROW_HEIGHT)
const DEFAULT_COLUMN_WIDTH = 125;
const DEFAULT_ROW_HEIGHT = 28;

/**
 * Returns the canvas-relative pixel center of a cell (1-indexed col/row).
 * Assumes default column widths and row heights — override sizes if the sheet
 * has custom dimensions.
 */
export function cellPosition(
  col: number,
  row: number,
  colWidth = DEFAULT_COLUMN_WIDTH,
  rowHeight = DEFAULT_ROW_HEIGHT,
): { x: number; y: number } {
  return {
    x: HEADER_COLUMN_WIDTH + (col - 1) * colWidth + colWidth / 2,
    y: HEADER_ROW_HEIGHT + (row - 1) * rowHeight + rowHeight / 2,
  };
}

/** Clicks the center of a cell on the worksheet canvas. */
export async function clickCell(
  page: Page,
  col: number,
  row: number,
): Promise<void> {
  const canvas = page.locator(".ic-worksheet-sheet-canvas");
  const box = await canvas.boundingBox();
  if (!box) throw new Error("Canvas not found or not visible");
  const { x, y } = cellPosition(col, row);
  // Use page.mouse so overlays (cell outline, etc.) don't intercept the click.
  await page.mouse.click(box.x + x, box.y + y);
}

/**
 * Waits for the canvas to be ready.
 * testing.ironcalc.com shows a "Welcome" modal on first load — this helper
 * dismisses it by clicking "Create workbook" (blank workbook is pre-selected).
 */
export async function waitForCanvas(page: Page): Promise<void> {
  await page.locator(".ic-worksheet-sheet-canvas").waitFor({ state: "visible" });

  const backdrop = page.locator(".app-ic-wd-backdrop");
  if (await backdrop.isVisible()) {
    await page.getByRole("button", { name: "Create workbook" }).click();
    await backdrop.waitFor({ state: "hidden" });
  }
}

/**
 * Returns the current value shown in the formula bar textarea.
 * This is the raw cell value or formula for the selected cell.
 */
export async function getFormulaBarValue(page: Page): Promise<string> {
  return page
    .locator(".ic-formula-bar-editor-wrapper textarea")
    .inputValue();
}
