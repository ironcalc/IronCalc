import test from 'node:test';
import assert from 'node:assert'
import { Model } from "../pkg/wasm.js";
import { setTimeout } from 'node:timers/promises';

const DEFAULT_ROW_HEIGHT = 28;

// Helper to sort cells for consistent comparison
const sortCells = (cells) => cells.sort((a, b) => {
    if (a.sheet !== b.sheet) return a.sheet - b.sheet;
    if (a.row !== b.row) return a.row - b.row;
    return a.column - b.column;
});

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


test('onDiffs', async () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    const events = [];
    
    model.onDiffs(diffs => {
        events.push(...diffs);
    });
    
    model.setUserInput(0, 1, 1, 'test');
    await setTimeout(0);
    
    const expectedEvents = [
        {
            SetCellValue: {
                sheet: 0,
                row: 1,
                column: 1,
                new_value: 'test',
                old_value: undefined
            }
        },
    ];
    
    // Verify we got the expected number of events
    assert.strictEqual(events.length, expectedEvents.length, `Should have exactly ${expectedEvents.length} diff events`);
    
    // Compare each event with deep equality
    for (let i = 0; i < expectedEvents.length; i++) {
        assert.deepStrictEqual(events[i], expectedEvents[i], `Event ${i} should match expected diff`);
    }
});


test('onDiffs emits correct diff types for various operations', async () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    const events = [];
    
    model.onDiffs(diffs => {
        events.push(...diffs);
    });
    
    // Test various operations that should emit different diff types
    model.setUserInput(0, 1, 1, '42');
    model.insertRow(0, 2);                                 
    model.setRowsHeight(0, 1, 1, 35);                     
    model.insertColumn(0, 2);                             
    model.setColumnsWidth(0, 1, 1, 120);                  
    model.newSheet();                                     
    model.renameSheet(1, "TestSheet");                    
    model.setFrozenRowsCount(0, 2);                       
    model.setFrozenColumnsCount(0, 3);                    
    
    // Allow any async operations to complete
    await setTimeout(0);
    
    const expectedEvents = [
        {
            SetCellValue: {
                sheet: 0,
                row: 1,
                column: 1,
                new_value: '42',
                old_value: undefined
            }
        },
        {
            InsertRow: {
                sheet: 0,
                row: 2
            }
        },
        {
            SetRowHeight: {
                sheet: 0,
                row: 1,
                new_value: 35,
                old_value: 28
            }
        },
        {
            InsertColumn: {
                sheet: 0,
                column: 2
            }
        },
        {
            SetColumnWidth: {
                sheet: 0,
                column: 1,
                new_value: 120,
                old_value: 125
            }
        },
        {
            NewSheet: {
                index: 1,
                name: 'Sheet2'
            }
        },
        {
            RenameSheet: {
                index: 1,
                old_value: 'Sheet2',
                new_value: 'TestSheet'
            }
        },
        {
            SetFrozenRowsCount: {
                sheet: 0,
                new_value: 2,
                old_value: 0
            }
        },
        {
            SetFrozenColumnsCount: {
                sheet: 0,
                new_value: 3,
                old_value: 0
            }
        }
    ];
    
    // Verify we got the expected number of events
    assert.strictEqual(events.length, expectedEvents.length, `Should have exactly ${expectedEvents.length} diff events`);
    
    // Compare each event with deep equality
    for (let i = 0; i < expectedEvents.length; i++) {
        assert.deepStrictEqual(events[i], expectedEvents[i], `Event ${i} should match expected diff`);
    }
});

test('onDiffs emits full diff objects for undo/redo operations', async () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    const events = [];
    
    model.onDiffs(diffs => {
        events.push(...diffs);
    });
    
    // Perform initial operations
    model.setUserInput(0, 1, 1, 'Hello');
    model.setUserInput(0, 1, 2, 'World'); 
    model.insertRow(0, 2);
    
    // Test undo - should emit diffs for undoing operations
    model.undo();
    model.undo();
    
    // Test redo - should emit diffs for redoing operations  
    model.redo();
    model.redo();
    
    await setTimeout(0);
    
    const expectedEvents = [
        // Initial operations (3 events)
        {
            SetCellValue: {
                sheet: 0,
                row: 1,
                column: 1,
                new_value: 'Hello',
                old_value: undefined
            }
        },
        {
            SetCellValue: {
                sheet: 0,
                row: 1,
                column: 2,
                new_value: 'World',
                old_value: undefined
            }
        },
        {
            InsertRow: {
                sheet: 0,
                row: 2
            }
        },
        // Undo operations (2 events) - Note: these emit the same diff structures as the forward operations
        {
            InsertRow: {
                sheet: 0,
                row: 2
            }
        },
        {
            SetCellValue: {
                sheet: 0,
                row: 1,
                column: 2,
                new_value: 'World',
                old_value: undefined
            }
        },
        // Redo operations (2 events) - These also emit the same diff structures
        {
            SetCellValue: {
                sheet: 0,
                row: 1,
                column: 2,
                new_value: 'World',
                old_value: undefined
            }
        },
        {
            InsertRow: {
                sheet: 0,
                row: 2
            }
        }
    ];
    
    // Verify we got the expected number of events
    assert.strictEqual(events.length, expectedEvents.length, `Should have exactly ${expectedEvents.length} diff events`);
    
    // Compare each event with deep equality
    for (let i = 0; i < expectedEvents.length; i++) {
        assert.deepStrictEqual(events[i], expectedEvents[i], `Event ${i} should match expected diff`);
    }
});

test('onDiffs handles multiple subscribers and provides full diff objects', async () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    const events = [];
    
    model.onDiffs(diffs => {
        events.push(...diffs);
    });
    
    // Perform complex operations that generate multiple diffs
    model.setUserInput(0, 1, 1, '=SUM(A2:A5)');
    model.setUserInput(0, 2, 1, '10');           
    model.setUserInput(0, 3, 1, '20');           
    model.setUserInput(0, 4, 1, '30');           
    
    // Test row operations
    model.insertRow(0, 2);
    model.deleteRow(0, 2);
    
    // Test range operations
    model.rangeClearContents(0, 2, 1, 3, 1);
    
    await setTimeout(0);
    
    const expectedEvents = [
        // SetUserInput operations (4 events)
        {
            SetCellValue: {
                sheet: 0,
                row: 1,
                column: 1,
                new_value: '=SUM(A2:A5)',
                old_value: undefined
            }
        },
        {
            SetCellValue: {
                sheet: 0,
                row: 2,
                column: 1,
                new_value: '10',
                old_value: undefined
            }
        },
        {
            SetCellValue: {
                sheet: 0,
                row: 3,
                column: 1,
                new_value: '20',
                old_value: undefined
            }
        },
        {
            SetCellValue: {
                sheet: 0,
                row: 4,
                column: 1,
                new_value: '30',
                old_value: undefined
            }
        },
        // Row operations (2 events)
        {
            InsertRow: {
                sheet: 0,
                row: 2
            }
        },
        {
            DeleteRow: {
                sheet: 0,
                row: 2,
                old_data: {
                    data: new Map(),
                    row: undefined
                }
            }
        },
        // Range clear operations (2 events)
        {
            CellClearContents: {
                sheet: 0,
                row: 2,
                column: 1,
                old_value: {
                    NumberCell: {
                        v: 10,
                        s: 0
                    }
                }
            }
        },
        {
            CellClearContents: {
                sheet: 0,
                row: 3,
                column: 1,
                old_value: {
                    NumberCell: {
                        v: 20,
                        s: 0
                    }
                }
            }
        }
    ];
    
    // Verify we got the expected number of events
    assert.strictEqual(events.length, expectedEvents.length, `Should have exactly ${expectedEvents.length} diff events`);
    
    // Compare each event with deep equality
    for (let i = 0; i < expectedEvents.length; i++) {
        assert.deepStrictEqual(events[i], expectedEvents[i], `Event ${i} should match expected diff`);
    }
});

test('onDiffs returns unregister function that works correctly', async () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    const events1 = [];
    const events2 = [];
    
    // Register two listeners
    const unregister1 = model.onDiffs(diffs => {
        events1.push(...diffs);
    });
    
    const unregister2 = model.onDiffs(diffs => {
        events2.push(...diffs);
    });
    
    // Both should be functions
    assert.strictEqual(typeof unregister1, 'function', 'onDiffs should return a function');
    assert.strictEqual(typeof unregister2, 'function', 'onDiffs should return a function');
    
    // Trigger some events
    model.setUserInput(0, 1, 1, 'Test');
    model.setUserInput(0, 1, 2, 'Test2');
    
    await setTimeout(0);
    
    // Both listeners should have received events
    assert.strictEqual(events1.length, 2, 'First listener should receive 2 events');
    assert.strictEqual(events2.length, 2, 'Second listener should receive 2 events');
    
    // Unregister first listener
    unregister1();
    
    // Trigger more events
    model.setUserInput(0, 1, 3, 'Test3');
    
    await setTimeout(0);
    
    assert.strictEqual(events1.length, 2, 'First listener should receive 2 events');
    assert.strictEqual(events2.length, 3, 'Second listener should receive 2 events');
    
    // Call the second unregister too
    unregister2();
});

test('onCellsEvaluated tracks formula evaluation', async () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    const evaluatedCells = [];
    
    model.onCellsEvaluated(cells => {
        evaluatedCells.push(...cells);
    });
    
    // Set up a formula and its dependencies
    model.setUserInput(0, 1, 1, '=A2+A3');  // A1 = A2 + A3
    model.setUserInput(0, 2, 1, '10');       // A2 = 10
    model.setUserInput(0, 3, 1, '20');       // A3 = 20
    model.evaluate();
    
    // Update a dependency to trigger re-evaluation
    model.setUserInput(0, 2, 1, '15');  // Change A2 from 10 to 15
    model.evaluate();

    await setTimeout(0);
    
    const expectedCells = [
        // First evaluation of A1
        { sheet: 0, row: 1, column: 1 },
        // Re-evaluation of A1 after dependency changed
        { sheet: 0, row: 1, column: 1 },
    ];

    assert.strictEqual(evaluatedCells.length, expectedCells.length, `Should have exactly ${expectedCells.length} cell evaluation events`);
    assert.deepStrictEqual(sortCells(evaluatedCells), sortCells(expectedCells), 'The evaluated cells should match the expected cells');
    
    // Verify the formula calculated correctly
    const result = model.getFormattedCellValue(0, 1, 1);
    assert.strictEqual(result, '35', 'Formula should calculate 15 + 20 = 35');
});

test('onCellsEvaluated tracks complex formula dependencies', async () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    const evaluatedCells = [];
    
    model.onCellsEvaluated(cells => {
        evaluatedCells.push(...cells);
    });
    
    // Set up a chain of formulas: A1 -> B1 -> C1
    model.setUserInput(0, 1, 1, '10');        // A1 = 10 (base value)
    model.setUserInput(0, 1, 2, '=A1*2');     // B1 = A1 * 2
    model.setUserInput(0, 1, 3, '=B1+5');     // C1 = B1 + 5
    model.evaluate();
    
    // Update the root value to trigger cascade re-evaluation
    model.setUserInput(0, 1, 1, '5');  // Change A1 from 10 to 5
    model.evaluate();
    
    await setTimeout(0);

    const expectedCells = [
        // Initial evaluation
        { sheet: 0, row: 1, column: 2 }, // B1
        { sheet: 0, row: 1, column: 3 }, // C1
        // Re-evaluation after dependency change
        { sheet: 0, row: 1, column: 2 }, // B1
        { sheet: 0, row: 1, column: 3 }, // C1
    ];
    
    assert.strictEqual(evaluatedCells.length, expectedCells.length, `Should have exactly ${expectedCells.length} cell evaluation events`);
    assert.deepStrictEqual(sortCells(evaluatedCells), sortCells(expectedCells), 'The evaluated cells should match the expected cells');
    
    // Verify the formulas calculated correctly
    assert.strictEqual(model.getFormattedCellValue(0, 1, 2), '10', 'B1 should be 5 * 2 = 10');
    assert.strictEqual(model.getFormattedCellValue(0, 1, 3), '15', 'C1 should be 10 + 5 = 15');
});

test('onCellsEvaluated only fires after evaluate', async () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    const evaluatedCells = [];

    model.onCellsEvaluated(cells => {
        evaluatedCells.push(...cells);
    });

    // Set a formula
    model.setUserInput(0, 1, 1, '=1+1');

    // No cells should be evaluated yet because model.evaluate() has not been called
    assert.strictEqual(evaluatedCells.length, 0, 'evaluatedCells should be empty before evaluate()');

    // Now, trigger the evaluation
    model.evaluate();
    await setTimeout(0);

    // Now, the cell should be evaluated
    const expectedCells = [
        { sheet: 0, row: 1, column: 1 },
    ];

    assert.deepStrictEqual(sortCells(evaluatedCells), sortCells(expectedCells), 'evaluatedCells should contain the evaluated cell after evaluate()');
});

test('onCellsEvaluated returns unsubscribe function', async () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    const evaluatedCells = [];
    
    const unsubscribe = model.onCellsEvaluated(cells => {
        evaluatedCells.push(...cells);
    });
    
    assert.strictEqual(typeof unsubscribe, 'function', 'onCellsEvaluated should return a function');
    
    // Set up a formula
    model.setUserInput(0, 1, 1, '=A2*2');
    model.setUserInput(0, 2, 1, '5');
    
    model.evaluate();
    await setTimeout(0);
    
    assert.ok(evaluatedCells.length > 0, 'Should have tracked evaluated cells');
    
    // Unsubscribe
    unsubscribe();
    evaluatedCells.length = 0;
    
    // Update to trigger re-evaluation
    model.setUserInput(0, 2, 1, '10');
    
    model.evaluate();
    await setTimeout(0);
    
    assert.strictEqual(evaluatedCells.length, 0, 'Should not track cells after unsubscribing');
});