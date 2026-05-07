import test from 'node:test';
import assert from 'node:assert'
import { Model } from "../pkg/wasm.js";

const DEFAULT_ROW_HEIGHT = 28;

test('Frozen rows and columns', () => {
    let model = new Model('Workbook1', 'en', 'UTC', 'en');
    assert.strictEqual(model.getFrozenRowsCount(0), 0);
    assert.strictEqual(model.getFrozenColumnsCount(0), 0);

    model.setFrozenColumnsCount(0, 4);
    model.setFrozenRowsCount(0, 3)

    assert.strictEqual(model.getFrozenRowsCount(0), 3);
    assert.strictEqual(model.getFrozenColumnsCount(0), 4);
});

test('Row height', () => {
    let model = new Model('Workbook1', 'en', 'UTC', 'en');
    assert.strictEqual(model.getRowHeight(0, 3), DEFAULT_ROW_HEIGHT);

    model.setRowsHeight(0, 3, 3, 32);
    assert.strictEqual(model.getRowHeight(0, 3), 32);

    model.undo();
    assert.strictEqual(model.getRowHeight(0, 3), DEFAULT_ROW_HEIGHT);

    model.redo();
    assert.strictEqual(model.getRowHeight(0, 3), 32);

    model.setRowsHeight(0, 3, 3, 320);
    assert.strictEqual(model.getRowHeight(0, 3), 320);
});

test('Evaluates correctly', (t) => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.setUserInput(0, 1, 1, "23");
    model.setUserInput(0, 1, 2, "=A1*3+1");

    const result = model.getFormattedCellValue(0, 1, 2);
    assert.strictEqual(result, "70");
});

const DEFAULT_STYLE = {
    num_fmt: 'general',
    fill: { pattern_type: 'none' },
    font: { sz: 13, color: '#000000', name: 'Calibri', family: 2, scheme: 'minor' },
    border: {},
    quote_prefix: false,
};

test('Styles work', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    // getCellStyle now returns ExtendedCellStyle: { style, icon, data_bar, custom_icon }
    let extended = model.getCellStyle(0, 1, 1);
    assert.deepEqual(extended.style, DEFAULT_STYLE);
    assert.strictEqual(extended.icon, undefined);
    assert.strictEqual(extended.data_bar, undefined);
    assert.strictEqual(extended.custom_icon, undefined);

    model.setUserInput(0, 1, 1, "'=1+1");
    extended = model.getCellStyle(0, 1, 1);
    assert.deepEqual(extended.style, { ...DEFAULT_STYLE, quote_prefix: true });
});

test("Add sheets", (t) => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.newSheet();
    model.renameSheet(1, "NewName");
    let props = model.getWorksheetsProperties();
    assert.deepEqual(props, [{
        name: 'Sheet1',
        sheet_id: 1,
        state: 'visible'
    },
    {
        name: 'NewName',
        sheet_id: 2,
        state: 'visible'
    }
    ]);
});

test("invalid sheet index throws an exception", () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    assert.throws(() => {
        model.setRowsHeight(1, 1, 1, 100);
    }, {
        name: 'Error',
        message: 'Invalid sheet index',
    });
});

test("invalid column throws an exception", () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    assert.throws(() => {
        model.setRowsHeight(0, -1, 0, 100);
    }, {
        name: 'Error',
        message: "Row number '-1' is not valid.",
    });
});

test("floating column numbers get truncated", () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.setRowsHeight(0.8, 5.2, 5.5, 100.5);

    assert.strictEqual(model.getRowHeight(0.11, 5.99), 100.5);
    assert.strictEqual(model.getRowHeight(0, 5), 100.5);
});

test("autofill", () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.setUserInput(0, 1, 1, "23");
    model.autoFillRows({sheet: 0, row: 1, column: 1, width: 1, height: 1}, 2);

    const result = model.getFormattedCellValue(0, 2, 1);
    assert.strictEqual(result, "23");
});

test('insertRows shifts cells', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.setUserInput(0, 1, 1, '42');
    model.insertRows(0, 1, 1);

    assert.strictEqual(model.getCellContent(0, 1, 1), '');
    assert.strictEqual(model.getCellContent(0, 2, 1), '42');
});

test('insertColumns shifts cells', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.setUserInput(0, 1, 1, 'A');
    model.setUserInput(0, 1, 2, 'B');

    model.insertColumns(0, 2, 1);

    assert.strictEqual(model.getCellContent(0, 1, 2), '');
    assert.strictEqual(model.getCellContent(0, 1, 3), 'B');
});

test('deleteRows removes cells', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.setUserInput(0, 1, 1, '1');
    model.setUserInput(0, 2, 1, '2');

    model.deleteRows(0, 1, 1);

    assert.strictEqual(model.getCellContent(0, 1, 1), '2');
    assert.strictEqual(model.getCellContent(0, 2, 1), '');
});

test('deleteColumns removes cells', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.setUserInput(0, 1, 1, 'A');
    model.setUserInput(0, 1, 2, 'B');

    model.deleteColumns(0, 1, 1);

    assert.strictEqual(model.getCellContent(0, 1, 1), 'B');
    assert.strictEqual(model.getCellContent(0, 1, 2), '');
});

test("move rows", () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.setUserInput(0, 3, 5, "=G3");
    model.setUserInput(0, 4, 5, "=G4");
    model.setUserInput(0, 5, 5, "=SUM(G3:J3)");
    model.setUserInput(0, 6, 5, "=SUM(G3:G3)");
    model.setUserInput(0, 7, 5, "=SUM(G4:G4)");
    model.evaluate();

    model.moveRows(0, 3, 1, 1);
    model.evaluate();

    assert.strictEqual(model.getCellContent(0, 3, 5), "=G3");
    assert.strictEqual(model.getCellContent(0, 4, 5), "=G4");
    assert.strictEqual(model.getCellContent(0, 5, 5), "=SUM(G4:J4)");
    assert.strictEqual(model.getCellContent(0, 6, 5), "=SUM(G4:G4)");
    assert.strictEqual(model.getCellContent(0, 7, 5), "=SUM(G3:G3)");
});

test("move columns", () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.setUserInput(0, 3, 5, "=G3");
    model.setUserInput(0, 4, 5, "=H3");
    model.setUserInput(0, 5, 5, "=SUM(G3:J7)");
    model.setUserInput(0, 6, 5, "=SUM(G3:G7)");
    model.setUserInput(0, 7, 5, "=SUM(H3:H7)");
    model.evaluate();

    model.moveColumns(0, 7, 1, 1);
    model.evaluate();

    assert.strictEqual(model.getCellContent(0, 3, 5), "=H3");
    assert.strictEqual(model.getCellContent(0, 4, 5), "=G3");
    assert.strictEqual(model.getCellContent(0, 5, 5), "=SUM(H3:J7)");
    assert.strictEqual(model.getCellContent(0, 6, 5), "=SUM(H3:H7)");
    assert.strictEqual(model.getCellContent(0, 7, 5), "=SUM(G3:G7)");
});

// ---------------------------------------------------------------------------
// Conditional formatting
// ---------------------------------------------------------------------------

const COLOR_SCALE = { type: 'ColorScale', cfvo: ['Min', 'Max'], colors: ['#FF0000', '#00FF00'] };
const DATA_BAR    = { type: 'DataBar', cfvo: ['Min', 'Max'], color: '#0000FF', show_value: true };

test('CF: empty list initially', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    assert.deepEqual(model.getConditionalFormattingList(0), []);
});

test('CF: add and retrieve', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.addConditionalFormatting(0, 'A1:A5', COLOR_SCALE);
    const list = model.getConditionalFormattingList(0);
    assert.strictEqual(list.length, 1);
    assert.strictEqual(list[0].range, 'A1:A5');
    assert.deepEqual(list[0].cf_rule, COLOR_SCALE);
    assert.strictEqual(typeof list[0].priority, 'number');
});

test('CF: add two rules, priorities increase', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.addConditionalFormatting(0, 'A1:A5', COLOR_SCALE);
    model.addConditionalFormatting(0, 'B1:B5', DATA_BAR);
    const list = model.getConditionalFormattingList(0);
    assert.strictEqual(list.length, 2);
    assert.ok(list[0].priority < list[1].priority);
});

test('CF: delete removes rule', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.addConditionalFormatting(0, 'A1:A5', COLOR_SCALE);
    model.deleteConditionalFormatting(0, 0);
    assert.deepEqual(model.getConditionalFormattingList(0), []);
});

test('CF: update changes range and rule', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.addConditionalFormatting(0, 'A1:A5', COLOR_SCALE);
    model.updateConditionalFormatting(0, 0, 'C1:C10', DATA_BAR);
    const list = model.getConditionalFormattingList(0);
    assert.strictEqual(list[0].range, 'C1:C10');
    assert.deepEqual(list[0].cf_rule, DATA_BAR);
});

test('CF: undo add removes rule', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.addConditionalFormatting(0, 'A1:A5', COLOR_SCALE);
    model.undo();
    assert.deepEqual(model.getConditionalFormattingList(0), []);
});

test('CF: redo add restores rule', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.addConditionalFormatting(0, 'A1:A5', COLOR_SCALE);
    model.undo();
    model.redo();
    const list = model.getConditionalFormattingList(0);
    assert.strictEqual(list.length, 1);
    assert.strictEqual(list[0].range, 'A1:A5');
});

test('CF: invalid sheet throws', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    assert.throws(() => model.addConditionalFormatting(99, 'A1:A5', COLOR_SCALE));
});

test('CF: invalid range throws', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    assert.throws(() => model.addConditionalFormatting(0, 'not_a_range', COLOR_SCALE));
});

test('CF: color scale applies fill to cells', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    for (let i = 1; i <= 5; i++) model.setUserInput(0, i, 1, String(i));
    model.addConditionalFormatting(0, 'A1:A5', COLOR_SCALE);
    // auto-evaluated — check that A1 (min) gets the min color
    const extended = model.getCellStyle(0, 1, 1);
    assert.strictEqual(extended.style.fill.bg_color, '#FF0000');
});

test('CF: data bar sets fill proportion on cells', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    for (let i = 1; i <= 5; i++) model.setUserInput(0, i, 1, String(i));
    model.addConditionalFormatting(0, 'A1:A5', DATA_BAR);
    const a1 = model.getCellStyle(0, 1, 1);
    const a5 = model.getCellStyle(0, 5, 1);
    assert.strictEqual(a1.data_bar.value, 0);
    assert.strictEqual(a5.data_bar.value, 1);
});
