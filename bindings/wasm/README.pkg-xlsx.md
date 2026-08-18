# IronCalc Web bindings with XLSX

This package is the IronCalc engine plus XLSX import and export. If you only need the calc engine, use [`@ironcalc/wasm`](https://www.npmjs.com/package/@ironcalc/wasm) instead — that package stays smaller.

This addresses the bundle size concern from [ironcalc/IronCalc#379](https://github.com/ironcalc/IronCalc/issues/379) and makes XLSX conversion available in the browser ([#1113](https://github.com/ironcalc/IronCalc/issues/1113)).

## Usage

In your project

```
npm install @ironcalc/wasm-xlsx
```

And then in your TypeScript

```TypeScript
import init, { Model } from "@ironcalc/wasm-xlsx";

await init();

async function convert() {
    const xlsxBytes = new Uint8Array(await file.arrayBuffer());
    const model = Model.fromXlsx(xlsxBytes, "Workbook1", "en", "UTC", "en");

    model.setUserInput(0, 1, 1, "23");
    model.setUserInput(0, 1, 2, "=A1*3+1");

    const out = model.toXlsx();
}

convert();
```
