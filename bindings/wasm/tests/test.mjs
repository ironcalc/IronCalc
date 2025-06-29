import test from 'node:test';
import assert from 'node:assert'
import { Model } from "../pkg/ironcalc.js";
import { fromXLSXBytes, toXLSXBytes } from "../pkg/xlsx.js";

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

test('toXLSXBytes returns data', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    const bytes = toXLSXBytes(model.toBytes());
    assert.ok(bytes instanceof Uint8Array);
    assert.ok(bytes.length > 0);
});

test('toBytes returns data', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    const bytes = model.toBytes();
    assert.ok(bytes instanceof Uint8Array);
    assert.ok(bytes.length > 0);
});

test('fromBytes loads model', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    model.setUserInput(0, 1, 1, '42');
    const bytes = model.toBytes();
    const m2 = Model.fromBytes(bytes);
    assert.strictEqual(m2.getCellContent(0, 1, 1), '42');
});

test('fromXLSXBytes loads model', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    model.setUserInput(0, 1, 1, '5');
    const xlsxBytes = toXLSXBytes(model.toBytes());
    const modelBytes = fromXLSXBytes(xlsxBytes, 'Workbook1', 'en', 'UTC');
    const m2 = Model.fromBytes(modelBytes);
    assert.strictEqual(m2.getCellContent(0, 1, 1), '5');
});

test('roundtrip via xlsx bytes', () => {
    const m1 = new Model('Workbook1', 'en', 'UTC');
    m1.setUserInput(0, 1, 1, '7');
    m1.setUserInput(0, 1, 2, '=A1*3');
    const xlsxBytes = toXLSXBytes(m1.toBytes());
    const m2Bytes = fromXLSXBytes(xlsxBytes, 'Workbook1', 'en', 'UTC');
    const m2 = Model.fromBytes(m2Bytes);
    m2.evaluate();
    assert.strictEqual(m2.getFormattedCellValue(0, 1, 2), '21');
});

test('roundtrip via bytes', () => {
    const m1 = new Model('Workbook1', 'en', 'UTC');
    m1.setUserInput(0, 1, 1, '9');
    m1.setUserInput(0, 1, 2, '=A1*4');
    const bytes = m1.toBytes();
    const m2 = Model.fromBytes(bytes);
    m2.evaluate();
    assert.strictEqual(m2.getFormattedCellValue(0, 1, 2), '36');
});
