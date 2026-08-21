// Golden-screenshot comparison. Each scenario has a committed PNG in
// screenshots/; the test re-renders it and compares pixel by pixel.
//
// On a mismatch the test fails and writes two extra files next to the golden:
//   <name>.new.png   what the renderer draws now
//   <name>.diff.png  the golden, faded, with every changed pixel in red
// Open them side by side, and if the change is intended either copy the
// .new.png over the golden or re-run with UPDATE_SCREENSHOTS=1.

import {
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  type Canvas,
  createCanvas,
  Image,
  type ImageData as SkiaImageData,
} from "@napi-rs/canvas";

const screenshotsDir = fileURLToPath(new URL("screenshots/", import.meta.url));

function pixelsOf(canvas: Canvas): SkiaImageData {
  return canvas
    .getContext("2d")
    .getImageData(0, 0, canvas.width, canvas.height);
}

async function decodePng(buffer: Buffer): Promise<SkiaImageData> {
  const image = new Image();
  image.src = buffer;
  // src assignment alone is not enough: the bitmap is not ready for
  // drawImage until decode() resolves
  await image.decode();
  const canvas = createCanvas(image.naturalWidth, image.naturalHeight);
  const context = canvas.getContext("2d");
  context.drawImage(image, 0, 0);
  return context.getImageData(0, 0, canvas.width, canvas.height);
}

// The golden, faded towards white, with every differing pixel painted red
function writeDiffImage(
  golden: SkiaImageData,
  actual: SkiaImageData,
  path: string,
): number {
  const width = golden.width;
  const height = golden.height;
  const canvas = createCanvas(width, height);
  const context = canvas.getContext("2d");
  const diff = context.createImageData(width, height);
  let changed = 0;
  for (let i = 0; i < golden.data.length; i += 4) {
    const equal =
      golden.data[i] === actual.data[i] &&
      golden.data[i + 1] === actual.data[i + 1] &&
      golden.data[i + 2] === actual.data[i + 2] &&
      golden.data[i + 3] === actual.data[i + 3];
    if (equal) {
      // Never-painted (transparent) regions read as white, not black
      const alpha = golden.data[i + 3] / 255;
      diff.data[i] = 255 - (255 - golden.data[i]) * 0.15 * alpha;
      diff.data[i + 1] = 255 - (255 - golden.data[i + 1]) * 0.15 * alpha;
      diff.data[i + 2] = 255 - (255 - golden.data[i + 2]) * 0.15 * alpha;
    } else {
      changed += 1;
      diff.data[i] = 255;
      diff.data[i + 1] = 0;
      diff.data[i + 2] = 0;
    }
    diff.data[i + 3] = 255;
  }
  context.putImageData(diff, 0, 0);
  writeFileSync(path, canvas.encodeSync("png"));
  return changed;
}

export async function expectScreenshot(
  canvas: Canvas,
  name: string,
): Promise<void> {
  mkdirSync(screenshotsDir, { recursive: true });
  const goldenPath = join(screenshotsDir, `${name}.png`);
  const newPath = join(screenshotsDir, `${name}.new.png`);
  const diffPath = join(screenshotsDir, `${name}.diff.png`);
  const actualPng = canvas.encodeSync("png");

  if (process.env.UPDATE_SCREENSHOTS) {
    writeFileSync(goldenPath, actualPng);
    rmSync(newPath, { force: true });
    rmSync(diffPath, { force: true });
    return;
  }

  if (!existsSync(goldenPath)) {
    // Locally a missing golden is created on first run; in the CI that would
    // silently pass, so it must fail instead
    if (process.env.CI) {
      throw new Error(
        `Screenshot "${name}" has no committed golden (${goldenPath}). ` +
          `Run the tests locally and commit the generated PNG.`,
      );
    }
    writeFileSync(goldenPath, actualPng);
    console.info(`[screenshot] new golden written, review it: ${goldenPath}`);
    return;
  }

  const goldenPng = readFileSync(goldenPath);
  if (goldenPng.equals(actualPng)) {
    rmSync(newPath, { force: true });
    rmSync(diffPath, { force: true });
    return;
  }

  const golden = await decodePng(goldenPng);
  const actual = pixelsOf(canvas);
  if (golden.width !== actual.width || golden.height !== actual.height) {
    writeFileSync(newPath, actualPng);
    throw new Error(
      `Screenshot "${name}" changed size: ` +
        `${golden.width}x${golden.height} -> ${actual.width}x${actual.height}\n` +
        `  golden: ${goldenPath}\n  actual: ${newPath}`,
    );
  }

  // Same bytes are the fast path above; the PNG encoder could change across
  // library versions, so equality is decided on pixels
  const changedPixels = writeDiffImage(golden, actual, diffPath);
  if (changedPixels === 0) {
    rmSync(diffPath, { force: true });
    return;
  }
  writeFileSync(newPath, actualPng);
  const total = golden.width * golden.height;
  throw new Error(
    `Screenshot "${name}" changed: ${changedPixels} of ${total} pixels differ.\n` +
      `  golden: ${goldenPath}\n` +
      `  actual: ${newPath}\n` +
      `  diff:   ${diffPath}\n` +
      `If the change is intended, re-run with UPDATE_SCREENSHOTS=1 ` +
      `or copy the .new.png over the golden.`,
  );
}
