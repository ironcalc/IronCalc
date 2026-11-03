import test from 'node:test';
import assert from 'node:assert';
import { Model } from '../pkg-nodejs/wasm.js';

const font = { size: 9, family: null, bold: false, italic: false, color: null };
const line = { style: 'Automatic', color: null };
const noLine = { style: 'None', color: null };

function chart(title, stacking = 'None') {
    return {
        type: 'ColumnChart',
        title: title === null ? null : { text: title, font, alignment: 'Middle', fill: null, border: line, subtitle: null },
        legend: {
            position: 'Bottom',
            vertical: false,
            padding: null,
            fill: null,
            border: line,
            font,
            overlay: false,
        },
        stacking,
        category_axis: {
            title: null,
            labels: font,
            rotation: null,
            line,
            gridlines: { major: noLine, minor: noLine, major_ticks: 'None', minor_ticks: 'None' },
        },
        value_axis: {
            title: null,
            labels: { number_format: null, font },
            line,
            gridlines: { major: line, minor: noLine, major_ticks: 'None', minor_ticks: 'None' },
        },
        data: {
            range: { type: 'Range', range: 'Sheet1!$A$1:$C$5', orientation: 'Columns' },
            empty_cells: 'Gaps',
            na_as_empty: false,
            show_hidden_data: false,
        },
        colors: ['#FF0000', [5, 0.4]],
        data_labels: [],
        bars: { gap: 219, overlap: -27 },
    };
}

function anchor(row, column) {
    const point = (r, c) => ({ row: r, column: c, row_offset: 0, column_offset: 0 });
    return { from: point(row, column), to: point(row + 15, column + 7), behavior: 'MoveAndResize' };
}

test('charts: empty list initially', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    assert.deepEqual(Array.from(model.getChartIds(0)), []);
});

test('charts: new chart round-trips through getChart', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    const id = model.newChart(0, anchor(2, 2), chart('Sales'));
    assert.deepEqual(Array.from(model.getChartIds(0)), [id]);
    assert.deepEqual(model.getChart(0, id), chart('Sales'));
});

test('charts: update, delete and undo', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    const id = model.newChart(0, anchor(2, 2), chart('Before'));
    model.updateChart(0, id, chart('After', 'Standard'));
    assert.strictEqual(model.getChart(0, id).title.text, 'After');
    assert.strictEqual(model.getChart(0, id).stacking, 'Standard');
    model.undo();
    assert.strictEqual(model.getChart(0, id).title.text, 'Before');

    model.deleteChart(0, id);
    assert.deepEqual(Array.from(model.getChartIds(0)), []);
    model.undo();
    assert.deepEqual(Array.from(model.getChartIds(0)), [id]);
    model.redo();
    assert.deepEqual(Array.from(model.getChartIds(0)), []);
});

test('charts: move and style the drawing', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    const id = model.newChart(0, anchor(2, 2), chart(null));
    model.moveDrawing(0, id, anchor(10, 4));
    model.setDrawingStyle(0, id, { Solid: [5, 0.6] }, { style: 'Dashed', color: '#FF0000' });
    assert.deepEqual(model.getDrawingStyle(0, id), {
        fill: { Solid: [5, 0.6] },
        border: { style: 'Dashed', color: '#FF0000' },
    });
    model.setDrawingStyle(0, id, 'None', { style: 'None', color: null });
    assert.deepEqual(model.getDrawingStyle(0, id), { fill: 'None', border: { style: 'None', color: null } });
    model.undo();
    model.undo();
    model.undo();
    assert.deepEqual(Array.from(model.getChartIds(0)), [id]);
    assert.strictEqual(model.getChart(0, id).title, null);
});

test('charts: errors are thrown', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    assert.throws(() => model.getChart(0, 99));
    assert.throws(() => model.deleteChart(0, 99));
    assert.throws(() => model.getChartIds(5));
    assert.throws(() => model.newChart(0, anchor(1, 1), { type: 'PieChart' }));
});

test('charts: anchor and svg', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.setUserInput(0, 1, 1, 'City');
    model.setUserInput(0, 1, 2, 'Visitors');
    model.setUserInput(0, 2, 1, 'London');
    model.setUserInput(0, 2, 2, '123');
    model.setUserInput(0, 3, 1, 'Madrid');
    model.setUserInput(0, 3, 2, '456');
    const id = model.newChart(0, anchor(5, 1), chart('Visitors'));
    assert.deepEqual(model.getDrawingAnchor(0, id), anchor(5, 1));
    const svg = model.getChartSvg(0, id, 480, 288);
    assert.ok(svg.startsWith('<svg xmlns="http://www.w3.org/2000/svg" width="480" height="288"'));
    assert.ok(svg.includes('>Visitors</text>'));
    assert.ok(svg.includes('>Madrid</text>'));
    assert.ok(svg.includes('class="series-0"'));
    // A chart whose data cannot be resolved is still drawn
    const broken = chart('Broken');
    broken.data.range = { type: 'Range', range: 'Nowhere!$A$1:$B$3', orientation: 'Columns' };
    const other = model.newChart(0, anchor(30, 1), broken);
    assert.ok(model.getChartSvg(0, other, 300, 200).includes('>Broken</text>'));
    assert.throws(() => model.getChartSvg(0, 99, 300, 200));
});

test('charts: data and setup for the editor', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.setUserInput(0, 1, 2, 'Visitors');
    model.setUserInput(0, 2, 1, 'London');
    model.setUserInput(0, 2, 2, '123');
    model.setUserInput(0, 3, 1, 'Madrid');
    const range = chart('Visitors');
    range.data.range = { type: 'Range', range: 'Sheet1!$A$1:$B$3', orientation: 'Columns' };
    const id = model.newChart(0, anchor(5, 1), range);
    assert.deepEqual(model.getChartData(0, id), {
        categories: ['London', 'Madrid'],
        series: [{ name: 'Visitors', values: [123, null], number_format: 'general' }],
    });
    assert.deepEqual(model.getChartSetup(0, id), {
        categories: 'Sheet1!$A$2:$A$3',
        series: [{ name: 'Sheet1!$B$1', values: 'Sheet1!$B$2:$B$3', label: 'Visitors' }],
    });
    assert.throws(() => model.getChartData(0, 99));
});
