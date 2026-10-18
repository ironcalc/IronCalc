// The text pass only produces an entry for cells that have something to draw
// or a link to hover: on a mostly empty sheet, this is what keeps a repaint
// proportional to the content rather than to the viewport.

import { expect, test } from "vitest";
import { newModel, renderWorksheet } from "./harness";

test("empty cells get no text entry", async () => {
  const model = await newModel();
  model.setUserInput(0, 2, 2, "hello");
  model.setUserInput(0, 3, 3, "=1+1");
  // a filled background is still not text
  model.updateRangeStyle(
    { sheet: 0, row: 5, column: 1, width: 3, height: 1 },
    "fill.fg_color",
    "#FF0000",
  );
  const { worksheet } = await renderWorksheet(model);
  expect(
    worksheet.cells.map(({ row, column }) => `${row}-${column}`).sort(),
  ).toEqual(["2-2", "3-3"]);
});

test("an empty cell with a link keeps its entry so the link can be hovered", async () => {
  const model = await newModel();
  model.setCellLink(
    0,
    4,
    2,
    { type: "External", target: "https://www.ironcalc.com" },
    "",
  );
  const { worksheet } = await renderWorksheet(model);
  expect(model.getFormattedCellValue(0, 4, 2)).toBe("");
  const entry = worksheet.cells.find((c) => c.row === 4 && c.column === 2);
  expect(entry?.link).toMatchObject({ type: "External" });
});
