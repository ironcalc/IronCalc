// Unit tests for the geometry of the DOM overlays (the selection outline,
// the area outline and the autofill handle). These are HTML elements laid on
// top of the canvas, so the screenshot tests never see them; here the real
// WorksheetCanvas renders against the fake DOM and the tests assert the
// exact pixel geometry it assigned to the overlay styles.
//
// The interesting cases are frozen panes: overlays must be laid out in
// *screen* geometry, where rows/columns scrolled behind the frozen separator
// take no space, not in sheet geometry (the plain sum of cell sizes).

import { expect, test } from "vitest";
import {
  headerColumnWidth,
  headerRowHeight,
} from "../../src/components/WorksheetCanvas/constants";
import { newModel, renderWorksheet } from "./harness";

// Default cell geometry (base/src/constants.rs) and the frozen-pane
// separator (worksheetCanvas.ts, not importable here: that module needs the
// fake DOM installed before it loads)
const COLUMN_WIDTH = 90;
const ROW_HEIGHT = 25;
const FROZEN_SEPARATOR = 3;

const px = (value: string): number => {
  expect(value).toMatch(/^-?[\d.]+px$/);
  return Number(value.slice(0, -2));
};

const rect = (element: HTMLElement) => ({
  left: px(element.style.left),
  top: px(element.style.top),
  width: px(element.style.width),
  height: px(element.style.height),
});

// The handle div is centered on its corner: left = x - width / 2 - 1, and
// the fake getBoundingClientRect reports a 6x6 handle
const handleCorner = (element: HTMLElement) => ({
  x: px(element.style.left) + 4,
  y: px(element.style.top) + 4,
});

function cell(row: number, column: number, width = 1, height = 1) {
  return { sheet: 0, row, column, width, height };
}

// Screen x of the left edge of a column in the top-left pane
const columnX = (column: number): number =>
  headerColumnWidth + (column - 1) * COLUMN_WIDTH;
// Screen y of the top edge of a row in the top-left pane
const rowY = (row: number): number => headerRowHeight + (row - 1) * ROW_HEIGHT;

test("a single selected cell: outline hugs the cell, handle on its corner", async () => {
  const model = await newModel();
  model.setSelectedCell(2, 2);
  const { worksheet } = await renderWorksheet(model);

  const [x, y] = [columnX(2), rowY(2)];
  expect(rect(worksheet.cellOutline)).toEqual({
    left: x - 1,
    top: y - 1,
    width: COLUMN_WIDTH - 1,
    height: ROW_HEIGHT - 1,
  });
  // A single cell shows no area outline
  expect(worksheet.areaOutline.style.visibility).toBe("hidden");
  expect(handleCorner(worksheet.cellOutlineHandle)).toEqual({
    x: x + COLUMN_WIDTH,
    y: y + ROW_HEIGHT,
  });
});

test("a selected merged cell: outline covers the merged rectangle", async () => {
  const model = await newModel();
  // B2:C3 merged, 2 columns by 2 rows
  model.mergeCells(cell(2, 2, 2, 2));
  model.setSelectedCell(2, 2);
  const { worksheet } = await renderWorksheet(model);

  const [x, y] = [columnX(2), rowY(2)];
  expect(rect(worksheet.cellOutline)).toEqual({
    left: x - 1,
    top: y - 1,
    width: 2 * COLUMN_WIDTH - 1,
    height: 2 * ROW_HEIGHT - 1,
  });
  expect(worksheet.areaOutline.style.visibility).toBe("hidden");
  expect(handleCorner(worksheet.cellOutlineHandle)).toEqual({
    x: x + 2 * COLUMN_WIDTH,
    y: y + 2 * ROW_HEIGHT,
  });
});

// Five frozen rows, scrolled down so rows 6-14 are hidden behind the frozen
// separator, and B3:B18 merged across the frozen line. On screen the merged
// cell spans rows 3-5 (frozen), the separator and rows 15-18 — NOT the sum
// of its sixteen row heights.
test("a merged cell across the frozen line: outline spans only the visible rows", async () => {
  const model = await newModel();
  model.setFrozenRowsCount(0, 5);
  model.setTopLeftVisibleCell(15, 1);
  model.mergeCells(cell(3, 2, 1, 16));
  model.setSelectedCell(3, 2);
  const { worksheet } = await renderWorksheet(model, { height: 300 });

  const [x, y] = [columnX(2), rowY(3)];
  // Rows 3-5 in the frozen pane, then the separator, then rows 15-18
  const heightOnScreen = 3 * ROW_HEIGHT + FROZEN_SEPARATOR + 4 * ROW_HEIGHT;
  expect(rect(worksheet.cellOutline)).toEqual({
    left: x - 1,
    top: y - 1,
    width: COLUMN_WIDTH - 1,
    height: heightOnScreen - 1,
  });
  expect(worksheet.areaOutline.style.visibility).toBe("hidden");
  // The handle sits on the on-screen bottom-right corner of B18
  expect(handleCorner(worksheet.cellOutlineHandle)).toEqual({
    x: x + COLUMN_WIDTH,
    y: y + heightOnScreen,
  });
});

// Six frozen rows, scrolled down to row 35, and B4:B34 merged: the merge
// starts in the frozen pane and disappears into the hidden band behind the
// separator. Selecting it must show the outline over the frozen part (rows
// 4-6 down to the separator), with no bottom edge (the cell continues under
// the separator) and no handle (its corner is not on screen).
test("a merged cell reaching from the frozen pane into the hidden band keeps its outline", async () => {
  const model = await newModel();
  model.setFrozenRowsCount(0, 6);
  model.setTopLeftVisibleCell(35, 1);
  model.mergeCells(cell(4, 2, 1, 31));
  model.setSelectedCell(4, 2);
  const { worksheet } = await renderWorksheet(model);

  const [x, y] = [columnX(2), rowY(4)];
  // Rows 4-6 in the frozen pane plus the separator the merge vanishes under
  const heightOnScreen = 3 * ROW_HEIGHT + FROZEN_SEPARATOR;
  expect(worksheet.cellOutline.style.visibility).toBe("visible");
  expect(rect(worksheet.cellOutline)).toEqual({
    left: x - 1,
    top: y - 1,
    width: COLUMN_WIDTH - 1,
    height: heightOnScreen - 1,
  });
  expect(worksheet.cellOutline.style.borderBottom).toBe("none");
  expect(worksheet.cellOutline.style.borderRight).toBe("");
  expect(worksheet.cellOutlineHandle.style.visibility).toBe("hidden");
});

// A merged cell that lies entirely in the hidden band shows no outline at all
test("a merged cell entirely scrolled behind the frozen pane hides its outline", async () => {
  const model = await newModel();
  model.setFrozenRowsCount(0, 6);
  model.setTopLeftVisibleCell(35, 1);
  model.mergeCells(cell(10, 2, 1, 11));
  model.setSelectedCell(10, 2);
  const { worksheet } = await renderWorksheet(model);

  expect(worksheet.cellOutline.style.visibility).toBe("hidden");
  expect(worksheet.cellOutlineHandle.style.visibility).toBe("hidden");
});

// The same geometry for a plain (unmerged) selection B3:B18 across the
// frozen line, drawn by the area outline
test("a selected area across the frozen line: area outline spans only the visible rows", async () => {
  const model = await newModel();
  model.setFrozenRowsCount(0, 5);
  model.setTopLeftVisibleCell(15, 1);
  model.setSelectedCell(3, 2);
  model.setSelectedRange(3, 2, 18, 2);
  const { worksheet } = await renderWorksheet(model, { height: 300 });

  const [x, y] = [columnX(2), rowY(3)];
  const heightOnScreen = 3 * ROW_HEIGHT + FROZEN_SEPARATOR + 4 * ROW_HEIGHT;
  expect(worksheet.areaOutline.style.visibility).toBe("visible");
  expect(rect(worksheet.areaOutline)).toEqual({
    left: x,
    top: y,
    width: COLUMN_WIDTH - 1,
    height: heightOnScreen - 1,
  });
  expect(handleCorner(worksheet.cellOutlineHandle)).toEqual({
    x: x + COLUMN_WIDTH,
    y: y + heightOnScreen,
  });
});
