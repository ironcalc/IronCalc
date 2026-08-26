// Visual regression tests for WorksheetCanvas. Each scenario builds a model,
// renders it through the real renderer onto a real canvas, and compares the
// PNG with the committed golden in screenshots/ — open those files to see
// exactly what each test protects.
//
// On failure, <name>.new.png and <name>.diff.png land next to the golden.
// To accept an intended change: UPDATE_SCREENSHOTS=1 npx vitest run

import { BorderStyle, BorderType, type Model } from "@ironcalc/wasm";
import { test } from "vitest";
import { newModel, renderToCanvas } from "./harness";
import { expectScreenshot } from "./screenshot";

function cell(row: number, column: number, width = 1, height = 1) {
  return { sheet: 0, row, column, width, height };
}

// The screenshots are cropped to the cell area (no row/column headers), so
// every scenario labels its top-left visible cell with a formula that
// resolves to the cell's own reference ("A1", "E1", ...): the screenshot
// tells you where in the sheet you are, and it exercises the formula engine.
function labelTopLeftCell(model: Model, row = 1, column = 1): void {
  model.setUserInput(0, row, column, "=ADDRESS(ROW(),COLUMN(),4)");
}

test("text spills right and is blocked by a non-empty neighbour", async () => {
  const model = await newModel();
  labelTopLeftCell(model);
  // B2 spills over the empty C2 and D2
  model.setUserInput(0, 2, 2, "A very long text that spills right");
  // B4 has the same text but C4 is occupied: it must clip at the cell edge
  model.setUserInput(0, 4, 2, "A very long text that spills right");
  model.setUserInput(0, 4, 3, "STOP");
  // B6 is right-aligned, so it spills into the empty A6 to its left
  model.setUserInput(0, 6, 2, "Spills to the left");
  model.updateRangeStyle(cell(6, 2), "alignment.horizontal", "right");

  await expectScreenshot(
    await renderToCanvas(model),
    "spill-right-and-blocked",
  );
});

test("text spills in from a cell scrolled out of the viewport", async () => {
  const model = await newModel();
  labelTopLeftCell(model, 1, 5);
  // Long text in B2; the viewport starts at column E, so column B is off
  // screen to the left but its text must still spill into view
  model.setUserInput(
    0,
    2,
    2,
    "A very long text that starts off screen and spills into the viewport",
  );
  // Same, but here D4 is non-empty: the spill must stop before the viewport
  model.setUserInput(0, 4, 2, "A very long text that starts off screen");
  model.setUserInput(0, 4, 4, "STOP");
  model.setTopLeftVisibleCell(1, 5);

  await expectScreenshot(
    await renderToCanvas(model),
    "spill-from-off-viewport",
  );
});

test("wrapped text, font sizes and vertical alignment", async () => {
  const model = await newModel();
  labelTopLeftCell(model);
  model.setRowsHeight(0, 2, 2, 80);
  // B2 wraps inside the cell
  model.setUserInput(0, 2, 2, "Wrapped text with quite a few words in it");
  model.updateRangeStyle(cell(2, 2), "alignment.wrap_text", "true");
  // C2 does not wrap: with the taller row it sits at the bottom by default
  model.setUserInput(0, 2, 3, "bottom");
  // D2 is centered vertically
  model.setUserInput(0, 2, 4, "center");
  model.updateRangeStyle(cell(2, 4), "alignment.vertical", "center");
  // B4 has a bigger font than the row: larger metrics, spills further
  model.setUserInput(0, 4, 2, "Large font text");
  model.updateRangeStyle(cell(4, 2), "font.size", "24");

  await expectScreenshot(await renderToCanvas(model), "wrap-and-font-sizes");
});

test("merged cells with frozen rows and columns", async () => {
  const model = await newModel();
  labelTopLeftCell(model);
  model.setFrozenRowsCount(0, 2);
  model.setFrozenColumnsCount(0, 1);
  // A merged range in the scrolling pane, with a fill and centered text
  model.setUserInput(0, 4, 3, "Merged C4:E5");
  model.updateRangeStyle(cell(4, 3, 3, 2), "fill.color", "#FFF2CC");
  model.updateRangeStyle(cell(4, 3, 3, 2), "alignment.horizontal", "center");
  model.mergeCells(cell(4, 3, 3, 2));
  // A vertical merge that straddles the frozen-rows boundary (rows 2-4)
  model.setUserInput(0, 2, 2, "Straddles");
  model.mergeCells(cell(2, 2, 1, 3));
  // Text next to the merged range must not spill into it
  model.setUserInput(0, 5, 2, "Text that would spill into the merge");

  await expectScreenshot(await renderToCanvas(model), "merged-and-frozen");
});

test("merged range renders when its anchor is scrolled out of view", async () => {
  const model = await newModel();
  labelTopLeftCell(model, 4, 1);
  // Anchor at B2, merged down to row 6; scrolling to row 4 hides the anchor
  // but the visible part of the range must still paint its fill and text
  model.setUserInput(0, 2, 2, "Tall merge");
  model.updateRangeStyle(cell(2, 2, 2, 5), "fill.color", "#D9EAD3");
  model.mergeCells(cell(2, 2, 2, 5));
  model.setTopLeftVisibleCell(4, 1);

  await expectScreenshot(await renderToCanvas(model), "merged-anchor-off-view");
});

test("right-aligned text spills in from a cell right of the viewport", async () => {
  const model = await newModel();
  labelTopLeftCell(model);
  // Columns A-D are visible; F2 is off screen to the right but its
  // right-aligned text must spill left into the viewport
  model.setUserInput(
    0,
    2,
    6,
    "A long right-aligned text that enters the viewport from the right",
  );
  model.updateRangeStyle(cell(2, 6), "alignment.horizontal", "right");
  // Same, but E4 is non-empty: the spill must stop before the viewport
  model.setUserInput(
    0,
    4,
    6,
    "A long right-aligned text that is blocked by E4",
  );
  model.updateRangeStyle(cell(4, 6), "alignment.horizontal", "right");
  model.setUserInput(0, 4, 5, "STOP");

  await expectScreenshot(await renderToCanvas(model), "spill-from-the-right");
});

test("bold text, colors and borders", async () => {
  const model = await newModel();
  labelTopLeftCell(model);
  // B2: bold text at a larger font size
  model.setUserInput(0, 2, 2, "Bold 18px");
  model.updateRangeStyle(cell(2, 2), "font.b", "true");
  model.updateRangeStyle(cell(2, 2), "font.size", "18");
  // B4: colored text on a colored background
  model.setUserInput(0, 4, 2, "Colored");
  model.updateRangeStyle(cell(4, 2), "font.color", "#C0392B");
  model.updateRangeStyle(cell(4, 2), "fill.color", "#FDEBD0");
  // D2: a double outer border on a single cell
  model.setAreaWithBorder(cell(2, 4), {
    item: { style: BorderStyle.Double, color: "#000000" },
    type: BorderType.Outer,
  });
  // C6:D7: medium borders on every edge, inner ones included
  model.setAreaWithBorder(cell(6, 3, 2, 2), {
    item: { style: BorderStyle.Medium, color: "#1F618D" },
    type: BorderType.All,
  });

  await expectScreenshot(await renderToCanvas(model), "bold-colors-borders");
});

test("merging cells with different fills paints only the anchor fill", async () => {
  const model = await newModel();
  labelTopLeftCell(model);
  // B2 is yellow and C2 red; merging B2:C2 with the yellow B2 as anchor
  // must paint the whole merged rectangle yellow — no red from the covered
  // cell may bleed through at the edges
  model.setUserInput(0, 2, 2, "Merged B2:C2");
  model.updateRangeStyle(cell(2, 2), "fill.color", "#FFFF00");
  model.updateRangeStyle(cell(2, 3), "fill.color", "#FF0000");
  model.mergeCells(cell(2, 2, 2, 1));
  // The same vertically: B4 yellow over a red B5
  model.setUserInput(0, 4, 2, "Merged B4:B5");
  model.updateRangeStyle(cell(4, 2), "fill.color", "#FFFF00");
  model.updateRangeStyle(cell(5, 2), "fill.color", "#FF0000");
  model.mergeCells(cell(4, 2, 1, 2));

  await expectScreenshot(await renderToCanvas(model), "merge-covers-fills");
});

test("wrapped text inside a merged range", async () => {
  const model = await newModel();
  labelTopLeftCell(model);
  // B2:C4 merged with wrapping: the text must wrap at the merged width
  // (not the cell width) and never spill outside the merged rectangle
  model.setUserInput(
    0,
    2,
    2,
    "A wrapped paragraph inside a merged range stays within its rectangle",
  );
  model.updateRangeStyle(cell(2, 2, 2, 3), "alignment.wrap_text", "true");
  model.updateRangeStyle(cell(2, 2, 2, 3), "fill.color", "#E8EAF6");
  model.mergeCells(cell(2, 2, 2, 3));

  await expectScreenshot(await renderToCanvas(model), "wrap-inside-merge");
});
