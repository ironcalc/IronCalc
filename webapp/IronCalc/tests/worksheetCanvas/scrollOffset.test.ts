// Rendering at a scroll offset that is not on a cell boundary: the first row
// and column of the scrolling pane are partly out of view, under the headers
// or the frozen panes. The assertions probe single pixels of fills and
// headers (no text, so they hold on every platform) and the styles of the
// HTML column headers and overlays.

import type { Canvas } from "@napi-rs/canvas";
import { expect, test } from "vitest";
import {
  headerColumnWidth,
  headerRowHeight,
} from "../../src/components/WorksheetCanvas/constants";
import type WorksheetCanvas from "../../src/components/WorksheetCanvas/worksheetCanvas";
import { newModel, renderWorksheet } from "./harness";

const COLUMN_WIDTH = 90;
const ROW_HEIGHT = 25;
const FROZEN_SEPARATOR = 3;
const DEVICE_PIXEL_RATIO = 2;

const RED = "#FF0000";
const BLUE = "#0000FF";

function cell(row: number, column: number, width = 1, height = 1) {
  return { sheet: 0, row, column, width, height };
}

// The color of the canvas at CSS pixel (x, y) as "#rrggbb", or null where
// nothing was painted
function pixel(canvas: Canvas, x: number, y: number): string | null {
  const [r, g, b, a] = canvas
    .getContext("2d")
    .getImageData(x * DEVICE_PIXEL_RATIO, y * DEVICE_PIXEL_RATIO, 1, 1).data;
  if (a === 0) {
    return null;
  }
  const hex = (v: number) => v.toString(16).padStart(2, "0").toUpperCase();
  return `#${hex(r)}${hex(g)}${hex(b)}`;
}

async function scrolledWorksheet(
  x: number,
  y: number,
  setup: (model: Awaited<ReturnType<typeof newModel>>) => void = () => {},
): Promise<{ canvas: Canvas; worksheet: WorksheetCanvas }> {
  const model = await newModel();
  model.updateRangeStyle(cell(1, 1), "fill.color", RED);
  model.updateRangeStyle(cell(2, 1), "fill.color", BLUE);
  model.updateRangeStyle(cell(1, 2), "fill.color", BLUE);
  setup(model);
  const { canvas, worksheet } = await renderWorksheet(model);
  worksheet.setScroll(x, y);
  worksheet.renderSheet();
  return { canvas, worksheet };
}

test("cells scrolled 10px up and 10px left are painted from the offset", async () => {
  const { canvas, worksheet } = await scrolledWorksheet(10, 10);
  const x0 = headerColumnWidth;
  const y0 = headerRowHeight;
  // The lines between the headers and the cells lie over the partly hidden
  // A1
  const line = worksheet.theme.headerBorderColor.toUpperCase();
  expect(pixel(canvas, x0 + 5, y0)).toBe(line);
  expect(pixel(canvas, x0 + 200, y0)).toBe(line);
  expect(pixel(canvas, x0 - 1, y0 + 5)).toBe(line);
  expect(pixel(canvas, x0 - 1, y0 + 150)).toBe(line);
  // and they edge the corner square (the select-all corner); the vertical
  // one inside the row headers, where the HTML column headers cannot cover
  // it
  expect(pixel(canvas, 15, y0)).toBe(line);
  expect(pixel(canvas, x0 - 1, 15)).toBe(line);
  expect(pixel(canvas, 15, 15)).toBe(
    worksheet.theme.headerCornerBackground.toUpperCase(),
  );
  // A1 (red) now ends 15px below the headers, 80px right of them
  expect(pixel(canvas, x0 + 5, y0 + 5)).toBe(RED);
  expect(pixel(canvas, x0 + 5, y0 + 13)).toBe(RED);
  expect(pixel(canvas, x0 + 78, y0 + 5)).toBe(RED);
  // then A2 (blue) and B1 (blue), past their grid lines
  expect(pixel(canvas, x0 + 5, y0 + ROW_HEIGHT - 10 + 3)).toBe(BLUE);
  expect(pixel(canvas, x0 + COLUMN_WIDTH - 10 + 3, y0 + 5)).toBe(BLUE);
  // Nothing is painted over the column headers' band of the canvas
  expect(pixel(canvas, x0 + 5, y0 - 5)).toBe(null);
  expect(pixel(canvas, x0 + 5, 1)).toBe(null);
});

test("row headers follow the offset and stay out of the corner", async () => {
  const { canvas, worksheet } = await scrolledWorksheet(0, 10);
  const { headerBackground, headerCornerBackground, headerSelectedBackground } =
    worksheet.theme;
  // Row 1's header (highlighted: A1 is selected) ends 15px below the corner,
  // row 2's starts after it
  expect(pixel(canvas, 10, headerRowHeight + 5)).toBe(
    headerSelectedBackground.toUpperCase(),
  );
  expect(pixel(canvas, 10, headerRowHeight + ROW_HEIGHT - 10 + 5)).toBe(
    headerBackground.toUpperCase(),
  );
  // The corner is untouched by the partly hidden row 1
  expect(pixel(canvas, 10, headerRowHeight - 5)).toBe(
    headerCornerBackground.toUpperCase(),
  );
  // A row header's resize handle sits on the row's bottom edge
  const handles = worksheet.canvas.parentElement?.querySelectorAll(
    ".row-resize-handle",
  ) as unknown as HTMLDivElement[];
  expect(handles[0].style.top).toBe(`${headerRowHeight + 15.5 - 1}px`);
});

test("column headers shift with the offset inside a clipped wrapper", async () => {
  const { worksheet } = await scrolledWorksheet(10, 0);
  const wrapper = worksheet.columnHeaders.querySelectorAll(
    ".column-headers-scroll",
  ) as unknown as HTMLDivElement[];
  expect(wrapper).toHaveLength(1);
  expect(wrapper[0].style.width).toBe(`${430 - headerColumnWidth}px`);
  const headers = wrapper[0].querySelectorAll(
    ".column-header",
  ) as unknown as HTMLDivElement[];
  expect(headers[0].textContent).toBe("A");
  expect(headers[0].style.marginLeft).toBe("-10px");
  expect(headers[1].textContent).toBe("B");
  expect(headers[1].style.marginLeft).toBeUndefined();
  // The resize handles are positioned from the headers' container
  const handles = worksheet.columnHeaders.querySelectorAll(
    ".column-resize-handle",
  ) as unknown as HTMLDivElement[];
  expect(handles[0].style.left).toBe(`${COLUMN_WIDTH - 10 - 1}px`);
  expect(handles[1].style.left).toBe(`${2 * COLUMN_WIDTH - 10 - 1}px`);
  expect(worksheet.columnHeaders.style.width).toBe(
    `${430 - headerColumnWidth}px`,
  );
});

test("with frozen panes the first scrolling cells hide under them", async () => {
  // Row 1 and column A frozen; B2 is the first scrolling cell, blue
  const { canvas, worksheet } = await scrolledWorksheet(10, 10, (model) => {
    model.setFrozenRowsCount(0, 1);
    model.setFrozenColumnsCount(0, 1);
    model.updateRangeStyle(cell(2, 2), "fill.color", BLUE);
  });
  const paneX = headerColumnWidth + COLUMN_WIDTH + FROZEN_SEPARATOR;
  const paneY = headerRowHeight + ROW_HEIGHT + FROZEN_SEPARATOR;
  // The frozen cells are where they always are
  expect(pixel(canvas, headerColumnWidth + 5, headerRowHeight + 5)).toBe(RED);
  expect(pixel(canvas, headerColumnWidth + 5, headerRowHeight + 22)).toBe(RED);
  // The separators are intact
  expect(pixel(canvas, paneX + 5, paneY - 2)).toBe(
    worksheet.theme.gridSeparatorColor.toUpperCase(),
  );
  // B2 shows its bottom-right 15x80 px in the scrolling pane
  expect(pixel(canvas, paneX + 5, paneY + 5)).toBe(BLUE);
  expect(pixel(canvas, paneX + 5, paneY + 13)).toBe(BLUE);
  expect(pixel(canvas, paneX + 78, paneY + 5)).toBe(BLUE);
  // The frozen rows pane shows B1 (blue) shifted left too
  expect(pixel(canvas, paneX + 5, headerRowHeight + 5)).toBe(BLUE);
  // The first scrolling column header is shifted and comes after the
  // frozen one and the separator
  const wrapper = worksheet.columnHeaders.querySelectorAll(
    ".column-headers-scroll",
  ) as unknown as HTMLDivElement[];
  const headers = wrapper[0].querySelectorAll(
    ".column-header",
  ) as unknown as HTMLDivElement[];
  expect(headers[0].textContent).toBe("B");
  expect(headers[0].style.marginLeft).toBe("-10px");
  expect(wrapper[0].style.width).toBe(
    `${430 - headerColumnWidth - COLUMN_WIDTH - FROZEN_SEPARATOR}px`,
  );
});

test("overlays of partly hidden cells are clipped to the pane", async () => {
  const { worksheet } = await scrolledWorksheet(10, 10, (model) => {
    model.setSelectedCell(1, 1);
  });
  // The outline box starts 1px above and left of the cell, which is 10px
  // under each header: clip 11px on the top and the left
  expect(worksheet.cellOutline.style.clip).toBe("rect(11px auto auto 11px)");
  expect(worksheet.cellOutline.style.visibility).toBe("visible");
  // A fully visible cell is not clipped
  worksheet.model.setSelectedCell(3, 3);
  worksheet.renderSheet();
  expect(worksheet.cellOutline.style.clip).toBe("");
  // An area starting in the hidden strip clips the area outline too
  worksheet.model.setSelectedRange(1, 1, 3, 3);
  worksheet.renderSheet();
  expect(worksheet.areaOutline.style.clip).toBe("rect(10px auto auto 10px)");
});

test("a cell scrolled entirely out of view hides its outline", async () => {
  const { worksheet } = await scrolledWorksheet(
    0,
    3 * ROW_HEIGHT + 10,
    (model) => {
      model.setSelectedCell(2, 1);
    },
  );
  expect(worksheet.cellOutline.style.visibility).toBe("hidden");
  expect(worksheet.cellOutlineHandle.style.visibility).toBe("hidden");
});
