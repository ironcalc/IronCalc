# IronCalc Web bindings

This package contains web bindings for IronCalc. It exposes the engine and helper functions to import or export workbooks as XLSX or IronCalc (icalc) byte arrays. The built-in XLSX support focuses on core spreadsheet features like cell values, formulas, and styling.

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

## Testing

Right now this is a manual process and only carries out a smoke test:

1. Build the package
2. Run `python -m http.server`
3. In your browser open <http://0.0.0.0:8000/test.html>

## Publishing

Follow the commands:

```bash
wasm-pack login
make
cd pkg
npm publish --access=public
```