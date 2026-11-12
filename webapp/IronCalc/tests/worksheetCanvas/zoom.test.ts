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

// Every frame paints every pixel (each cell gets an opaque background), so
// nothing of a previous frame can survive a repaint. The clear at the start
// of a frame is in sheet pixels; in backing pixels it would cover only part
// of the bitmap when the backing scale is below 1.
test("a repaint covers the whole canvas when the backing scale is below 1", async () => {
  const model = await newModel();
  // 50% on a 1x screen: half a backing pixel per sheet pixel
  const { canvas, worksheet } = await renderWorksheet(model, {
    zoom: 0.5,
    devicePixelRatio: 1,
  });
  expect(worksheet.backingScale()).toBe(0.5);
  // Paint the whole bitmap red behind the renderer's back, as stale pixels
  // of a previous frame would be
  const context = canvas.getContext("2d");
  context.save();
  context.resetTransform();
  context.fillStyle = "#ff0000";
  context.fillRect(0, 0, canvas.width, canvas.height);
  context.restore();
  // A repaint on the same canvas (a scroll, a drag) must clear all of it,
  // including the bottom-right corner, not just the top-left quarter
  worksheet.renderSheet();
  const corner = context.getImageData(
    canvas.width - 1,
    canvas.height - 1,
    1,
    1,
  ).data;
  expect([corner[0], corner[1], corner[2]]).not.toEqual([255, 0, 0]);
  const middle = context.getImageData(
    canvas.width / 2 + 2,
    canvas.height / 2 + 2,
    1,
    1,
  ).data;
  expect([middle[0], middle[1], middle[2]]).not.toEqual([255, 0, 0]);
});
