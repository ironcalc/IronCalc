import test from 'ava'

import { Model } from '../index.js';
 
test('sum from native', (t) => {
  const model = new Model("Workbook1", "en", "UTC");

  model.setUserInput(0, 1, 1, "=1+1");
  t.is(model.getFormattedCellValue(0, 1, 1), '2');
});

