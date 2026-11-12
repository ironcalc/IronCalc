// Unit tests for the zoom of the WorksheetCanvas. The sheet container is
// scaled by the zoom with CSS: the canvas keeps its size in sheet pixels and
// the zoom only raises its backing resolution and converts pointer positions.

import { expect, test } from "vitest";
import { newModel, renderWorksheet } from "./harness";

// The harness renders at a device pixel ratio of 2
const DEVICE_PIXEL_RATIO = 2;

test("the canvas keeps its sheet size and doubles its backing resolution at 200%", async () => {
  const model = await newModel();
  const { worksheet } = await renderWorksheet(model, { zoom: 2 });
  expect(worksheet.width).toBe(430);
  expect(worksheet.height).toBe(230);
  expect(worksheet.backingScale()).toBe(2 * DEVICE_PIXEL_RATIO);
  expect(worksheet.canvas.width).toBe(430 * 2 * DEVICE_PIXEL_RATIO);
  expect(worksheet.canvas.height).toBe(230 * 2 * DEVICE_PIXEL_RATIO);
  expect(worksheet.canvas.style.width).toBe("430px");
  expect(worksheet.canvas.style.height).toBe("230px");
});

test("the backing resolution is the device pixel ratio alone at 100%", async () => {
  const model = await newModel();
  const { worksheet } = await renderWorksheet(model);
  expect(worksheet.zoom).toBe(1);
  expect(worksheet.backingScale()).toBe(DEVICE_PIXEL_RATIO);
  expect(worksheet.canvas.width).toBe(430 * DEVICE_PIXEL_RATIO);
});

test("pointer positions are converted from scaled pixels to sheet pixels", async () => {
  const model = await newModel();
  // The fake canvas sits at the screen origin
  const { worksheet } = await renderWorksheet(model, { zoom: 2 });
  expect(worksheet.eventCoordinates({ clientX: 200, clientY: 100 })).toEqual({
    x: 100,
    y: 50,
  });
  // At 200% the first cell covers 180x50 screen pixels past the headers
  // (30x28 sheet pixels, so 60x56 on screen)
  expect(
    worksheet.getCellByCoordinates(
      worksheet.eventCoordinates({ clientX: 200, clientY: 100 }).x,
      worksheet.eventCoordinates({ clientX: 200, clientY: 100 }).y,
    ),
  ).toEqual({ row: 1, column: 1 });
});
