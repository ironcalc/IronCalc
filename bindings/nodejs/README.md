# IronCalc nodejs bindings

## Installation

This package is published to GitHub Packages. Add the following to your `.npmrc`:

```
@zolidar:registry=https://npm.pkg.github.com
//npm.pkg.github.com/:_authToken=${GITHUB_TOKEN}
```

Then install:

```bash
npm install @zolidar/ironcalc-nodejs
```

## Example usage

```javascript
import { Model } from '@zolidar/ironcalc-nodejs';

const model = new Model("Workbook1", "en", "UTC", "en");

model.setUserInput(0, 1, 1, "=1+1");

const result1 = model.getFormattedCellValue(0, 1, 1);
console.log('Cell value', result1); // "#ERROR"

model.evaluate();

const resultAfterEvaluate = model.getFormattedCellValue(0, 1, 1);
console.log('Cell value', resultAfterEvaluate); // 2

let result2 = model.getCellStyle(0, 1, 1);
console.log('Cell style', result2);
```
