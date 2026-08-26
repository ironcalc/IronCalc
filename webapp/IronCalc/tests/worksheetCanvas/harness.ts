// Renders a model through the real WorksheetCanvas onto a real (Skia-backed)
// canvas in node — no browser involved. The fonts are the Inter files the
// workbook itself ships (fonts/*.woff2), so the screenshots show exactly
// what users see and the output is identical on every machine.

import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { initSync, Model } from "@ironcalc/wasm";
import { type Canvas, createCanvas, GlobalFonts } from "@napi-rs/canvas";
import {
  headerColumnWidth,
  headerRowHeight,
} from "../../src/components/WorksheetCanvas/constants";
import type WorksheetCanvas from "../../src/components/WorksheetCanvas/worksheetCanvas";
import { WorkbookState } from "../../src/components/workbookState";
import { FakeCanvasElement, FakeElement, installDomGlobals } from "./fakeDom";

// Rendered at 2x, like a retina display: crisper screenshots to review
const DEVICE_PIXEL_RATIO = 2;

// The app's own fonts. This must run before anything in the process draws
// with "Inter": a font string resolved before registration is cached against
// the fallback font.
for (const file of [
  "inter-v13-latin-regular.woff2",
  "inter-v13-latin-600.woff2",
]) {
  GlobalFonts.registerFromPath(
    fileURLToPath(new URL(`../../fonts/${file}`, import.meta.url)),
    "Inter",
  );
}

let wasmLoaded = false;

export async function newModel(): Promise<Model> {
  if (!wasmLoaded) {
    const buffer = await readFile("node_modules/@ironcalc/wasm/wasm_bg.wasm");
    initSync({ module: buffer });
    wasmLoaded = true;
  }
  return new Model("workbook", "en", "UTC", "en");
}

export interface RenderOptions {
  // 430x230 fits the 30px/28px headers plus four 100px columns and eight
  // 25px rows
  width?: number;
  height?: number;
}

// Builds the WorksheetCanvas for `model` and renders one frame. Returns the
// (uncropped) canvas and the worksheet itself, whose DOM overlays
// (cellOutline, areaOutline, cellOutlineHandle, ...) are fake elements: tests
// can assert the exact geometry the renderer assigned to their styles.
export async function renderWorksheet(
  model: Model,
  options: RenderOptions = {},
): Promise<{ canvas: Canvas; worksheet: WorksheetCanvas }> {
  installDomGlobals(DEVICE_PIXEL_RATIO);
  // Imported dynamically: the module reads `window` at load time, so the
  // fake DOM globals must be installed first
  const { default: WorksheetCanvas } = await import(
    "../../src/components/WorksheetCanvas/worksheetCanvas"
  );

  const width = options.width ?? 430;
  const height = options.height ?? 230;
  const canvas = createCanvas(
    width * DEVICE_PIXEL_RATIO,
    height * DEVICE_PIXEL_RATIO,
  );

  const root = new FakeElement("div");
  root.className = "ic-root";
  const container = root.appendChild(new FakeElement("div"));
  const canvasElement = container.appendChild(
    new FakeCanvasElement(canvas.getContext("2d")),
  );
  const div = (): HTMLDivElement =>
    container.appendChild(new FakeElement("div")) as unknown as HTMLDivElement;

  const worksheet = new WorksheetCanvas({
    model,
    width,
    height,
    workbookState: new WorkbookState(),
    elements: {
      canvas: canvasElement as unknown as HTMLCanvasElement,
      cellOutline: div(),
      areaOutline: div(),
      cellArrayStructure: div(),
      extendToOutline: div(),
      columnGuide: div(),
      rowGuide: div(),
      columnHeaders: div(),
      editor: div(),
    },
    onColumnWidthChanges: () => {},
    onRowHeightChanges: () => {},
    linkTooltipCell: null,
    refresh: () => {},
  });
  worksheet.renderSheet();
  return { canvas, worksheet };
}

export async function renderToCanvas(
  model: Model,
  options: RenderOptions = {},
): Promise<Canvas> {
  const { canvas } = await renderWorksheet(model, options);
  const width = options.width ?? 430;
  const height = options.height ?? 230;

  // Crop the headers away: the column headers are HTML and never appear on
  // the canvas (so they show as a blank band), and without them the row
  // headers are just noise — the screenshots show the cell area only.
  const cropped = createCanvas(
    (width - headerColumnWidth) * DEVICE_PIXEL_RATIO,
    (height - headerRowHeight) * DEVICE_PIXEL_RATIO,
  );
  cropped
    .getContext("2d")
    .drawImage(
      canvas,
      -headerColumnWidth * DEVICE_PIXEL_RATIO,
      -headerRowHeight * DEVICE_PIXEL_RATIO,
    );
  return cropped;
}
