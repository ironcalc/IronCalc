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

## Run them in Docker (the only way to match the CI)

Text anti-aliasing differs slightly between platforms (and between arm64 and
x86_64), so on a Mac every test with text fails by a few hundred pixels at
the glyph edges even when nothing changed. The goldens are generated on
`linux/amd64`, which is what the CI (`ubuntu-latest`) runs. To get the same
pixels locally, run the tests in a `linux/amd64` container from the repo
root (Docker Desktop on Apple Silicon runs it through Rosetta):

```sh
docker run --rm --platform linux/amd64 \
  -v "$PWD":/work -v /work/webapp/IronCalc/node_modules \
  -w /work/webapp/IronCalc -e CI=1 node:24 \
  sh -c 'npm ci && npx vitest run tests/worksheetCanvas'
```

The second `-v` hides the host `node_modules` behind an anonymous volume so
the Linux build of `@napi-rs/canvas` is installed inside the container (the
`npm ci` takes about 40s) without touching the Mac one. The wasm package must
have been built first (`make -C bindings/wasm`): `node_modules/@ironcalc/wasm`
is a symlink into `bindings/wasm/pkg`, which the bind mount brings along.

When a failure is intended, regenerate the goldens the same way, with
`UPDATE_SCREENSHOTS=1` instead of `CI=1`:

```sh
docker run --rm --platform linux/amd64 \
  -v "$PWD":/work -v /work/webapp/IronCalc/node_modules \
  -w /work/webapp/IronCalc -e UPDATE_SCREENSHOTS=1 node:24 \
  sh -c 'npm ci && npx vitest run tests/worksheetCanvas'
```

The PNGs are written straight to `screenshots/` on the host. Never regenerate
the goldens natively on a Mac: they would then fail in the CI.

Running the tests natively (`npx vitest run tests/worksheetCanvas`) is still
useful to check *what* changed: open the `.diff.png` files. Anti-aliasing
noise shows as thin red outlines around glyphs; a real change (a moved or
recolored text, a different fill or border) shows as solid red.

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
