# IronCalc Web bindings

This crate is used to build the web bindings for IronCalc.

## Usage

In your project

```
npm install @ironcalc/wasm
```

And then in your TypeScript

The core engine and the XLSX helpers are separate WebAssembly modules. Import
the engine from the package root, and the helpers from `@ironcalc/wasm/xlsx`
only if you need them, so XLSX support is not added to your bundle unless used.

```TypeScript
import init, { Model } from "@ironcalc/wasm";
import initXLSX, { toXLSXBytes, fromXLSXBytes } from "@ironcalc/wasm/xlsx";

await init();
await initXLSX();

// Model(name, locale, timezone, languageId)
const model = new Model("Workbook1", "en", "UTC", "en");
model.setUserInput(0, 1, 1, "23");
model.setUserInput(0, 1, 2, "=A1*3+1");
console.log(model.getFormattedCellValue(0, 1, 2)); // "70"

// Export the workbook to XLSX bytes, then read them back.
const xlsxBytes = toXLSXBytes(model.toBytes(), "en");
const modelBytes = fromXLSXBytes(xlsxBytes, "Workbook1", "en", "UTC", "en");
const roundTripped = Model.fromBytes(modelBytes, "en");
```

