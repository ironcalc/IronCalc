import { readFile } from "node:fs/promises";
import { initSync, Model } from "@ironcalc/wasm";
import { beforeAll, expect, test } from "vitest";
import { Axis } from "../../src/components/WorksheetCanvas/axis";
import {
  LAST_COLUMN,
  LAST_ROW,
} from "../../src/components/WorksheetCanvas/constants";

test("uniform axis", () => {
  const a = Axis.uniform(10, 100);
  expect(a.start(1)).toBe(0);
  expect(a.start(5)).toBe(40);
  expect(a.total()).toBe(1000);
  expect(a.at(0)).toBe(1);
  expect(a.at(39.9)).toBe(4);
  expect(a.at(40)).toBe(5);
  expect(a.at(5000)).toBe(100);
  expect(a.size(7)).toBe(10);
});

test("custom sizes and hidden", () => {
  // Rows 3-4 are 30 tall, row 6 is hidden, row 7 is 5 tall (unsorted input,
  // default dropped)
  const a = new Axis(10, 20, [
    [7, 7, 5],
    [3, 3, 30],
    [4, 4, 30],
    [6, 6, 0],
    [9, 9, 10],
  ]);
  expect(a.start(3)).toBe(20);
  expect(a.start(4)).toBe(50);
  expect(a.start(5)).toBe(80);
  expect(a.start(6)).toBe(90);
  expect(a.start(7)).toBe(90);
  expect(a.start(8)).toBe(95);
  expect(a.total()).toBe(95 + 13 * 10);
  expect(a.size(6)).toBe(0);
  expect(a.size(7)).toBe(5);
  expect(a.at(79)).toBe(4);
  expect(a.at(80)).toBe(5);
  expect(a.at(90)).toBe(7);
  expect(a.at(94)).toBe(7);
  expect(a.at(95)).toBe(8);
  expect(a.at(1e9)).toBe(20);
  expect(a.nextVisible(6)).toBe(7);
  expect(a.prevVisible(6)).toBe(5);
  expect(a.nextVisible(21)).toBe(null);
  // Every offset maps back to the index whose span holds it
  for (let i = 1; i <= 20; i += 1) {
    if (a.size(i) > 0) {
      expect(a.at(a.start(i)), `row ${i}`).toBe(i);
      expect(a.at(a.start(i) + a.size(i) - 0.01), `row ${i}`).toBe(i);
    }
  }
});

test("column ranges", () => {
  const a = new Axis(90, 16_384, [
    [2, 3, 50],
    [4, 4, 50],
    [10, 12, 0],
  ]);
  expect(a.start(2)).toBe(90);
  expect(a.start(5)).toBe(90 + 150);
  expect(a.start(10)).toBe(240 + 5 * 90);
  expect(a.start(13)).toBe(a.start(10));
  expect(a.at(a.start(13))).toBe(13);
  expect(a.nextVisible(10)).toBe(13);
  expect(a.total()).toBe(90 * 16_384 - 120 - 270);
});

test("indices outside the axis are clamped", () => {
  const a = new Axis(10, 20, [[21, 25, 50]]);
  // A run past the end is dropped
  expect(a.total()).toBe(200);
  expect(a.start(0)).toBe(0);
  expect(a.start(100)).toBe(200);
  expect(a.size(0)).toBe(10);
  expect(a.at(-5)).toBe(1);
  expect(a.prevVisible(0)).toBe(null);
  expect(a.nextVisible(0)).toBe(1);
  expect(a.prevVisible(500)).toBe(20);
  // A sheet with every index hidden
  const hidden = new Axis(10, 5, [[1, 5, 0]]);
  expect(hidden.total()).toBe(0);
  expect(hidden.nextVisible(1)).toBe(null);
  expect(hidden.prevVisible(5)).toBe(null);
  expect(hidden.at(0)).toBe(1);
});

// Built from the engine's sizes, the axis must agree with the per-row and
// per-column queries the canvas uses today.

beforeAll(async () => {
  const buffer = await readFile("node_modules/@ironcalc/wasm/wasm_bg.wasm");
  initSync({ module: buffer });
});

test("row axis from the model matches getRowHeight", () => {
  const model = new Model("Workbook1", "en", "UTC", "en");
  model.setRowsHeight(0, 3, 4, 40);
  model.setRowsHeight(0, 7, 7, 12.6);
  model.setRowsHidden(0, 5, 5, true);
  const rows = Axis.fromSizes(model.getRowSizes(0), LAST_ROW);
  let y = 0;
  for (let row = 1; row <= 10; row += 1) {
    const height = Math.round(model.getRowHeight(0, row));
    expect(rows.start(row), `start of row ${row}`).toBe(y);
    expect(rows.size(row), `size of row ${row}`).toBe(height);
    y += height;
  }
  expect(rows.size(5)).toBe(0);
  expect(rows.size(7)).toBe(13);
  expect(rows.count).toBe(LAST_ROW);
  expect(rows.start(LAST_ROW + 1)).toBe(rows.total());
});

test("column axis from the model matches getColumnWidth", () => {
  const model = new Model("Workbook1", "en", "UTC", "en");
  model.setColumnsWidth(0, 2, 3, 120);
  model.setColumnsHidden(0, 4, 4, true);
  model.setColumnsWidth(0, 6, 6, 45.4);
  const columns = Axis.fromSizes(model.getColumnSizes(0), LAST_COLUMN);
  let x = 0;
  for (let column = 1; column <= 10; column += 1) {
    const width = Math.round(model.getColumnWidth(0, column));
    expect(columns.start(column), `start of column ${column}`).toBe(x);
    expect(columns.size(column), `size of column ${column}`).toBe(width);
    x += width;
  }
  expect(columns.size(4)).toBe(0);
  expect(columns.nextVisible(4)).toBe(5);
  expect(columns.at(columns.start(5))).toBe(5);
});
