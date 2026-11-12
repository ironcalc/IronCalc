import test from 'node:test';
import assert from 'node:assert'
import { Model } from "../pkg-nodejs/wasm.js";

const DEFAULT_ROW_HEIGHT = 25;

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

test('Row and column sizes', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    assert.deepStrictEqual(model.getRowSizes(0), { default_size: 25, runs: [] });
    assert.deepStrictEqual(model.getColumnSizes(0), { default_size: 90, runs: [] });

    model.setRowsHeight(0, 3, 4, 40);
    model.setRowsHidden(0, 6, 6, true);
    model.setColumnsWidth(0, 2, 3, 120);
    model.setColumnsHidden(0, 5, 5, true);

    assert.deepStrictEqual(model.getRowSizes(0), {
        default_size: 25,
        runs: [
            { first: 3, last: 3, size: 40 },
            { first: 4, last: 4, size: 40 },
            { first: 6, last: 6, size: 0 },
        ],
    });
    const columns = model.getColumnSizes(0);
    assert.strictEqual(columns.default_size, 90);
    // The engine keeps one record per column
    assert.deepStrictEqual(columns.runs, [
        { first: 2, last: 2, size: 120 },
        { first: 3, last: 3, size: 120 },
        { first: 5, last: 5, size: 0 },
    ]);
    // The runs agree with the per-row and per-column queries
    for (let i = 1; i <= 8; i += 1) {
        const row = model.getRowSizes(0).runs.find((r) => r.first <= i && i <= r.last);
        assert.strictEqual(row ? row.size : 25, model.getRowHeight(0, i));
        const column = columns.runs.find((c) => c.first <= i && i <= c.last);
        assert.strictEqual(column ? column.size : 90, model.getColumnWidth(0, i));
    }

    assert.throws(() => model.getRowSizes(3));
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
    fill: { },
    font: { sz: 12, name: 'Inter', family: 2, scheme: 'minor' },
    border: {},
    quote_prefix: false,
};

test('Styles work', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
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
    model.setRowsHeight(0.8, 5.2, 5.5, 125);

    assert.strictEqual(model.getRowHeight(0.11, 5.99), 125);
    assert.strictEqual(model.getRowHeight(0, 5), 125);
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

test('Cell links', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    assert.strictEqual(model.getCellLink(0, 2, 2), undefined);

    model.setCellLink(0, 2, 2, { type: "External", target: "https://www.ironcalc.com/" });
    const link = model.getCellLink(0, 2, 2);
    assert.strictEqual(link.type, "External");
    assert.strictEqual(link.target, "https://www.ironcalc.com/");

    model.setCellLink(0, 5, 1, { type: "Internal", location: "Sheet1!A30", tooltip: "Jump!" });
    const links = model.getLinks(0);
    assert.strictEqual(links.length, 2);
    assert.strictEqual(links[0].row, 2);
    assert.strictEqual(links[0].column, 2);
    assert.strictEqual(links[0].target, "https://www.ironcalc.com/");
    assert.strictEqual(links[1].row, 5);
    assert.strictEqual(links[1].column, 1);
    assert.strictEqual(links[1].location, "Sheet1!A30");
    assert.strictEqual(links[1].tooltip, "Jump!");

    // links are undoable
    model.undo();
    model.undo();
    assert.strictEqual(model.getCellLink(0, 2, 2), undefined);
    model.redo();
    assert.strictEqual(model.getCellLink(0, 2, 2).target, "https://www.ironcalc.com/");

    model.deleteCellLink(0, 2, 2);
    assert.strictEqual(model.getCellLink(0, 2, 2), undefined);

    // invalid cell
    assert.throws(() => model.setCellLink(0, 0, 1, { type: "External", target: "https://x" }));
});

test('Cell link label and style are one undo step', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.setCellLink(0, 2, 2, { type: "External", target: "https://www.ironcalc.com/" }, "IronCalc");
    assert.strictEqual(model.getFormattedCellValue(0, 2, 2), "IronCalc");
    assert.strictEqual(model.getCellStyle(0, 2, 2).style.font.u, true);

    // one undo reverts the link, the content and the style together
    model.undo();
    assert.strictEqual(model.getCellLink(0, 2, 2), undefined);
    assert.strictEqual(model.getFormattedCellValue(0, 2, 2), "");
    assert.ok(!model.getCellStyle(0, 2, 2).style.font.u);
    assert.strictEqual(model.canUndo(), false);

    // one redo restores everything
    model.redo();
    assert.strictEqual(model.getFormattedCellValue(0, 2, 2), "IronCalc");
    assert.strictEqual(model.getCellStyle(0, 2, 2).style.font.u, true);
    assert.strictEqual(model.getCellLink(0, 2, 2).target, "https://www.ironcalc.com/");
});

test('Merged cells', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.setUserInput(0, 2, 2, "5");
    model.setUserInput(0, 3, 3, "hello");

    // merging a range where more than one cell has content fails
    assert.throws(
        () => model.mergeCells({ sheet: 0, row: 2, column: 2, width: 2, height: 2 }),
        /more than one cell has content/,
    );
    assert.deepEqual(model.getMergedCells(0), []);

    // with a single cell with content, its content moves to the anchor
    model.setUserInput(0, 2, 2, "");
    model.mergeCells({ sheet: 0, row: 2, column: 2, width: 2, height: 2 });
    assert.deepEqual(model.getMergedCells(0), [{ row: 2, column: 2, width: 2, height: 2 }]);
    assert.strictEqual(model.getFormattedCellValue(0, 2, 2), "hello");
    assert.strictEqual(model.getFormattedCellValue(0, 3, 3), "");
    // covered cells cannot be edited
    assert.throws(() => model.setUserInput(0, 3, 3, "42"));

    model.undo();
    assert.deepEqual(model.getMergedCells(0), []);
    assert.strictEqual(model.getFormattedCellValue(0, 2, 2), "");
    assert.strictEqual(model.getFormattedCellValue(0, 3, 3), "hello");

    model.redo();
    model.unmergeCells({ sheet: 0, row: 1, column: 1, width: 10, height: 10 });
    assert.deepEqual(model.getMergedCells(0), []);
    model.setUserInput(0, 3, 3, "42");
    assert.strictEqual(model.getFormattedCellValue(0, 3, 3), "42");
});

test('Merge cells variants', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');

    // merge & center: merging and centering are a single undo step
    model.setUserInput(0, 2, 2, "5");
    model.mergeCellsCenter({ sheet: 0, row: 2, column: 2, width: 2, height: 2 });
    assert.deepEqual(model.getMergedCells(0), [{ row: 2, column: 2, width: 2, height: 2 }]);
    assert.strictEqual(model.getCellStyle(0, 2, 2).style.alignment.horizontal, "center");
    model.undo();
    assert.deepEqual(model.getMergedCells(0), []);
    assert.strictEqual(model.getCellStyle(0, 2, 2).style.alignment, undefined);
    model.undo();

    // merge across: one merged cell per row, a single undo step
    model.mergeCellsAcross({ sheet: 0, row: 2, column: 2, width: 2, height: 3 });
    assert.deepEqual(model.getMergedCells(0), [
        { row: 2, column: 2, width: 2, height: 1 },
        { row: 3, column: 2, width: 2, height: 1 },
        { row: 4, column: 2, width: 2, height: 1 },
    ]);
    model.undo();
    assert.deepEqual(model.getMergedCells(0), []);

    // merge down: one merged cell per column, a single undo step
    model.mergeCellsDown({ sheet: 0, row: 2, column: 2, width: 2, height: 3 });
    assert.deepEqual(model.getMergedCells(0), [
        { row: 2, column: 2, width: 1, height: 3 },
        { row: 2, column: 3, width: 1, height: 3 },
    ]);
    model.undo();
    assert.deepEqual(model.getMergedCells(0), []);
});

test('Cells block', () => {
    let model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.setUserInput(0, 1, 1, 'hello');
    model.setUserInput(0, 2, 2, '=1+1');
    model.updateRangeStyle({ sheet: 0, row: 2, column: 1, width: 1, height: 1 }, 'fill.fg_color', '#FF0000');

    // A1:B2
    const block = model.getCellsBlock(0, 1, 1, 2, 2);
    assert.strictEqual(block.row, 1);
    assert.strictEqual(block.column, 1);
    assert.strictEqual(block.width, 2);
    assert.strictEqual(block.height, 2);
    // A1, B1, B2 are plain and share a style; A2 has its own
    assert.strictEqual(block.styles.length, 2);
    assert.strictEqual(block.style_indices.length, 4);
    assert.strictEqual(block.style_indices[0], block.style_indices[1]);
    assert.strictEqual(block.style_indices[1], block.style_indices[3]);
    assert.notStrictEqual(block.style_indices[2], block.style_indices[0]);
    const a2 = block.styles[block.style_indices[2]];
    assert.strictEqual(a2.style.fill.color, '#FF0000');
    assert.deepStrictEqual(a2, model.getCellStyle(0, 2, 1));
    assert.deepStrictEqual(block.texts, [
        { row: 1, column: 1, text: 'hello', cell_type: 2 },
        { row: 2, column: 2, text: '2', cell_type: 1 },
    ]);
    assert.deepStrictEqual(block.fills, [
        { row: 1, column: 1, width: 2, color: '' },
        { row: 2, column: 1, width: 1, color: '#FF0000' },
        { row: 2, column: 2, width: 1, color: '' },
    ]);

    assert.throws(() => model.getCellsBlock(0, 1, 1, 0, 1));
    assert.throws(() => model.getCellsBlock(3, 1, 1, 1, 1));
});

test('focus on open', () => {
    let model = new Model('Workbook1', 'en', 'UTC', 'en');
    assert.strictEqual(model.getFocusOnOpen(), undefined);
    model.newSheet();
    model.setFocusOnOpen(1, 5, 3);
    assert.deepStrictEqual(model.getFocusOnOpen(), { sheet: 1, row: 5, column: 3 });
    assert.throws(() => model.setFocusOnOpen(7, 1, 1));

    // The workbook opens on that cell
    model = Model.fromBytes(model.toBytes(), 'en');
    assert.deepStrictEqual(Array.from(model.getSelectedCell()), [1, 5, 3]);

    model.clearFocusOnOpen();
    assert.strictEqual(model.getFocusOnOpen(), undefined);
    model = Model.fromBytes(model.toBytes(), 'en');
    assert.deepStrictEqual(Array.from(model.getSelectedCell()), [0, 1, 1]);
});

test('Typed setters keep text as text', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    const texts = ["001", "=1+1", "3/4"];
    texts.forEach((text, index) => model.setCellText(0, index + 1, 1, text));
    model.evaluate();

    texts.forEach((text, index) => {
        assert.strictEqual(model.getCellType(0, index + 1, 1), 2);
        assert.strictEqual(model.getFormattedCellValue(0, index + 1, 1), text);
    });
});

test('Typed setters store numbers and booleans as they are', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    const numbers = [42, -3.5, 0.000001, 123456789.125, 1e20];
    numbers.forEach((number, index) => model.setCellNumber(0, index + 1, 1, number));
    model.setCellBoolean(0, 1, 2, true);
    model.setCellBoolean(0, 2, 2, false);
    model.setCellsBatch(0, [{ row: 3, column: 2, kind: "formula", value: "=AND(B1,NOT(B2))" }]);
    model.evaluate();

    numbers.forEach((number, index) => {
        assert.strictEqual(model.getCellType(0, index + 1, 1), 1);
        assert.strictEqual(Number(model.getCellContent(0, index + 1, 1)), number);
    });
    assert.strictEqual(model.getCellType(0, 1, 2), 4);
    assert.strictEqual(model.getFormattedCellValue(0, 1, 2), "TRUE");
    assert.strictEqual(model.getFormattedCellValue(0, 2, 2), "FALSE");
    assert.strictEqual(model.getFormattedCellValue(0, 3, 2), "TRUE");
});

test('Typed setters keep the style of the cell', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.updateRangeStyle({ sheet: 0, row: 1, column: 1, width: 1, height: 3 }, "font.b", "true");
    model.updateRangeStyle({ sheet: 0, row: 1, column: 1, width: 1, height: 1 }, "num_fmt", "0.00");
    model.setCellNumber(0, 1, 1, 3);
    model.setCellText(0, 2, 1, "001");
    model.setCellBoolean(0, 3, 1, true);
    model.evaluate();

    assert.strictEqual(model.getFormattedCellValue(0, 1, 1), "3.00");
    assert.strictEqual(model.getCellStyle(0, 1, 1).style.num_fmt, "0.00");
    for (let row = 1; row <= 3; row += 1) {
        assert.strictEqual(model.getCellStyle(0, row, 1).style.font.b, true);
    }
    // A text that would read as a number gets a quote prefix
    assert.strictEqual(model.getCellStyle(0, 2, 1).style.quote_prefix, true);
});

test('setCellsBatch loads cells like setUserInput', () => {
    const cells = [];
    for (let row = 1; row <= 2000; row += 1) {
        cells.push({ row, column: 1, kind: "number", value: row * 1.5 });
        cells.push({ row, column: 2, kind: "text", value: `item ${row}` });
        cells.push({ row, column: 3, kind: "boolean", value: row % 3 === 0 });
        cells.push({ row, column: 4, kind: "formula", value: `=IF(C${row},A${row}*2,LEN(B${row}))` });
        cells.push({ row, column: 5, kind: "input", value: row % 2 ? `${row}%` : `=SUM(A$1:A${row})` });
    }
    assert.strictEqual(cells.length, 10000);
    const batched = new Model('Workbook1', 'en', 'UTC', 'en');
    batched.setCellsBatch(0, cells);
    batched.evaluate();

    // The same cells typed in
    const typed = new Model('Workbook1', 'en', 'UTC', 'en');
    typed.pauseEvaluation();
    for (const { row, column, kind, value } of cells) {
        const input = kind === "boolean" ? (value ? "TRUE" : "FALSE") : String(value);
        typed.setUserInput(0, row, column, input);
    }
    typed.evaluate();

    for (const { row, column } of cells) {
        assert.strictEqual(batched.getCellType(0, row, column), typed.getCellType(0, row, column));
        assert.strictEqual(
            batched.getFormattedCellValue(0, row, column),
            typed.getFormattedCellValue(0, row, column),
        );
    }
    assert.strictEqual(batched.getFormattedCellValue(0, 4, 5), "15");
    assert.strictEqual(batched.canUndo(), false);
});

test('setCellsBatch stops at the first bad cell', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    assert.throws(() => model.setCellsBatch(0, [
        { row: 1, column: 1, kind: "number", value: 1 },
        { row: 2, column: 1, kind: "text", value: "two" },
        { row: 3, column: 1, kind: "number", value: "three" },
        { row: 4, column: 1, kind: "number", value: 4 },
    ]), { name: 'Error', message: /^Cell 2: / });
    model.evaluate();
    // The cells before the bad one are set, the others are not
    assert.strictEqual(model.getFormattedCellValue(0, 1, 1), "1");
    assert.strictEqual(model.getFormattedCellValue(0, 2, 1), "two");
    assert.strictEqual(model.getFormattedCellValue(0, 3, 1), "");
    assert.strictEqual(model.getFormattedCellValue(0, 4, 1), "");

    // A cell the engine refuses
    assert.throws(() => model.setCellsBatch(0, [
        { row: 5, column: 1, kind: "text", value: "five" },
        { row: 0, column: 1, kind: "text", value: "zero" },
    ]), { name: 'Error', message: "Cell 1: Invalid row" });
    assert.strictEqual(model.getFormattedCellValue(0, 5, 1), "five");
});

test('getCellValue gives values unformatted', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    model.setCellNumber(0, 1, 1, 3.14159);
    model.updateRangeStyle({ sheet: 0, row: 1, column: 1, width: 1, height: 1 }, "num_fmt", "0.00");
    model.setCellText(0, 2, 1, "001");
    model.setCellBoolean(0, 3, 1, false);
    model.setCellFormula(0, 4, 1, "=VLOOKUP(9,A1:A3,1,FALSE)");
    model.setCellFormula(0, 5, 1, "=A1*2");
    model.setCellFormula(0, 1, 2, "=SEQUENCE(3)");
    model.evaluate();

    assert.strictEqual(model.getFormattedCellValue(0, 1, 1), "3.14");
    assert.deepStrictEqual(model.getCellValue(0, 1, 1), { type: "number", value: 3.14159 });
    assert.deepStrictEqual(model.getCellValue(0, 2, 1), { type: "text", value: "001" });
    assert.deepStrictEqual(model.getCellValue(0, 3, 1), { type: "boolean", value: false });
    assert.deepStrictEqual(model.getCellValue(0, 4, 1), { type: "error", value: "#N/A" });
    assert.deepStrictEqual(model.getCellValue(0, 5, 1), { type: "number", value: 6.28318 });
    assert.deepStrictEqual(model.getCellValue(0, 9, 9), { type: "empty" });
    // A cell the formula spills into
    assert.deepStrictEqual(model.getCellValue(0, 3, 2), { type: "number", value: 3 });

    assert.deepStrictEqual(model.getCellValuesBlock(0, 2, 1, 2, 2), [
        [{ type: "text", value: "001" }, { type: "number", value: 2 }],
        [{ type: "boolean", value: false }, { type: "number", value: 3 }],
    ]);
    assert.throws(() => model.getCellValuesBlock(0, 1, 1, 0, 1));
});

test('setCellFormula and getCellFormula use English whatever the locale', () => {
    const model = new Model('Workbook1', 'de', 'UTC', 'de');
    model.setCellNumber(0, 1, 1, 1.5);
    model.setCellNumber(0, 2, 1, 2.25);
    model.setCellFormula(0, 1, 2, '=SUM(A1,A2,0.25)');
    model.setCellsBatch(0, [{ row: 2, column: 2, kind: "formula", value: '=IF(A1>1,"big","small")' }]);
    model.evaluate();

    assert.deepStrictEqual(model.getCellValue(0, 1, 2), { type: "number", value: 4 });
    assert.deepStrictEqual(model.getCellValue(0, 2, 2), { type: "text", value: "big" });
    assert.strictEqual(model.getCellFormula(0, 1, 2), '=SUM(A1,A2,0.25)');
    assert.strictEqual(model.getCellFormula(0, 2, 2), '=IF(A1>1,"big","small")');
    assert.strictEqual(model.getCellFormula(0, 1, 1), undefined);
    // What the user sees and types is localised
    assert.strictEqual(model.getCellContent(0, 1, 2), '=SUMME(A1;A2;0,25)');
    model.setUserInput(0, 3, 2, '=SUMME(A1;A2;0,25)');
    assert.deepStrictEqual(model.getCellValue(0, 3, 2), { type: "number", value: 4 });
});

test('evaluateFormula evaluates in the context of a cell without storing', () => {
    const model = new Model('Workbook1', 'en', 'UTC', 'en');
    [4, 7, 7, 9].forEach((value, index) => model.setCellNumber(0, index + 1, 2, value));
    model.evaluate();

    // Relative references are relative to the cell
    assert.deepStrictEqual(
        model.evaluateFormula(0, 2, 2, '=AND(B2>0,COUNTIF($B$1:$B$4,B2)=1)'),
        { type: "boolean", value: false },
    );
    assert.deepStrictEqual(
        model.evaluateFormula(0, 4, 2, 'AND(B4>0,COUNTIF($B$1:$B$4,B4)=1)'),
        { type: "boolean", value: true },
    );
    assert.deepStrictEqual(model.evaluateFormula(0, 1, 1, '=1/0'), { type: "error", value: "#DIV/0!" });
    // An array or a range gives its top-left value
    assert.deepStrictEqual(model.evaluateFormula(0, 1, 1, '=B2:B4*10'), { type: "number", value: 70 });
    assert.throws(() => model.evaluateFormula(0, 1, 1, '=SUM(1,'));

    // Nothing was stored
    assert.deepStrictEqual(model.getCellValue(0, 2, 2), { type: "number", value: 7 });
    assert.deepStrictEqual(model.getCellValue(0, 1, 1), { type: "empty" });
    assert.strictEqual(model.canUndo(), false);
});

test('TODAY, NOW and RAND read the clock and the RNG of JavaScript on every evaluation', () => {
    const { now, random } = { now: Date.now, random: Math.random };
    try {
        const model = new Model('Workbook1', 'en', 'UTC', 'en');
        model.setCellFormula(0, 1, 1, '=TODAY()');
        model.setCellFormula(0, 2, 1, '=NOW()');
        model.setCellFormula(0, 3, 1, '=RAND()');
        model.setCellFormula(0, 4, 1, '=RANDBETWEEN(1,100)');
        model.setCellFormula(0, 5, 1, '=RANDARRAY(1,2)');

        Date.now = () => Date.UTC(2026, 0, 15, 12, 0, 0);
        Math.random = () => 0.25;
        model.evaluate();
        assert.deepStrictEqual(model.getCellValuesBlock(0, 1, 1, 2, 5).map((row) => row.map((v) => v.value)), [
            [46037, undefined], [46037.5, undefined], [0.25, undefined], [26, undefined], [0.25, 0.25],
        ]);

        Date.now = () => Date.UTC(2026, 0, 16, 18, 0, 0);
        Math.random = () => 0.75;
        model.evaluate();
        assert.deepStrictEqual(model.getCellValuesBlock(0, 1, 1, 2, 5).map((row) => row.map((v) => v.value)), [
            [46038, undefined], [46038.75, undefined], [0.75, undefined], [76, undefined], [0.75, 0.75],
        ]);
    } finally {
        Date.now = now;
        Math.random = random;
    }
});
