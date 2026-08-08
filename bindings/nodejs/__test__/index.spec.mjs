import test from 'ava'

import {
  UserModel,
  Model,
  CellType,
  columnNameFromNumber,
  columnNumberFromName,
  quoteName,
  getAllTimezones,
  getSupportedLocales,
} from '../index.js';

test('User Model smoke test', (t) => {
  const model = new UserModel("Workbook1", "en", "UTC", "en");

  model.setUserInput(0, 1, 1, "=1+1");
  t.is(model.getFormattedCellValue(0, 1, 1), '2');
});


test('Raw API smoke test', (t) => {
  const model = new Model("Workbook1", "en", "UTC", "en");

  model.setUserInput(0, 1, 1, "=1+1");
  model.evaluate();
  t.is(model.getFormattedCellValue(0, 1, 1), '2');
});

test('constructor arguments are optional', (t) => {
  const model = new UserModel("Workbook1");
  model.setUserInput(0, 1, 1, "=2*3");
  t.is(model.getFormattedCellValue(0, 1, 1), '6');
});

test('cell types', (t) => {
  const model = new UserModel("Workbook1");
  model.setUserInput(0, 1, 1, "42");
  model.setUserInput(0, 1, 2, "Hello");
  model.setUserInput(0, 1, 3, "true");
  t.is(model.getCellType(0, 1, 1), CellType.Number);
  t.is(model.getCellType(0, 1, 2), CellType.Text);
  t.is(model.getCellType(0, 1, 3), CellType.LogicalValue);
});

test('raw cell values', (t) => {
  const model = new Model("Workbook1");
  model.updateCellWithNumber(0, 1, 1, 3.5);
  model.updateCellWithText(0, 1, 2, "Hello");
  model.updateCellWithBool(0, 1, 3, true);
  model.updateCellWithFormula(0, 1, 4, "=A1*2");
  model.evaluate();
  t.is(model.getCellValue(0, 1, 1), 3.5);
  t.is(model.getCellValue(0, 1, 2), "Hello");
  t.is(model.getCellValue(0, 1, 3), true);
  t.is(model.getCellValue(0, 1, 4), 7);
  t.is(model.getCellValueByRef("Sheet1!A1"), 3.5);
  t.is(model.getCellFormula(0, 1, 4), "=A1*2");
  t.is(model.getCellValue(0, 2, 1), null);
  t.true(model.isEmptyCell(0, 2, 1));
});

test('styles', (t) => {
  const model = new UserModel("Workbook1");
  model.updateRangeStyle(0, 1, 1, 2, 2, "font.b", "true");
  const style = model.getCellStyle(0, 1, 1);
  t.true(style.font.b);

  model.setAreaWithBorder(0, 1, 1, 2, 2, {
    item: { style: "thin", color: "#FF0000" },
    type: "All",
  });
  t.deepEqual(model.getCellStyle(0, 1, 1).border.top, {
    style: "thin",
    color: "#FF0000",
  });
});

test('named styles', (t) => {
  const model = new UserModel("Workbook1");
  const style = model.getCellStyle(0, 1, 1);
  style.font.i = true;
  model.createNamedStyle("italics", style, null);
  t.true(model.getNamedStyleList().includes("italics"));
  t.true(model.getNamedStyle("italics").font.i);
});

test('defined names', (t) => {
  const model = new UserModel("Workbook1");
  model.newDefinedName("myname", null, "Sheet1!$A$1");
  model.newDefinedName("localname", 0, "Sheet1!$B$2");
  const names = model.getDefinedNameList();
  t.is(names.length, 2);
  t.is(names[0].name, "myname");
  t.is(names[0].formula, "Sheet1!$A$1");
  // a global name has no scope property (not even null)
  t.false("scope" in names[0]);
  t.is(names[1].name, "localname");
  t.is(names[1].scope, 0);
});

test('conditional formatting', (t) => {
  const model = new UserModel("Workbook1");
  model.addConditionalFormatting(0, "A1:B10", {
    type: "CellIs",
    operator: "GreaterThan",
    formula: "5",
    formula2: null,
    format: { fill: { color: "#FF0000" } },
    stop_if_true: false,
  });
  const rules = model.getConditionalFormattingList(0);
  t.is(rules.length, 1);
  t.is(rules[0].cf_rule.type, "CellIs");
  t.deepEqual(model.getDxfForConditionalFormatting(0, 0).fill, {
    color: "#FF0000",
  });
});

test('sheets', (t) => {
  const model = new UserModel("Workbook1");
  model.newSheet();
  model.renameSheet(1, "Data");
  model.setSheetColor(1, "#00FF00");
  const sheets = model.getWorksheetsProperties();
  t.is(sheets.length, 2);
  t.is(sheets[1].name, "Data");
  t.is(sheets[1].color, "#00FF00");
});

test('undo redo', (t) => {
  const model = new UserModel("Workbook1");
  t.false(model.canUndo());
  model.setUserInput(0, 1, 1, "42");
  t.true(model.canUndo());
  model.undo();
  t.is(model.getFormattedCellValue(0, 1, 1), "");
  t.true(model.canRedo());
  model.redo();
  t.is(model.getFormattedCellValue(0, 1, 1), "42");
});

test('bytes roundtrip', (t) => {
  const model = new UserModel("Workbook1");
  model.setUserInput(0, 1, 1, "=6*7");
  const clone = UserModel.fromBytes(model.toBytes());
  t.is(clone.getFormattedCellValue(0, 1, 1), "42");
});

test('collaboration diffs', (t) => {
  const model = new UserModel("Workbook1");
  const peer = UserModel.fromBytes(model.toBytes());
  model.setUserInput(0, 1, 1, "=6*7");
  peer.applyExternalDiffs(model.flushSendQueue());
  t.is(peer.getFormattedCellValue(0, 1, 1), "42");
});

test('theme colors', (t) => {
  const model = new UserModel("Workbook1");
  model.setSheetColor(0, [4, 0.2]);
  const sheets = model.getWorksheetsProperties();
  t.deepEqual(sheets[0].color, [4, 0.2]);
  t.regex(model.resolveColor([4, 0.2]), /^#[0-9A-F]{6}$/i);
  t.is(model.resolveColor(null), "");
  t.throws(() => model.setSheetColor(0, "#ZZZ"));
});

test('utility functions', (t) => {
  t.is(columnNameFromNumber(28), "AB");
  t.is(columnNumberFromName("AB"), 28);
  t.is(quoteName("My Sheet"), "'My Sheet'");
  t.true(getAllTimezones().includes("Europe/Berlin"));
  t.true(getSupportedLocales().includes("en"));
});

test('cell links', (t) => {
  const model = new UserModel("Workbook1");
  const external = { type: "External", target: "https://www.ironcalc.com/", tooltip: null };
  const internal = { type: "Internal", location: "Sheet1!A30", tooltip: "Jump!" };

  t.is(model.getCellLink(0, 2, 2), null);

  model.setCellLink(0, 2, 2, external);
  t.deepEqual(model.getCellLink(0, 2, 2), external);

  // tooltip is optional
  model.setCellLink(0, 5, 1, { type: "Internal", location: "Sheet1!A30", tooltip: "Jump!" });
  t.deepEqual(model.getLinks(0), [
    { row: 2, column: 2, dynamic: false, ...external },
    { row: 5, column: 1, dynamic: false, ...internal },
  ]);

  model.undo();
  model.undo();
  t.is(model.getCellLink(0, 2, 2), null);
  model.redo();
  t.deepEqual(model.getCellLink(0, 2, 2), external);

  model.deleteCellLink(0, 2, 2);
  t.is(model.getCellLink(0, 2, 2), null);

  t.throws(() => model.setCellLink(0, 0, 1, external));
});

test('cell links raw model', (t) => {
  const model = new Model("Workbook1");
  const external = { type: "External", target: "mailto:hello@ironcalc.com", tooltip: null };
  model.setCellLink(0, 1, 1, { type: "External", target: "mailto:hello@ironcalc.com" });
  t.deepEqual(model.getCellLink(0, 1, 1), external);
  t.deepEqual(model.getLinks(0), [{ row: 1, column: 1, dynamic: false, ...external }]);
  model.deleteCellLink(0, 1, 1);
  t.is(model.getCellLink(0, 1, 1), null);
});

test('cell link label and style are one undo step', (t) => {
  const model = new UserModel("Workbook1");
  model.setCellLink(0, 2, 2, { type: "External", target: "https://www.ironcalc.com/" }, "IronCalc");
  t.is(model.getFormattedCellValue(0, 2, 2), "IronCalc");
  t.true(model.getCellStyle(0, 2, 2).font.u);

  // one undo reverts the link, the content and the style together
  model.undo();
  t.is(model.getCellLink(0, 2, 2), null);
  t.is(model.getFormattedCellValue(0, 2, 2), "");
  t.falsy(model.getCellStyle(0, 2, 2).font.u);
  t.false(model.canUndo());

  // one redo restores everything
  model.redo();
  t.is(model.getFormattedCellValue(0, 2, 2), "IronCalc");
  t.true(model.getCellStyle(0, 2, 2).font.u);
  t.deepEqual(model.getCellLink(0, 2, 2), {
    type: "External",
    target: "https://www.ironcalc.com/",
    tooltip: null,
  });
});
