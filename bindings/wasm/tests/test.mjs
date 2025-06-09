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

test('getRecentDiffs returns recent diffs without modifying queue', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    // Perform some actions to generate diffs
    model.setUserInput(0, 1, 1, "42");
    model.setUserInput(0, 1, 2, "=A1*2");
    
    // Get recent diffs
    const diffs = model.getRecentDiffs();
    assert.strictEqual(diffs.length > 0, true, 'Diffs array should not be empty after actions');
    
    // Check structure of diffs - regular operations are marked as "Redo" type
    const firstDiff = diffs[0];
    assert.strictEqual(firstDiff.type, 'Redo', 'Regular operations should be of type Redo');
    assert.strictEqual(Array.isArray(firstDiff.list), true, 'Diff entry should have a list of diffs');
    assert.strictEqual(firstDiff.list.length > 0, true, 'Diff list should not be empty');
    
    // Look for SetCellValue diff in any of the diff entries
    let foundSetCellValue = false;
    for (const diffEntry of diffs) {
        const setCellDiff = diffEntry.list.find(d => d.SetCellValue && d.SetCellValue.row === 1 && d.SetCellValue.column === 1);
        if (setCellDiff) {
            assert.strictEqual(setCellDiff.SetCellValue.new_value, '42', 'New value for A1 should be 42');
            foundSetCellValue = true;
            break;
        }
    }
    assert.ok(foundSetCellValue, 'Should find a SetCellValue diff for cell A1 somewhere in the diffs');
    
    // Verify queue is not modified by checking again
    const diffsAgain = model.getRecentDiffs();
    assert.strictEqual(diffsAgain.length, diffs.length, 'Queue length should remain the same after multiple calls');
    assert.deepStrictEqual(diffsAgain, diffs, 'Queue contents should remain unchanged after multiple calls');
});

test('getRecentDiffs captures style changes', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    // Perform a style change
    model.updateRangeStyle({ sheet: 0, row: 1, column: 1, width: 1, height: 1 }, 'font.b', 'true');
    
    // Get recent diffs
    const diffs = model.getRecentDiffs();
    assert.strictEqual(diffs.length > 0, true, 'Diffs array should not be empty after style change');
    
    // Look for SetCellStyle diff in any of the diff entries 
    let foundStyleDiff = false;
    for (const diffEntry of diffs) {
        const styleDiff = diffEntry.list.find(d => d.SetCellStyle);
        if (styleDiff) {
            assert.strictEqual(styleDiff.SetCellStyle.sheet, 0, 'Sheet index should be 0');
            assert.strictEqual(styleDiff.SetCellStyle.row, 1, 'Row should be 1');
            assert.strictEqual(styleDiff.SetCellStyle.column, 1, 'Column should be 1');
            assert.ok(styleDiff.SetCellStyle.new_value.font.b, 'New style should have bold set to true');
            foundStyleDiff = true;
            break;
        }
    }
    assert.ok(foundStyleDiff, 'Should find a SetCellStyle diff after style update');
});

test('getRecentDiffs captures undo and redo diffs', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    // Perform an action and undo it
    model.setUserInput(0, 1, 1, "100");
    model.undo();
    
    // Get recent diffs
    const diffs = model.getRecentDiffs();
    assert.strictEqual(diffs.length > 0, true, 'Diffs array should not be empty after undo');
    
    // Check for Undo type in diffs
    const undoDiff = diffs.find(d => d.type === 'Undo');
    assert.ok(undoDiff, 'Should find an Undo diff entry after undo operation');
    assert.strictEqual(undoDiff.list.length > 0, true, 'Undo diff list should not be empty');
    
    // Redo the action
    model.redo();
    const diffsAfterRedo = model.getRecentDiffs();
    const redoDiff = diffsAfterRedo.find(d => d.type === 'Redo');
    assert.ok(redoDiff, 'Should find a Redo diff entry after redo operation');
});

test('getRecentDiffs captures setCellValue diff', () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    // Set a cell value to generate a SetCellValue diff
    model.setUserInput(0, 2, 3, "99");
    
    // Get recent diffs
    const diffs = model.getRecentDiffs();
    assert.strictEqual(diffs.length > 0, true, 'Diffs array should not be empty after setting cell value');
    
    // Look for SetCellValue diff in any of the diff entries
    let foundSetCellDiff = false;
    for (const diffEntry of diffs) {
        const setCellDiff = diffEntry.list.find(d => d.SetCellValue);
        if (setCellDiff) {
            assert.strictEqual(setCellDiff.SetCellValue.sheet, 0, 'Sheet index should be 0');
            assert.strictEqual(setCellDiff.SetCellValue.row, 2, 'Row should be 2');
            assert.strictEqual(setCellDiff.SetCellValue.column, 3, 'Column should be 3');
            assert.strictEqual(setCellDiff.SetCellValue.new_value, '99', 'New value should be 99');
            foundSetCellDiff = true;
            break;
        }
    }
    assert.ok(foundSetCellDiff, 'Should find a SetCellValue diff after setting cell value');
});



