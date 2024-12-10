import test from 'node:test';
import assert from 'node:assert';
import { Model } from '../pkg/wasm.js';

const COLOR_SCALE = {
    type: 'ColorScale',
    thresholds: [
        { cfvo: 'Min', color: '#FF0000' },
        { cfvo: 'Max', color: '#00FF00' },
    ],
};
const DATA_BAR = {
    type: 'DataBar',
    min: 'Min',
    max: 'Max',
    positive_color: '#0000FF',
    negative_color: '#FF0000',
    is_gradient: true,
    show_value: true,
};

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
    // A1 is the minimum value → gets the min color
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

test('CF: CellIs GreaterThan applies fill color', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    for (let i = 1; i <= 5; i++) model.setUserInput(0, i, 1, String(i));
    const rule = {
        type: 'CellIs',
        operator: 'GreaterThan',
        formula: '3',
        formula2: null,
        format: { fill: { pattern_type: 'solid', bg_color: '#FF0000' } },
    };
    model.addConditionalFormatting(0, 'A1:A5', rule);
    // A4=4 > 3 → red fill
    assert.strictEqual(model.getCellStyle(0, 4, 1).style.fill.bg_color, '#FF0000');
    // A1=1, not > 3 → no CF fill
    const a1Fill = model.getCellStyle(0, 1, 1).style.fill.bg_color;
    assert.ok(a1Fill === undefined || a1Fill === null || a1Fill === '');
});

test('CF: CellIs Between applies fill to cells in range', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    for (let i = 1; i <= 5; i++) model.setUserInput(0, i, 1, String(i));
    const rule = {
        type: 'CellIs',
        operator: 'Between',
        formula: '2',
        formula2: '4',
        format: { fill: { pattern_type: 'solid', bg_color: '#FF0000' } },
    };
    model.addConditionalFormatting(0, 'A1:A5', rule);
    // A2=2, A3=3, A4=4 are between 2 and 4 → red fill
    assert.strictEqual(model.getCellStyle(0, 2, 1).style.fill.bg_color, '#FF0000');
    assert.strictEqual(model.getCellStyle(0, 3, 1).style.fill.bg_color, '#FF0000');
    assert.strictEqual(model.getCellStyle(0, 4, 1).style.fill.bg_color, '#FF0000');
    // A1=1, A5=5 are outside the range → no fill
    const a1Fill = model.getCellStyle(0, 1, 1).style.fill.bg_color;
    const a5Fill = model.getCellStyle(0, 5, 1).style.fill.bg_color;
    assert.ok(a1Fill === undefined || a1Fill === null || a1Fill === '');
    assert.ok(a5Fill === undefined || a5Fill === null || a5Fill === '');
});

test('CF: CellIs Equal applies fill only to matching cell', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    for (let i = 1; i <= 3; i++) model.setUserInput(0, i, 1, String(i));
    const rule = {
        type: 'CellIs',
        operator: 'Equal',
        formula: '2',
        formula2: null,
        format: { fill: { pattern_type: 'solid', bg_color: '#0000FF' } },
    };
    model.addConditionalFormatting(0, 'A1:A3', rule);
    // A2=2 matches → blue fill
    assert.strictEqual(model.getCellStyle(0, 2, 1).style.fill.bg_color, '#0000FF');
    // A1=1, A3=3 don't match → no fill
    const a1Fill = model.getCellStyle(0, 1, 1).style.fill.bg_color;
    const a3Fill = model.getCellStyle(0, 3, 1).style.fill.bg_color;
    assert.ok(a1Fill === undefined || a1Fill === null || a1Fill === '');
    assert.ok(a3Fill === undefined || a3Fill === null || a3Fill === '');
});

test('CF: Text Contains applies fill to matching cells', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.setUserInput(0, 1, 1, 'hello world');
    model.setUserInput(0, 2, 1, 'goodbye');
    model.setUserInput(0, 3, 1, 'hello');
    const rule = {
        type: 'Text',
        operator: 'Contains',
        value: 'hello',
        format: { fill: { pattern_type: 'solid', bg_color: '#00FF00' } },
    };
    model.addConditionalFormatting(0, 'A1:A3', rule);
    // A1 and A3 contain "hello" → green fill
    assert.strictEqual(model.getCellStyle(0, 1, 1).style.fill.bg_color, '#00FF00');
    assert.strictEqual(model.getCellStyle(0, 3, 1).style.fill.bg_color, '#00FF00');
    // A2 does not contain "hello" → no fill
    const a2Fill = model.getCellStyle(0, 2, 1).style.fill.bg_color;
    assert.ok(a2Fill === undefined || a2Fill === null || a2Fill === '');
});

test('CF: getDxfForConditionalFormatting returns format', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    const rule = {
        type: 'DuplicateValues',
        format: { fill: { pattern_type: 'solid', bg_color: '#AABBCC' } },
    };
    model.addConditionalFormatting(0, 'A1:A5', rule);
    const dxf = model.getDxfForConditionalFormatting(0, 0);
    assert.ok(dxf != null, 'dxf should be present');
    assert.strictEqual(dxf.fill.bg_color, '#AABBCC');
});

test('CF: getDxfForConditionalFormatting returns undefined for ColorScale', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.addConditionalFormatting(0, 'A1:A5', COLOR_SCALE);
    const dxf = model.getDxfForConditionalFormatting(0, 0);
    assert.strictEqual(dxf, undefined);
});
