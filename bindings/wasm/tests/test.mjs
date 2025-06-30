import test from 'node:test';
import assert from 'node:assert'
import { Model } from "../pkg/wasm.js";

const DEFAULT_ROW_HEIGHT = 28;

test('Frozen rows and columns', () => {
    let model = new Model('Workbook1', 'en', 'UTC');
    assert.strictEqual(model.getFrozenRowsCount(0), 0);
    assert.strictEqual(model.getFrozenColumnsCount(0), 0);

    model.setFrozenColumnsCount(0, 4);
    model.setFrozenRowsCount(0, 3)

    assert.strictEqual(model.getFrozenRowsCount(0), 3);
    assert.strictEqual(model.getFrozenColumnsCount(0), 4);
});

test('Row height', () => {
    let model = new Model('Workbook1', 'en', 'UTC');
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
    const model = new Model('Workbook1', 'en', 'UTC');
    model.setUserInput(0, 1, 1, "23");
    model.setUserInput(0, 1, 2, "=A1*3+1");

    const result = model.getFormattedCellValue(0, 1, 2);
    assert.strictEqual(result, "70");
});

test('Styles work', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    let style = model.getCellStyle(0, 1, 1);
    assert.deepEqual(style, {
        num_fmt: 'general',
        fill: { pattern_type: 'none' },
        font: {
            sz: 13,
            color: '#000000',
            name: 'Calibri',
            family: 2,
            scheme: 'minor'
        },
        border: {},
        quote_prefix: false
    });
    model.setUserInput(0, 1, 1, "'=1+1");
    style = model.getCellStyle(0, 1, 1);
    assert.deepEqual(style, {
        num_fmt: 'general',
        fill: { pattern_type: 'none' },
        font: {
            sz: 13,
            color: '#000000',
            name: 'Calibri',
            family: 2,
            scheme: 'minor'
        },
        border: {},
        quote_prefix: true
    });
});

test("Add sheets", (t) => {
    const model = new Model('Workbook1', 'en', 'UTC');
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
    const model = new Model('Workbook1', 'en', 'UTC');
    assert.throws(() => {
        model.setRowsHeight(1, 1, 1, 100);
    }, {
        name: 'Error',
        message: 'Invalid sheet index',
    });
});

test("invalid column throws an exception", () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    assert.throws(() => {
        model.setRowsHeight(0, -1, 0, 100);
    }, {
        name: 'Error',
        message: "Row number '-1' is not valid.",
    });
});

test("floating column numbers get truncated", () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    model.setRowsHeight(0.8, 5.2, 5.5, 100.5);

    assert.strictEqual(model.getRowHeight(0.11, 5.99), 100.5);
    assert.strictEqual(model.getRowHeight(0, 5), 100.5);
});

test("autofill", () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    model.setUserInput(0, 1, 1, "23");
    model.autoFillRows({sheet: 0, row: 1, column: 1, width: 1, height: 1}, 2);

    const result = model.getFormattedCellValue(0, 2, 1);
    assert.strictEqual(result, "23");
});

test('insertRows shifts cells', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    model.setUserInput(0, 1, 1, '42');
    model.insertRows(0, 1, 1);

    assert.strictEqual(model.getCellContent(0, 1, 1), '');
    assert.strictEqual(model.getCellContent(0, 2, 1), '42');
});

test('insertColumns shifts cells', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    model.setUserInput(0, 1, 1, 'A');
    model.setUserInput(0, 1, 2, 'B');

    model.insertColumns(0, 2, 1);

    assert.strictEqual(model.getCellContent(0, 1, 2), '');
    assert.strictEqual(model.getCellContent(0, 1, 3), 'B');
});

test('deleteRows removes cells', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    model.setUserInput(0, 1, 1, '1');
    model.setUserInput(0, 2, 1, '2');

    model.deleteRows(0, 1, 1);

    assert.strictEqual(model.getCellContent(0, 1, 1), '2');
    assert.strictEqual(model.getCellContent(0, 2, 1), '');
});

test('deleteColumns removes cells', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    model.setUserInput(0, 1, 1, 'A');
    model.setUserInput(0, 1, 2, 'B');

    model.deleteColumns(0, 1, 1);

    assert.strictEqual(model.getCellContent(0, 1, 1), 'B');
    assert.strictEqual(model.getCellContent(0, 1, 2), '');
});

test("move row", () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    model.setUserInput(0, 3, 5, "=G3");
    model.setUserInput(0, 4, 5, "=G4");
    model.setUserInput(0, 5, 5, "=SUM(G3:J3)");
    model.setUserInput(0, 6, 5, "=SUM(G3:G3)");
    model.setUserInput(0, 7, 5, "=SUM(G4:G4)");
    model.evaluate();

    model.moveRow(0, 3, 1);
    model.evaluate();

    assert.strictEqual(model.getCellContent(0, 3, 5), "=G3");
    assert.strictEqual(model.getCellContent(0, 4, 5), "=G4");
    assert.strictEqual(model.getCellContent(0, 5, 5), "=SUM(G4:J4)");
    assert.strictEqual(model.getCellContent(0, 6, 5), "=SUM(G4:G4)");
    assert.strictEqual(model.getCellContent(0, 7, 5), "=SUM(G3:G3)");
});

test("move column", () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    model.setUserInput(0, 3, 5, "=G3");
    model.setUserInput(0, 4, 5, "=H3");
    model.setUserInput(0, 5, 5, "=SUM(G3:J7)");
    model.setUserInput(0, 6, 5, "=SUM(G3:G7)");
    model.setUserInput(0, 7, 5, "=SUM(H3:H7)");
    model.evaluate();

    model.moveColumn(0, 7, 1);
    model.evaluate();

    assert.strictEqual(model.getCellContent(0, 3, 5), "=H3");
    assert.strictEqual(model.getCellContent(0, 4, 5), "=G3");
    assert.strictEqual(model.getCellContent(0, 5, 5), "=SUM(H3:J7)");
    assert.strictEqual(model.getCellContent(0, 6, 5), "=SUM(H3:H7)");
    assert.strictEqual(model.getCellContent(0, 7, 5), "=SUM(G3:G7)");
});
