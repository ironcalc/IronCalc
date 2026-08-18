# IronCalc Web bindings

This crate is used to build the web bindings for IronCalc.

Two packages are produced from the same crate:

* `@ironcalc/wasm` — the calc engine only. This is the default `make` / `wasm-pack` build.
* `@ironcalc/wasm-xlsx` — the same engine plus XLSX import (`Model.fromXlsx`) and export (`model.toXlsx`). Built with `--features xlsx`.

The default package does not contain the xlsx writer and reader, so projects that do not import or export spreadsheets are not charged for that code.

https://www.npmjs.com/package/@ironcalc/wasm?activeTab=readme

## Building

Dependencies:

* Rust
* wasm-pack
* TypeScript
* Python
* binutils (for make)


```bash
make
```

That writes `@ironcalc/wasm` to `pkg/` and `@ironcalc/wasm-xlsx` to `pkg-xlsx/`.

## Testing

Right now this is a manual process and only carries out a smoke test:

1. Build the package
2. Run `python -m http.server`
3. In your browser open <http://0.0.0.0:8000/test.html>

Node tests (including XLSX round-trip on the larger build):

```bash
make tests
```

## Publishing

Follow the commands:

```bash
wasm-pack login
make
cd pkg
npm publish --access=public
cd ../pkg-xlsx
npm publish --access=public
```
