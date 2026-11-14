import { execSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { initSync, Model } from "@ironcalc/wasm";
import { createCanvas } from "@napi-rs/canvas";
import { describe, expect, it } from "vitest";
import { renderWorksheet } from "./harness";

const DEVICE_PIXEL_RATIO = 2;

describe("Complex table PNG export", () => {
  it("exports complex table as a PNG matching app's PNG download capability", async () => {
    const wasmPath = fileURLToPath(
      new URL("../../node_modules/@ironcalc/wasm/wasm_bg.wasm", import.meta.url),
    );
    const wasmBuffer = readFileSync(wasmPath);
    try {
      initSync({ module: wasmBuffer });
    } catch {
      // WASM module may already be initialized
    }

    const bytesPath = fileURLToPath(
      new URL("../../../../pdf/tests/output/complex_table.bytes", import.meta.url),
    );

    if (!existsSync(bytesPath)) {
      const repoRoot = fileURLToPath(new URL("../../../../", import.meta.url));
      execSync(
        "cargo test -p ironcalc_pdf --test test_complex_table_export -- test_complex_table_evaluation_integrity",
        {
          cwd: repoRoot,
          stdio: "inherit",
        },
      );
    }
    expect(existsSync(bytesPath)).toBe(true);

    const modelBytes = readFileSync(bytesPath);
    const model = Model.fromBytes(modelBytes, "en");
    model.evaluate();

    // Render the worksheet with viewport covering the full table (13 columns, 30 rows)
    const width = 1600;
    const height = 900;
    const { canvas, worksheet } = await renderWorksheet(model, {
      width,
      height,
    });

    // Current app capability for PNG export (from Workbook.tsx: onDownloadPNG):
    // Coordinates of selected table cells: row 1..30, columns 1..13
    const rowStart = 1;
    const columnStart = 1;
    const rowEnd = 30;
    const columnEnd = 13;

    let [x, y] = worksheet.getCoordinatesByCell(rowStart, columnStart);
    const [x1, y1] = worksheet.getCoordinatesByCell(rowEnd + 1, columnEnd + 1);

    const capturedWidth = (x1 - x) * DEVICE_PIXEL_RATIO;
    const capturedHeight = (y1 - y) * DEVICE_PIXEL_RATIO;
    x *= DEVICE_PIXEL_RATIO;
    y *= DEVICE_PIXEL_RATIO;

    const capturedCanvas = createCanvas(capturedWidth, capturedHeight);
    const ctx = capturedCanvas.getContext("2d");
    expect(ctx).not.toBeNull();

    ctx!.drawImage(
      canvas,
      x,
      y,
      capturedWidth,
      capturedHeight,
      0,
      0,
      capturedWidth,
      capturedHeight,
    );

    const pngBuffer = capturedCanvas.encodeSync("png");
    expect(pngBuffer.length).toBeGreaterThan(5000);
  });
});
