# IronCalc Web bindings

This package contains web bindings for IronCalc. It exposes the engine and helper functions to import or export workbooks as XLSX or IronCalc (icalc) byte arrays. The built-in XLSX support focuses on core spreadsheet features like cell values, formulas, and styling.


## Usage

In your project

```
npm install @ironcalc/wasm
```

And then in your TypeScript

```TypeScript
import init, { Model } from "@ironcalc/wasm";

await init();

function compute() {
    const model = new Model('en', 'UTC');
    
    model.setUserInput(0, 1, 1, "23");
    model.setUserInput(0, 1, 2, "=A1*3+1");
    
    const result = model.getFormattedCellValue(0, 1, 2);
    
    console.log("Result: ", result);
}

compute();
```

### Importing and exporting bytes

The `Model` class provides helpers to load or save workbooks as raw byte arrays.

```ts
// create a new workbook and export as XLSX bytes
const model = new Model('Workbook1', 'en', 'UTC');
model.setUserInput(0, 1, 1, '42');
const xlsxBytes = model.saveToXlsx();

// load from those bytes
const roundTripped = Model.fromXlsxBytes(xlsxBytes, 'Workbook1', 'en', 'UTC');

// same helpers exist for IronCalc's internal format
const icalcBytes = model.saveToIcalc();
const restored = Model.fromIcalcBytes(icalcBytes);
```

