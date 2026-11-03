import { describe, expect, test } from "vitest";
import { Axis } from "../src/components/WorksheetCanvas/axis";
import {
  anchorFromSheetRect,
  MIN_CHART_SIZE,
  movedRect,
  resizedRect,
  sheetRectFromAnchor,
} from "../src/components/WorksheetCanvas/chartInteraction";

// 100px columns, 20px rows, except column 3 which is 50px and row 2 which is hidden
const axes = {
  columns: new Axis(100, 50, [[3, 3, 50]]),
  rows: new Axis(20, 100, [[2, 2, 0]]),
};

describe("anchors and sheet rectangles", () => {
  test("a rectangle becomes cells plus offsets and back", () => {
    const rect = { x: 130, y: 45, width: 240, height: 70 };
    const anchor = anchorFromSheetRect(axes, rect);
    // 130px is 30px into column 2; 45px is 5px into row 4 (row 2 is hidden,
    // so rows 1, 3 and 4 cover the first 60px)
    expect(anchor.from).toEqual({
      row: 4,
      column: 2,
      row_offset: 5 * 9525,
      column_offset: 30 * 9525,
    });
    // 370px: columns 1, 2 (200), 3 (50) and 4 (100) end at 350, so 20px
    // into column 5
    expect(anchor.to).toEqual({
      row: 7,
      column: 5,
      row_offset: 15 * 9525,
      column_offset: 20 * 9525,
    });
    expect(anchor.behavior).toBe("MoveAndResize");
    expect(sheetRectFromAnchor(axes, anchor)).toEqual(rect);
  });

  test("a corner on a cell boundary has no offset", () => {
    const anchor = anchorFromSheetRect(axes, {
      x: 100,
      y: 20,
      width: 100,
      height: 20,
    });
    expect(anchor.from).toEqual({
      row: 3,
      column: 2,
      row_offset: 0,
      column_offset: 0,
    });
    expect(anchor.to).toEqual({
      row: 4,
      column: 3,
      row_offset: 0,
      column_offset: 0,
    });
  });

  test("the behavior is kept", () => {
    const anchor = anchorFromSheetRect(
      axes,
      { x: 0, y: 0, width: 10, height: 10 },
      "Fixed",
    );
    expect(anchor.behavior).toBe("Fixed");
  });
});

describe("drag geometry", () => {
  const start = { x: 100, y: 100, width: 200, height: 120 };

  test("moving keeps the size", () => {
    expect(movedRect(start, 15, -30)).toEqual({
      x: 115,
      y: 70,
      width: 200,
      height: 120,
    });
  });

  test("resizing from a corner keeps the opposite corner", () => {
    expect(resizedRect(start, "se", 20, 10)).toEqual({
      x: 100,
      y: 100,
      width: 220,
      height: 130,
    });
    expect(resizedRect(start, "nw", 20, 10)).toEqual({
      x: 120,
      y: 110,
      width: 180,
      height: 110,
    });
    expect(resizedRect(start, "e", -50, 999)).toEqual({
      x: 100,
      y: 100,
      width: 150,
      height: 120,
    });
  });

  test("a chart cannot shrink below the minimum", () => {
    expect(resizedRect(start, "se", -500, -500)).toEqual({
      x: 100,
      y: 100,
      width: MIN_CHART_SIZE,
      height: MIN_CHART_SIZE,
    });
    expect(resizedRect(start, "nw", 500, 500)).toEqual({
      x: 300 - MIN_CHART_SIZE,
      y: 220 - MIN_CHART_SIZE,
      width: MIN_CHART_SIZE,
      height: MIN_CHART_SIZE,
    });
  });
});
