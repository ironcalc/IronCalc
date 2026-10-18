// FrameCells answers the renderer's per-cell questions from blocks fetched
// in one call per pane; whatever it answers must be what the engine would
// have said cell by cell, inside the blocks and outside them.

import { BorderStyle, BorderType, type Model } from "@ironcalc/wasm";
import { expect, test } from "vitest";
import { FrameCells } from "../../src/components/WorksheetCanvas/frameCells";
import { newModel } from "./harness";

function cell(row: number, column: number) {
  return { sheet: 0, row, column, width: 1, height: 1 };
}

async function sampleModel(): Promise<Model> {
  const model = await newModel();
  model.setUserInput(0, 1, 1, "Title");
  model.setUserInput(0, 2, 2, "=1+1");
  model.setUserInput(0, 3, 3, "TRUE");
  model.setUserInput(0, 20, 20, "far away");
  model.updateRangeStyle(cell(2, 1), "fill.fg_color", "#FF0000");
  model.updateRangeStyle(cell(3, 1), "font.b", "true");
  model.updateRangeStyle(cell(2, 2), "num_fmt", "0.00");
  model.setAreaWithBorder(cell(2, 3), {
    item: { style: BorderStyle.Thin, color: "#00FF00" },
    type: BorderType.Bottom,
  });
  return model;
}

test("answers inside the blocks match the engine cell by cell", async () => {
  const model = await sampleModel();
  const frame = new FrameCells(model, 0, [
    { row: 1, column: 1, width: 4, height: 4 },
    { row: 10, column: 1, width: 2, height: 2 },
  ]);
  const inside = [
    ...[1, 2, 3, 4].flatMap((r) => [1, 2, 3, 4].map((c) => [r, c])),
    [10, 1],
    [11, 2],
  ];
  for (const [row, column] of inside) {
    expect(frame.getCellStyle(row, column), `style ${row},${column}`).toEqual(
      model.getCellStyle(0, row, column),
    );
    expect(frame.getFormattedCellValue(row, column)).toBe(
      model.getFormattedCellValue(0, row, column),
    );
    expect(frame.getCellType(row, column)).toBe(
      model.getCellType(0, row, column),
    );
  }
  // The formatted value and the border merged from the neighbour above
  expect(frame.getFormattedCellValue(2, 2)).toBe("2.00");
  expect(frame.getCellStyle(3, 3).style.border.top).not.toBeNull();
});

test("a cell outside the blocks goes to the engine", async () => {
  const model = await sampleModel();
  const frame = new FrameCells(model, 0, [
    { row: 1, column: 1, width: 4, height: 4 },
  ]);
  expect(frame.getFormattedCellValue(20, 20)).toBe("far away");
  expect(frame.getCellType(20, 20)).toBe(2);
  expect(frame.getCellStyle(20, 20)).toEqual(model.getCellStyle(0, 20, 20));
  expect(frame.getCellStyle(0, 1)).toEqual(model.getCellStyle(0, 0, 1));
});

test("empty areas are skipped and overlapping areas agree", async () => {
  const model = await sampleModel();
  const frame = new FrameCells(model, 0, [
    { row: 1, column: 1, width: 0, height: 3 },
    { row: 1, column: 1, width: 3, height: 0 },
    { row: 1, column: 1, width: 3, height: 3 },
    { row: 2, column: 2, width: 3, height: 3 },
  ]);
  expect(frame.getFormattedCellValue(1, 1)).toBe("Title");
  expect(frame.getFormattedCellValue(2, 2)).toBe("2.00");
  expect(frame.getFormattedCellValue(4, 4)).toBe("");
  expect(frame.getCellStyle(3, 3)).toEqual(model.getCellStyle(0, 3, 3));
});

test("fills are known per cell and as runs per row of an area", async () => {
  const model = await sampleModel();
  model.updateRangeStyle(
    { sheet: 0, row: 2, column: 1, width: 3, height: 1 },
    "fill.fg_color",
    "#FF0000",
  );
  const frame = new FrameCells(model, 0, [
    { row: 1, column: 1, width: 0, height: 0 },
    { row: 1, column: 1, width: 4, height: 3 },
  ]);
  expect(frame.isFilled(2, 1)).toBe(true);
  expect(frame.isFilled(2, 4)).toBe(false);
  expect(frame.isFilled(1, 1)).toBe(false);
  // outside the areas: the engine answers
  expect(frame.isFilled(20, 20)).toBe(false);
  expect(frame.fillRuns(0, 2)).toEqual([]);
  expect(frame.fillRuns(1, 2)).toEqual([
    { row: 2, column: 1, width: 3, color: "#FF0000" },
    { row: 2, column: 4, width: 1, color: "" },
  ]);
  expect(frame.fillRuns(1, 1)).toEqual([
    { row: 1, column: 1, width: 4, color: "" },
  ]);
  expect(frame.fillRuns(1, 7)).toEqual([]);
});

test("the sheet and its grid lines flag are recorded", async () => {
  const model = await sampleModel();
  model.setShowGridLines(0, false);
  const frame = new FrameCells(model, 0, [
    { row: 1, column: 1, width: 2, height: 2 },
  ]);
  expect(frame.sheet).toBe(0);
  expect(frame.showGridLines).toBe(false);
});

test("colours resolve as the engine resolves them, once per colour", async () => {
  const model = await sampleModel();
  let calls = 0;
  const counting = new Proxy(model, {
    get(target, property, receiver) {
      const value = Reflect.get(target, property, receiver);
      if (property === "resolveColor") {
        return (...args: unknown[]) => {
          calls += 1;
          return (value as (...a: unknown[]) => string).apply(target, args);
        };
      }
      return typeof value === "function" ? value.bind(target) : value;
    },
  });
  const frame = new FrameCells(counting, 0, [
    { row: 1, column: 1, width: 2, height: 2 },
  ]);
  for (const color of ["#FF0000", [1, 0.5] as [number, number], undefined]) {
    const expected = model.resolveColor(color);
    expect(frame.resolveColor(color)).toBe(expected);
    expect(frame.resolveColor(color)).toBe(expected);
  }
  expect(calls).toBe(3);
});
