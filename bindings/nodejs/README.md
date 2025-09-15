# IronCalc nodejs bindingds


Example usage:

```javascript
import { Model } from '@ironcalc/nodejs';
 
const model = new Model("Workbook1", "en", "UTC");

model.setUserInput(0, 1, 1, "=1+1");

const result1 = model.getFormattedCellValue(0, 1, 1);
console.log('Cell value', result1); // "#ERROR"

model.evaluate();

const resultAfterEvaluate = model.getFormattedCellValue(0, 1, 1);
console.log('Cell value', resultAfterEvaluate); // 2

let result2 = model.getCellStyle(0, 1, 1);
console.log('Cell style', result2);
```
