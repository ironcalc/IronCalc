# WorksheetCanvas visual regression tests

Each test in `renderSheet.test.ts` builds a workbook, renders it through the
real `WorksheetCanvas` onto a real canvas (Skia via `@napi-rs/canvas` — no
browser involved), and compares the result pixel by pixel with the golden PNG
committed in `screenshots/`.

**Open the PNGs in `screenshots/` to see exactly what each test protects.**

## When a test fails

Two files appear next to the golden:

- `<name>.new.png` — what the renderer draws now
- `<name>.diff.png` — the golden, faded, with every changed pixel in red

Open them side by side. If the change is a bug, fix it. If it is intended:

```sh
UPDATE_SCREENSHOTS=1 npx vitest run tests/worksheetCanvas
```

(or copy the `.new.png` over the golden). Then review the new goldens in the
PR diff like any other change. The `.new.png`/`.diff.png` files are
git-ignored and removed automatically once the test passes again.

These tests are part of `npm run test`, so they run in `make tests` /
`make test-js` and therefore in the CI. In the CI (`$CI` set) a missing
golden fails the test instead of being silently created — generate goldens
locally and commit them.

## How it works

- `fakeDom.ts` — a minimal DOM stub (the renderer's constructor touches DOM
  elements incidentally; only the canvas is real). If `WorksheetCanvas`
  starts using a DOM API the stub lacks, the tests fail loudly: extend the
  stub.
- `harness.ts` — `newModel()` (wasm engine) and `renderToCanvas(model)`.
  Screenshots render at 2x, like a retina display.
- `screenshot.ts` — golden comparison and diff-image generation.

## Limitations

- The screenshots are cropped to the cell area. Column headers are DOM divs
  (they never appear on the canvas), and the row headers are cropped away
  with them.
- Selection outlines, the editor, and tooltips and any other HTML DOM overlays and are not
  captured either.
