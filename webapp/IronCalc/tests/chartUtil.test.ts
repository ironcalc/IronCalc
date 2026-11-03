import { describe, expect, test } from "vitest";
import {
  expandRegion,
  formatAbsoluteRange,
  formatCell,
  manualToRange,
  parseCell,
  parseRange,
  seriesNameFormula,
  seriesNameText,
} from "../src/components/RightDrawer/Chart/chartUtil";

describe("expandRegion", () => {
  // A 3x2 table at B2:C4 with a gap around it
  const content = new Set(["2,2", "2,3", "3,2", "3,3", "4,2", "4,3", "6,2"]);
  const isEmpty = (row: number, column: number) =>
    !content.has(`${row},${column}`);
  const limits = { lastRow: 100, lastColumn: 100 };

  test("grows to the block around the cell", () => {
    expect(expandRegion(isEmpty, 3, 3, limits)).toEqual([2, 2, 4, 3]);
    expect(expandRegion(isEmpty, 2, 2, limits)).toEqual([2, 2, 4, 3]);
  });

  test("an empty cell next to the block joins it", () => {
    expect(expandRegion(isEmpty, 5, 2, limits)).toEqual([2, 2, 6, 3]);
  });

  test("an isolated cell stays alone", () => {
    expect(expandRegion(isEmpty, 20, 20, limits)).toEqual([20, 20, 20, 20]);
  });

  test("stops at the sheet edges", () => {
    expect(
      expandRegion(() => false, 1, 1, { lastRow: 3, lastColumn: 2 }),
    ).toEqual([1, 1, 3, 2]);
  });
});

describe("parseRange", () => {
  test("reads absolute and relative references with a sheet", () => {
    expect(parseRange("Sheet1!$B$2:$B$5")).toEqual({
      sheet: "Sheet1",
      rowStart: 2,
      columnStart: 2,
      rowEnd: 5,
      columnEnd: 2,
    });
    expect(parseRange("='My Sheet'!c3")).toEqual({
      sheet: "'My Sheet'",
      rowStart: 3,
      columnStart: 3,
      rowEnd: 3,
      columnEnd: 3,
    });
    expect(parseRange("B2:B5")).toBeNull();
    expect(parseRange("Sheet1!nonsense")).toBeNull();
    expect(parseRange("Product A")).toBeNull();
  });

  test("formats back in absolute form", () => {
    const range = parseRange("Sheet1!D7:B2");
    expect(range && formatAbsoluteRange(range)).toBe("Sheet1!$B$2:$D$7");
    const cell = parseRange("Sheet1!B2");
    expect(cell && formatAbsoluteRange(cell)).toBe("Sheet1!$B$2");
  });
});

describe("series names", () => {
  test("references stay, text becomes a literal, empty stays empty", () => {
    expect(seriesNameFormula("Sheet1!$C$1")).toBe("Sheet1!$C$1");
    expect(seriesNameFormula("=Sheet1!$C$1")).toBe("Sheet1!$C$1");
    expect(seriesNameFormula('Say "hi"')).toBe('"Say ""hi"""');
    expect(seriesNameFormula("  ")).toBe("");
  });

  test("literals lose their quotes for editing", () => {
    expect(seriesNameText('"Say ""hi"""')).toBe('Say "hi"');
    expect(seriesNameText("Sheet1!$C$1")).toBe("Sheet1!$C$1");
  });
});

describe("manualToRange", () => {
  test("columns next to the categories with names on top form a range", () => {
    expect(
      manualToRange("Sheet1!$B$2:$B$6", [
        { name: "Sheet1!$C$1", values: "Sheet1!$C$2:$C$6" },
        { name: "Sheet1!$D$1", values: "Sheet1!$D$2:$D$6" },
      ]),
    ).toEqual({ range: "Sheet1!$B$1:$D$6", orientation: "Columns" });
  });

  test("rows under the categories form a range by rows", () => {
    expect(
      manualToRange("Sheet1!$C$1:$E$1", [
        { name: "Sheet1!$B$2", values: "Sheet1!$C$2:$E$2" },
      ]),
    ).toEqual({ range: "Sheet1!$B$1:$E$2", orientation: "Rows" });
  });

  test("anything else is null", () => {
    // a gap between the categories and the series
    expect(
      manualToRange("Sheet1!$B$2:$B$6", [
        { name: "", values: "Sheet1!$D$2:$D$6" },
      ]),
    ).toBeNull();
    // different sheets
    expect(
      manualToRange("Sheet1!$B$2:$B$6", [
        { name: "", values: "Sheet2!$C$2:$C$6" },
      ]),
    ).toBeNull();
    // no series
    expect(manualToRange("Sheet1!$B$2:$B$6", [])).toBeNull();
  });
});

describe("cells", () => {
  test("parse and format a cell", () => {
    expect(parseCell("F3")).toEqual({ row: 3, column: 6 });
    expect(parseCell(" $aa$10 ")).toEqual({ row: 10, column: 27 });
    expect(parseCell("F0")).toBeNull();
    expect(parseCell("Sheet1!F3")).toBeNull();
    expect(parseCell("3F")).toBeNull();
    expect(formatCell(3, 6)).toBe("F3");
    expect(formatCell(10, 27)).toBe("AA10");
  });
});
