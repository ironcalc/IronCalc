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
npm run test:visual:update
```

(or copy the `.new.png` over the golden; see below for why this goes
through Docker on a Mac). Then review the new goldens in the
PR diff like any other change. The `.new.png`/`.diff.png` files are
git-ignored and removed automatically once the test passes again.

## Running them: Linux x86_64 natively, everything else in Docker

Text anti-aliasing differs slightly between platforms (and between arm64 and
x86_64), so on a Mac every test with text fails by a few hundred pixels at
the glyph edges even when nothing changed. The goldens are generated on
`linux/amd64`, which is what the CI (`ubuntu-latest`) runs. Two commands in
`webapp/IronCalc` take care of this:

```sh
npm run test:visual          # compare with the goldens
npm run test:visual:update   # regenerate the goldens
```

On Linux x86_64 they run `vitest` directly. On any other platform they run
the same thing inside a `linux/amd64` `node:24` container (Docker must be
running; on Apple Silicon the container runs through Rosetta). The repo is
bind-mounted, so the goldens land straight in `screenshots/`, and the wasm
package built by `make -C bindings/wasm` is picked up through the
`node_modules/@ironcalc/wasm` symlink. The Linux `node_modules` live in a
Docker volume (`ironcalc-visual-tests-node_modules`): the first run installs
them (about 40s), later runs are fast. `npm run test` runs the rest of the
suite natively and then `test:visual`, so `make test-js` works everywhere.

Never regenerate the goldens natively on a Mac
(`UPDATE_SCREENSHOTS=1 npx vitest run tests/worksheetCanvas`): they would
fail in the CI.

Running the tests natively is still useful to see *what* changed: open the
`.diff.png` files. Anti-aliasing noise shows as thin red outlines around
glyphs; a real change (a moved or recolored text, a different fill or
border) shows as solid red.

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
