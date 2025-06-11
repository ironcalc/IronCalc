# IronCalc Web bindings

This package contains web bindings for IronCalc. It exposes the engine and helper functions to import or export workbooks as XLSX or IronCalc (icalc) byte arrays, but it does not bundle a full XLSX reader.


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
