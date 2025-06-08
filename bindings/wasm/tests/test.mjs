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

test("add sheets", (t) => {
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

test('invalid sheet index throws an exception', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    assert.throws(() => {
        model.setRowsHeight(1, 1, 1, 100);
    }, {
        name: 'Error',
        message: 'Invalid sheet index',
    });
});

test('invalid column throws an exception', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    assert.throws(() => {
        model.setRowsHeight(0, -1, 0, 100);
    }, {
        name: 'Error',
        message: "Row number '-1' is not valid.",
    });
});

test('floating column numbers get truncated', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    model.setRowsHeight(0.8, 5.2, 5.5, 100.5);

    assert.strictEqual(model.getRowHeight(0.11, 5.99), 100.5);
    assert.strictEqual(model.getRowHeight(0, 5), 100.5);
});

test('autofill', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    model.setUserInput(0, 1, 1, "23");
    model.autoFillRows({sheet: 0, row: 1, column: 1, width: 1, height: 1}, 2);

    const result = model.getFormattedCellValue(0, 2, 1);
    assert.strictEqual(result, "23");
});

test('track changed cells - basic update', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    model.setUserInput(0, 1, 1, "10");
    model.setUserInput(0, 1, 2, "=A1*2");
    model.evaluate();
    const changedCells = model.getChangedCells();
    assert.strictEqual(changedCells.length, 1, 'Changed cells should include directly set cell and dependent cell');
    assert.deepEqual(changedCells[0], { sheet: 0, row: 1, column: 2 }, 'Second changed cell should be B1');
});

test('track changed cells - circular dependency with external dependent', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    // Setup circular dependency: A1 -> B1 -> C1 -> A1
    model.setUserInput(0, 1, 1, "=B1");
    model.setUserInput(0, 1, 2, "=C1");
    model.setUserInput(0, 1, 3, "=A1");
    // Setup external dependent: D1 depends on A1
    model.setUserInput(0, 1, 4, "=A1+1");
    // Evaluate to set initial state
    model.evaluate();
    // Update A1 to trigger circular dependency error
    model.setUserInput(0, 1, 1, "=B1+10");
    model.evaluate();
    // Get changed cells
    const changedCells = model.getChangedCells();
    // Check if dependent cells are tracked as changed, excluding A1 which was directly updated
    assert.strictEqual(changedCells.some(c => c.sheet === 0 && c.row === 1 && c.column === 2), true, 'B1 should be tracked as changed due to circular dependency');
    assert.strictEqual(changedCells.some(c => c.sheet === 0 && c.row === 1 && c.column === 3), true, 'C1 should be tracked as changed due to circular dependency');
    assert.strictEqual(changedCells.some(c => c.sheet === 0 && c.row === 1 && c.column === 4), true, 'D1 should be tracked as changed due to dependency on A1');
});

test('track changed cells - multi-sheet cascading with defined name', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    // Add additional sheets
    model.newSheet();
    model.renameSheet(1, "Sheet2");
    model.newSheet();
    model.renameSheet(2, "Sheet3");
    // Define a name 'TotalSales' for Sheet1!A1:A2
    model.newDefinedName("TotalSales", 0, "=Sheet1!A1:A2");
    // Set values in Sheet1
    model.setUserInput(0, 1, 1, "100");
    model.setUserInput(0, 2, 1, "200");
    // Set formula in Sheet2 using defined name
    model.setUserInput(1, 2, 2, "=SUM(TotalSales)");
    // Set formula in Sheet3 referencing Sheet2!B2
    model.setUserInput(2, 3, 3, "=Sheet2!B2*2");
    // Evaluate initial state
    model.evaluate();
    // Update Sheet1!A1 to trigger cascading changes
    model.setUserInput(0, 1, 1, "150");
    model.evaluate();
    // Get changed cells
    const changedCells = model.getChangedCells();
    // Verify only dependent cells are tracked, excluding Sheet1!A1 which was directly updated
    assert.strictEqual(changedCells.some(c => c.sheet === 1 && c.row === 2 && c.column === 2), true, 'Sheet2!B2 should be tracked as changed');
    assert.strictEqual(changedCells.some(c => c.sheet === 2 && c.row === 3 && c.column === 3), true, 'Sheet3!C3 should be tracked as changed');
});

test('track changed cells - move row updates formulas', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    model.setUserInput(0, 1, 1, "10");
    model.setUserInput(0, 2, 2, "=A1*2");
    model.evaluate();
    assert.strictEqual(model.getFormattedCellValue(0, 2, 2), "20");
    // Move row 1 to row 3
    model.insertRow(0, 1);
    model.insertRow(0, 1);
    model.evaluate();
    const changedCells = model.getChangedCells();
    assert.strictEqual(changedCells.length, 1, 'One cell should be marked as changed after row insertion');
    assert.deepEqual(changedCells[0], { sheet: 0, row: 4, column: 2 }, 'Changed cell should be B4 due to formula update after row shift');
});

test('track changed cells - move column updates formulas', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    model.setUserInput(0, 1, 1, "5");
    model.setUserInput(0, 1, 2, "=A1+3");
    model.evaluate();
    assert.strictEqual(model.getFormattedCellValue(0, 1, 2), "8");
    // Insert a column before column 1, shifting existing columns
    model.insertColumn(0, 1);
    model.evaluate();
    const changedCells = model.getChangedCells();
    assert.strictEqual(changedCells.length, 1, 'One cell should be marked as changed after column insertion');
    assert.deepEqual(changedCells[0], { sheet: 0, row: 1, column: 3 }, 'Changed cell should be C1 due to formula update after column shift');
});
