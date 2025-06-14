# IronCalc Web bindings

This package contains web bindings for IronCalc. Note that it does not contain the xlsx writer and reader, only the engine.


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

To listen to model changes you can subscribe to diff events:

```TypeScript
import init, { Model } from "@ironcalc/wasm";

await init();

const model = new Model("Sheet1", "en", "UTC");

model.onDiffs(() => {
    // React to diff list updates
    redraw();
});

model.setUserInput(0, 1, 1, "=1+1");
```

To listen to cells that change during evaluation (formulas that recalculate):

```TypeScript
import init, { Model } from "@ironcalc/wasm";

await init();

const model = new Model("Sheet1", "en", "UTC");

model.onCellsEvaluated((cellReferences) => {
    // cellReferences is an array of {sheet, row, column} objects
    // that represent cells that were recalculated during evaluation
    cellReferences.forEach(cell => {
        console.log(`Cell ${cell.sheet}:${cell.row}:${cell.column} was evaluated`);
    });
});

// Setting a formula will trigger evaluation
model.setUserInput(0, 1, 1, "=SUM(A2:A5)");
model.setUserInput(0, 2, 1, "10");  // This will trigger re-evaluation of A1
```
