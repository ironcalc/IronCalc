import test from 'node:test';
import assert from 'node:assert'
import { Model } from "../pkg/wasm.js";
import { setTimeout } from 'node:timers/promises';

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


test('onDiffs', async () => {
    const model = new Model('Workbook1', 'en', 'UTC');
    const events = [];
    
    model.onDiffs(diff => {
        events.push(diff);
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
    
    model.onDiffs(diff => {
        events.push(diff);
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
    
    model.onDiffs(diff => {
        events.push(diff);
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
    
    model.onDiffs(diff => {
        events.push(diff);
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
    const unregister1 = model.onDiffs(diff => {
        events1.push(diff);
    });
    
    const unregister2 = model.onDiffs(diff => {
        events2.push(diff);
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