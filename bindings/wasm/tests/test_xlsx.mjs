import test from 'node:test';
import assert from 'node:assert';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { Model } from '../pkg-nodejs-xlsx/wasm.js';

const exampleXlsx = readFileSync(
    path.join(
        path.dirname(fileURLToPath(import.meta.url)),
        '..',
        '..',
        '..',
        'xlsx',
        'tests',
        'example.xlsx',
    ),
);

test('fromXlsx loads example.xlsx', () => {
    const model = Model.fromXlsx(exampleXlsx, 'example', 'en', 'UTC', 'en');
    const sheets = model.getWorksheetsProperties();
    const names = sheets.map((sheet) => sheet.name);
    assert.deepEqual(names, [
        'Sheet1',
        'Second',
        'Sheet4',
        'shared',
        'Table',
        'Sheet2',
        'Created fourth',
        'Frozen',
        'Split',
        'Hidden',
    ]);
});

test('toXlsx round-trips a workbook', () => {
    const model = Model.fromXlsx(exampleXlsx, 'example', 'en', 'UTC', 'en');
    const bytes = model.toXlsx();
    assert.ok(bytes.length > 0);

    const roundtrip = Model.fromXlsx(bytes, 'example', 'en', 'UTC', 'en');
    const names = roundtrip.getWorksheetsProperties().map((sheet) => sheet.name);
    assert.deepEqual(names, model.getWorksheetsProperties().map((sheet) => sheet.name));
});

test('engine APIs still work on the xlsx build', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.setUserInput(0, 1, 1, '23');
    model.setUserInput(0, 1, 2, '=A1*3+1');
    assert.strictEqual(model.getFormattedCellValue(0, 1, 2), '70');

    const bytes = model.toXlsx();
    const loaded = Model.fromXlsx(bytes, 'Workbook1', 'en', 'UTC', 'en');
    assert.strictEqual(loaded.getFormattedCellValue(0, 1, 2), '70');
});
