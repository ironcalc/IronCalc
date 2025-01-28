import test from 'ava'

import { UserModel, Model } from '../index.js';
 
test('User Model smoke test', (t) => {
  const model = new UserModel("Workbook1", "en", "UTC");

  model.setUserInput(0, 1, 1, "=1+1");
  t.is(model.getFormattedCellValue(0, 1, 1), '2');
});


test('Raw API smoke test', (t) => {
  const model = new Model("Workbook1", "en", "UTC");

  model.setUserInput(0, 1, 1, "=1+1");
  model.evaluate();
  t.is(model.getFormattedCellValue(0, 1, 1), '2');
});

