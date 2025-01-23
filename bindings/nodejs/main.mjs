import { Model } from './index.js'
 
const model = new Model("Workbook1", "en", "UTC");

model.setUserInput(0, 1, 1, "=1+1");
let t = model.getFormattedCellValue(0, 1, 1);

console.log('From native', t);

let t2 = model.getCellStyle(0, 1, 1);
console.log('From native', t2);