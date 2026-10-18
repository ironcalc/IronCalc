// Unit tests for the scroll geometry of the WorksheetCanvas: the scroll
// offset of the scrolling pane, the cells on screen, the cell <-> canvas
// coordinate conversions and `reveal`.

import { expect, test } from "vitest";
import {
  headerColumnWidth,
  headerRowHeight,
  LAST_COLUMN,
  LAST_ROW,
} from "../../src/components/WorksheetCanvas/constants";
import { newModel, renderWorksheet } from "./harness";

// Default cell geometry (base/src/constants.rs) and the frozen-pane
// separator (worksheetCanvas.ts)
const COLUMN_WIDTH = 90;
const ROW_HEIGHT = 25;
const FROZEN_SEPARATOR = 3;

// The harness renders a 430x230 canvas: a 400x202 scrolling pane without
// frozen panes
const PANE_WIDTH = 430 - headerColumnWidth;
const PANE_HEIGHT = 230 - headerRowHeight;

test("the cells on screen run from the scroll offset to the pane's corner", async () => {
  const model = await newModel();
  const { worksheet } = await renderWorksheet(model);
  expect(worksheet.getScroll()).toEqual({ x: 0, y: 0 });
  // 202 / 25 = 8.08 rows and 400 / 90 = 4.4 columns fit: the ninth row and
  // the fifth column hold the corner
  expect(worksheet.getVisibleCells()).toEqual({
    topLeftCell: { row: 1, column: 1 },
    bottomRightCell: { row: 9, column: 5 },
  });
});

test("a row or column ending exactly at the pane's edge is the last one", async () => {
  const model = await newModel();
  // Four columns of 90 fit exactly in a 360 wide pane
  const { worksheet } = await renderWorksheet(model, {
    width: headerColumnWidth + 4 * COLUMN_WIDTH,
    height: headerRowHeight + 3 * ROW_HEIGHT,
  });
  expect(worksheet.getVisibleCells().bottomRightCell).toEqual({
    row: 3,
    column: 4,
  });
});

test("the DOM scroller's position is the offset; the engine's view is the cell at it", async () => {
  const model = await newModel();
  const { worksheet } = await renderWorksheet(model);
  // 60px down is in row 3 (50-75), 100px right is in column B (90-180)
  worksheet.setScrollPosition({ left: 100, top: 60 });
  expect(worksheet.getScroll()).toEqual({ x: 100, y: 60 });
  const view = model.getSelectedView();
  expect([view.top_row, view.left_column]).toEqual([3, 2]);
  expect(worksheet.getVisibleCells().topLeftCell).toEqual({
    row: 3,
    column: 2,
  });
  // The cell at the offset starts before the pane's corner; the cells
  // before it are laid out before the pane
  expect(worksheet.getCoordinatesByCell(3, 2)).toEqual([
    headerColumnWidth - 10,
    headerRowHeight - 10,
  ]);
  expect(worksheet.getCoordinatesByCell(1, 1)).toEqual([
    headerColumnWidth - 100,
    headerRowHeight - 60,
  ]);
});

test("the offset is clamped to the sheet", async () => {
  const model = await newModel();
  const { worksheet } = await renderWorksheet(model);
  worksheet.setScroll(-5, -5);
  expect(worksheet.getScroll()).toEqual({ x: 0, y: 0 });
  worksheet.setScroll(1e12, 1e12);
  const max = {
    x: LAST_COLUMN * COLUMN_WIDTH - PANE_WIDTH,
    y: LAST_ROW * ROW_HEIGHT - PANE_HEIGHT,
  };
  expect(worksheet.getMaxScroll()).toEqual(max);
  expect(worksheet.getScroll()).toEqual(max);
  expect(worksheet.getVisibleCells().bottomRightCell).toEqual({
    row: LAST_ROW,
    column: LAST_COLUMN,
  });
  expect(worksheet.getSheetDimensions()).toEqual([
    headerColumnWidth + LAST_COLUMN * COLUMN_WIDTH,
    headerRowHeight + LAST_ROW * ROW_HEIGHT,
  ]);
});

test("when the engine moves its view the offset follows", async () => {
  const model = await newModel();
  const { worksheet } = await renderWorksheet(model);
  // Keyboard navigation, links and sheet switches move the engine's top-left
  // cell without going through the canvas
  model.setTopLeftVisibleCell(15, 3);
  expect(worksheet.getScroll()).toEqual({
    x: 2 * COLUMN_WIDTH,
    y: 14 * ROW_HEIGHT,
  });
  expect(worksheet.getCoordinatesByCell(15, 3)).toEqual([
    headerColumnWidth,
    headerRowHeight,
  ]);
});

test("a sub-cell offset lays the cells out across the pane's corner", async () => {
  const model = await newModel();
  const { worksheet } = await renderWorksheet(model);
  worksheet.setScroll(40, 10);
  expect(worksheet.getScroll()).toEqual({ x: 40, y: 10 });
  // The cells holding the offset are the first ones, partly out of view
  expect(worksheet.getVisibleCells()).toEqual({
    topLeftCell: { row: 1, column: 1 },
    bottomRightCell: { row: 9, column: 5 },
  });
  expect(worksheet.getCoordinatesByCell(1, 1)).toEqual([
    headerColumnWidth - 40,
    headerRowHeight - 10,
  ]);
  expect(worksheet.getCoordinatesByCell(2, 2)).toEqual([
    headerColumnWidth + COLUMN_WIDTH - 40,
    headerRowHeight + ROW_HEIGHT - 10,
  ]);
  // The engine's top-left cell is the one at the offset
  const view = model.getSelectedView();
  expect([view.top_row, view.left_column]).toEqual([1, 1]);
  // The pane's corner is in A1, 15px below its top edge the second row starts
  expect(
    worksheet.getCellByCoordinates(headerColumnWidth, headerRowHeight),
  ).toEqual({ row: 1, column: 1 });
  expect(
    worksheet.getCellByCoordinates(headerColumnWidth, headerRowHeight + 15),
  ).toEqual({ row: 2, column: 1 });
});

test("reveal scrolls the least needed to show a cell", async () => {
  const model = await newModel();
  const { worksheet } = await renderWorksheet(model);
  // Already on screen: no change
  expect(worksheet.reveal(5, 2)).toBe(false);
  expect(worksheet.getScroll()).toEqual({ x: 0, y: 0 });
  // Below the pane: its bottom edge lands on the pane's bottom edge
  expect(worksheet.reveal(20, 1)).toBe(true);
  expect(worksheet.getScroll()).toEqual({
    x: 0,
    y: 20 * ROW_HEIGHT - PANE_HEIGHT,
  });
  const [, y] = worksheet.getCoordinatesByCell(20, 1);
  expect(y + ROW_HEIGHT).toBe(headerRowHeight + PANE_HEIGHT);
  expect(worksheet.reveal(20, 1)).toBe(false);
  // The engine's view is the row at the offset (298px is in row 12)
  expect(model.getSelectedView().top_row).toBe(12);
  // Right of the pane
  expect(worksheet.reveal(20, 20)).toBe(true);
  expect(worksheet.getScroll().x).toBe(20 * COLUMN_WIDTH - PANE_WIDTH);
  // Above and left of the pane: its top-left corner lands on the pane's
  expect(worksheet.reveal(3, 2)).toBe(true);
  expect(worksheet.getScroll()).toEqual({
    x: COLUMN_WIDTH,
    y: 2 * ROW_HEIGHT,
  });
});

test("reveal of a cell taller than the pane shows its top", async () => {
  const model = await newModel();
  model.setRowsHeight(0, 5, 5, 400);
  const { worksheet } = await renderWorksheet(model);
  expect(worksheet.reveal(5, 1)).toBe(true);
  expect(worksheet.getScroll().y).toBe(4 * ROW_HEIGHT);
});

test("reveal of a merged cell shows the whole range", async () => {
  const model = await newModel();
  // B2:B6
  model.mergeCells({ sheet: 0, row: 2, column: 2, width: 1, height: 5 });
  const { worksheet } = await renderWorksheet(model);
  worksheet.setScroll(0, 9 * ROW_HEIGHT);
  expect(worksheet.reveal(4, 2)).toBe(true);
  expect(worksheet.getScroll().y).toBe(ROW_HEIGHT);
  worksheet.setScroll(0, 0);
  // From above the range reveals down to its last row
  expect(worksheet.reveal(2, 2)).toBe(false);
  worksheet.setScroll(0, 0);
  model.setRowsHeight(0, 6, 6, 200);
  worksheet.renderSheet();
  expect(worksheet.reveal(2, 2)).toBe(true);
  // Rows 2-6 are 4 x 25 + 200 = 300 tall, more than the pane: the top of
  // the range stays on screen
  expect(worksheet.getScroll().y).toBe(ROW_HEIGHT);
});

test("frozen panes: the offset counts from the first unfrozen cell", async () => {
  const model = await newModel();
  model.setFrozenRowsCount(0, 2);
  model.setFrozenColumnsCount(0, 1);
  const { worksheet } = await renderWorksheet(model);
  const frozenHeight = 2 * ROW_HEIGHT + FROZEN_SEPARATOR;
  const frozenWidth = COLUMN_WIDTH + FROZEN_SEPARATOR;
  const paneHeight = PANE_HEIGHT - frozenHeight;
  const paneWidth = PANE_WIDTH - frozenWidth;
  expect(worksheet.getVisibleCells().topLeftCell).toEqual({
    row: 3,
    column: 2,
  });
  // Frozen cells never scroll
  expect(worksheet.reveal(1, 1)).toBe(false);
  expect(worksheet.reveal(20, 1)).toBe(true);
  expect(worksheet.getScroll()).toEqual({
    x: 0,
    y: 18 * ROW_HEIGHT - paneHeight,
  });
  const [, y] = worksheet.getCoordinatesByCell(20, 1);
  expect(y + ROW_HEIGHT).toBe(230);
  // Frozen cells are laid out in their panes
  expect(worksheet.getCoordinatesByCell(2, 1)).toEqual([
    headerColumnWidth,
    headerRowHeight + ROW_HEIGHT,
  ]);
  // The first unfrozen row starts at the pane's corner, less the scroll
  expect(worksheet.getCoordinatesByCell(3, 2)).toEqual([
    headerColumnWidth + frozenWidth,
    headerRowHeight + frozenHeight - worksheet.getScroll().y,
  ]);
  expect(worksheet.reveal(3, 10)).toBe(true);
  expect(worksheet.getScroll().x).toBe(9 * COLUMN_WIDTH - paneWidth);
  expect(worksheet.getMaxScroll()).toEqual({
    x: (LAST_COLUMN - 1) * COLUMN_WIDTH - paneWidth,
    y: (LAST_ROW - 2) * ROW_HEIGHT - paneHeight,
  });
  // The engine's view never points inside the frozen panes
  worksheet.setScroll(0, 0);
  const view = model.getSelectedView();
  expect([view.top_row, view.left_column]).toEqual([3, 2]);
});

test("cells by canvas coordinates, with frozen panes", async () => {
  const model = await newModel();
  model.setFrozenRowsCount(0, 2);
  model.setFrozenColumnsCount(0, 1);
  const { worksheet } = await renderWorksheet(model);
  worksheet.setScroll(COLUMN_WIDTH, 4 * ROW_HEIGHT);
  const frozenHeight = 2 * ROW_HEIGHT + FROZEN_SEPARATOR;
  const frozenWidth = COLUMN_WIDTH + FROZEN_SEPARATOR;
  // In the frozen corner
  expect(
    worksheet.getCellByCoordinates(headerColumnWidth + 1, headerRowHeight + 1),
  ).toEqual({ row: 1, column: 1 });
  expect(
    worksheet.getCellByCoordinates(
      headerColumnWidth + 1,
      headerRowHeight + ROW_HEIGHT + 1,
    ),
  ).toEqual({ row: 2, column: 1 });
  // In the frozen rows, right of the frozen columns: column at the scroll
  expect(
    worksheet.getCellByCoordinates(
      headerColumnWidth + frozenWidth + 1,
      headerRowHeight + 1,
    ),
  ).toEqual({ row: 1, column: 3 });
  // In the scrolling pane: the row and column at the scroll
  expect(
    worksheet.getCellByCoordinates(
      headerColumnWidth + frozenWidth + 1,
      headerRowHeight + frozenHeight + 1,
    ),
  ).toEqual({ row: 7, column: 3 });
  // The separators belong to the frozen panes: the last frozen cell
  expect(
    worksheet.getCellByCoordinates(
      headerColumnWidth + COLUMN_WIDTH + 1,
      headerRowHeight + 2 * ROW_HEIGHT + 1,
    ),
  ).toEqual({ row: 2, column: 1 });
  // Over the headers: a frozen cell
  expect(worksheet.getCellByCoordinates(10, 10)).toEqual({
    row: 1,
    column: 1,
  });
});

test("cells by canvas coordinates past the headers, without frozen panes", async () => {
  const model = await newModel();
  const { worksheet } = await renderWorksheet(model);
  worksheet.setScroll(2 * COLUMN_WIDTH, 4 * ROW_HEIGHT);
  // Over the headers: the cells just before the pane, so a drag past the
  // headers extends the selection out of view
  expect(worksheet.getCellByCoordinates(10, 10)).toEqual({
    row: 4,
    column: 2,
  });
  // Far past them: the sheet's first cell
  expect(worksheet.getCellByCoordinates(-1000, -1000)).toEqual({
    row: 1,
    column: 1,
  });
  // Past the canvas: cells out of view to the right and below
  expect(worksheet.getCellByCoordinates(1000, 1000)).toEqual({
    row: 5 + Math.floor((1000 - headerRowHeight) / ROW_HEIGHT),
    column: 3 + Math.floor((1000 - headerColumnWidth) / COLUMN_WIDTH),
  });
});

test("hidden rows and columns take no space", async () => {
  const model = await newModel();
  model.setRowsHidden(0, 2, 2, true);
  model.setColumnsHidden(0, 1, 1, true);
  const { worksheet } = await renderWorksheet(model);
  expect(worksheet.getCoordinatesByCell(3, 2)).toEqual([
    headerColumnWidth,
    headerRowHeight + ROW_HEIGHT,
  ]);
  // Scrolling into a hidden row lands on the next visible one
  worksheet.setScrollPosition({ left: 0, top: ROW_HEIGHT });
  expect(worksheet.getScroll()).toEqual({ x: 0, y: ROW_HEIGHT });
  expect(model.getSelectedView().top_row).toBe(3);
  expect(worksheet.getVisibleCells().topLeftCell).toEqual({
    row: 3,
    column: 1,
  });
  // A size change is picked up by the next render
  model.setColumnsHidden(0, 1, 1, false);
  worksheet.renderSheet();
  expect(worksheet.getCoordinatesByCell(3, 2)).toEqual([
    headerColumnWidth + COLUMN_WIDTH,
    headerRowHeight,
  ]);
});
