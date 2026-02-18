# IronCalc Web bindings

This crate is used to build the web bindings for IronCalc.

## Usage

In your project

```
npm install @ironcalc/wasm
```

And then in your TypeScript

```TypeScript
import init, { Model } from "@ironcalc/wasm";
import initXLSX, { toXLSXBytes, fromXLSXBytes } from "@ironcalc/wasm/xlsx";


await init();
await initXLSX();

function compute() {
    const model = new Model('en', 'UTC');
    
    model.setUserInput(0, 1, 1, "23");
    model.setUserInput(0, 1, 2, "=A1*3+1");
    
    const result = model.getFormattedCellValue(0, 1, 2);
    
    console.log("Result: ", result);
}

compute();

// create a new workbook and export as XLSX bytes
const model = new Model('Workbook1', 'en', 'UTC');
model.setUserInput(0, 1, 1, '42');
const xlsxBytes = toXLSXBytes(model.toBytes());

// load from those bytes
const roundTrippedBytes = fromXLSXBytes(xlsxBytes, 'Workbook1', 'en', 'UTC');
const roundTripped = Model.fromBytes(roundTrippedBytes);

```

