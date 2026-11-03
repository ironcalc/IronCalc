import type {
  Anchor,
  Bars,
  Chart,
  Color,
  DataRange,
  Font,
  IronCalcTheme,
  Line,
  Model,
  Orientation,
  Series,
  Stacking,
  Title,
} from "@ironcalc/wasm";
import { LAST_COLUMN, LAST_ROW } from "../../WorksheetCanvas/constants";

// English Metric Units per CSS pixel at 96 dpi
const EMU_PER_PIXEL = 9525;

// Excel's default chart size
export const DEFAULT_CHART_WIDTH = 480;
export const DEFAULT_CHART_HEIGHT = 288;

const MAX_REGION_STEPS = 10_000;

/**
 * The block of non-empty cells around (row, column): grows a rectangle while
 * any cell on a new edge has content, as Excel does for a chart inserted
 * from a single cell. Returns [rowStart, columnStart, rowEnd, columnEnd].
 */
export function expandRegion(
  isEmpty: (row: number, column: number) => boolean,
  row: number,
  column: number,
  limits: { lastRow: number; lastColumn: number },
): [number, number, number, number] {
  let [r1, c1, r2, c2] = [row, column, row, column];
  const anyContentInRow = (r: number): boolean => {
    for (let c = c1; c <= c2; c += 1) {
      if (!isEmpty(r, c)) {
        return true;
      }
    }
    return false;
  };
  const anyContentInColumn = (c: number): boolean => {
    for (let r = r1; r <= r2; r += 1) {
      if (!isEmpty(r, c)) {
        return true;
      }
    }
    return false;
  };
  for (let step = 0; step < MAX_REGION_STEPS; step += 1) {
    let grown = false;
    if (r1 > 1 && anyContentInRow(r1 - 1)) {
      r1 -= 1;
      grown = true;
    }
    if (r2 < limits.lastRow && anyContentInRow(r2 + 1)) {
      r2 += 1;
      grown = true;
    }
    if (c1 > 1 && anyContentInColumn(c1 - 1)) {
      c1 -= 1;
      grown = true;
    }
    if (c2 < limits.lastColumn && anyContentInColumn(c2 + 1)) {
      c2 += 1;
      grown = true;
    }
    if (!grown) {
      break;
    }
  }
  return [r1, c1, r2, c2];
}

/**
 * An anchor whose top-left corner is the corner of (row, column) and that
 * spans `width` by `height` pixels, measured along the sheet's columns and
 * rows.
 */
export function chartAnchor(
  model: Model,
  sheet: number,
  row: number,
  column: number,
  width: number,
  height: number,
): Anchor {
  let toColumn = column;
  let remaining = width;
  for (;;) {
    const columnWidth = model.getColumnWidth(sheet, toColumn);
    if (remaining < columnWidth) {
      break;
    }
    remaining -= columnWidth;
    toColumn += 1;
  }
  let toRow = row;
  let remainingHeight = height;
  for (;;) {
    const rowHeight = model.getRowHeight(sheet, toRow);
    if (remainingHeight < rowHeight) {
      break;
    }
    remainingHeight -= rowHeight;
    toRow += 1;
  }
  return {
    from: { row, column, row_offset: 0, column_offset: 0 },
    to: {
      row: toRow,
      column: toColumn,
      row_offset: Math.round(remainingHeight * EMU_PER_PIXEL),
      column_offset: Math.round(remaining * EMU_PER_PIXEL),
    },
    behavior: "MoveAndResize",
  };
}

export function font(size: number): Font {
  return { size, family: null, bold: false, italic: false, color: null };
}

export const automaticLine: Line = { style: "Automatic", color: null };
export const noLine: Line = { style: "None", color: null };

/** Excel's placeholder title: what a chart gets when its title is switched on. */
export function defaultChartTitle(): Title {
  return {
    text: "Chart Title",
    font: font(14),
    alignment: "Middle",
    fill: null,
    border: automaticLine,
    subtitle: null,
  };
}

/** Excel's spacing for a new chart: stacked columns always overlap fully. */
export function defaultBars(stacking: Stacking): Bars {
  return stacking === "None"
    ? { gap: 219, overlap: -27 }
    : { gap: 150, overlap: 100 };
}

/** A column chart with Excel's defaults over `range`. */
export function defaultColumnChart(range: string, stacking: Stacking): Chart {
  return {
    type: "ColumnChart",
    title: defaultChartTitle(),
    legend: {
      position: "Bottom",
      vertical: false,
      padding: null,
      fill: null,
      border: automaticLine,
      font: font(9),
      overlay: false,
    },
    stacking,
    category_axis: {
      title: null,
      labels: font(9),
      rotation: null,
      line: automaticLine,
      gridlines: {
        major: noLine,
        minor: noLine,
        major_ticks: "None",
        minor_ticks: "None",
      },
    },
    value_axis: {
      title: null,
      labels: { number_format: null, font: font(9) },
      line: automaticLine,
      gridlines: {
        major: automaticLine,
        minor: noLine,
        major_ticks: "None",
        minor_ticks: "None",
      },
    },
    data: {
      range: { type: "Range", range, orientation: "Columns" },
      empty_cells: "Gaps",
      na_as_empty: false,
      show_hidden_data: false,
    },
    colors: [],
    data_labels: [],
    bars: defaultBars(stacking),
  };
}

// Column names without the engine, so this module needs no wasm to run
function columnNumberFromName(name: string): number {
  let number = 0;
  for (const letter of name.toUpperCase()) {
    number = number * 26 + (letter.charCodeAt(0) - 64);
  }
  return number;
}

function columnNameFromNumber(column: number): string {
  let name = "";
  let rest = column;
  while (rest > 0) {
    const digit = (rest - 1) % 26;
    name = String.fromCharCode(65 + digit) + name;
    rest = Math.floor((rest - 1) / 26);
  }
  return name;
}

const CELL_PATTERN = /^\$?([A-Za-z]{1,3})\$?(\d+)$/;

/** Parses `F3` or `$F$3` into a row and a column; null when it is not a cell. */
export function parseCell(
  text: string,
): { row: number; column: number } | null {
  const match = CELL_PATTERN.exec(text.trim());
  if (!match) {
    return null;
  }
  const column = columnNumberFromName(match[1]);
  const row = Number(match[2]);
  if (row < 1 || row > LAST_ROW || column < 1 || column > LAST_COLUMN) {
    return null;
  }
  return { row, column };
}

/** `F3` for row 3, column 6. */
export function formatCell(row: number, column: number): string {
  return `${columnNameFromNumber(column)}${row}`;
}

/** An anchor offset in EMU as whole pixels. */
export function emuToPixels(emu: number): number {
  return Math.round(emu / EMU_PER_PIXEL);
}

export function pixelsToEmu(pixels: number): number {
  return Math.round(pixels * EMU_PER_PIXEL);
}

export interface ParsedRange {
  sheet: string;
  rowStart: number;
  columnStart: number;
  rowEnd: number;
  columnEnd: number;
}

const RANGE_PATTERN =
  /^(?:\$?([A-Za-z]{1,3})\$?(\d+))(?::\$?([A-Za-z]{1,3})\$?(\d+))?$/;

/** Parses `Sheet1!$B$2:$B$5` or `'My Sheet'!B2`; null when it is not a range. */
export function parseRange(reference: string): ParsedRange | null {
  const text = reference.trim().replace(/^=/, "");
  const bang = text.lastIndexOf("!");
  if (bang === -1) {
    return null;
  }
  const sheet = text.slice(0, bang);
  const match = RANGE_PATTERN.exec(text.slice(bang + 1));
  if (!match) {
    return null;
  }
  const [, column1, row1, column2 = column1, row2 = row1] = match;
  const c1 = columnNumberFromName(column1);
  const c2 = columnNumberFromName(column2);
  const r1 = Number(row1);
  const r2 = Number(row2);
  if (c1 > LAST_COLUMN || c2 > LAST_COLUMN || r1 > LAST_ROW || r2 > LAST_ROW) {
    return null;
  }
  return {
    sheet,
    rowStart: Math.min(r1, r2),
    columnStart: Math.min(c1, c2),
    rowEnd: Math.max(r1, r2),
    columnEnd: Math.max(c1, c2),
  };
}

/** `Sheet1!$B$2:$B$5`, with the sheet part as given. */
export function formatAbsoluteRange(range: ParsedRange): string {
  const cell = (row: number, column: number) =>
    `$${columnNameFromNumber(column)}$${row}`;
  const first = cell(range.rowStart, range.columnStart);
  if (
    range.rowStart === range.rowEnd &&
    range.columnStart === range.columnEnd
  ) {
    return `${range.sheet}!${first}`;
  }
  return `${range.sheet}!${first}:${cell(range.rowEnd, range.columnEnd)}`;
}

/** Whether `text` is a reference (as opposed to a literal series name). */
export function isReference(text: string): boolean {
  return parseRange(text) !== null;
}

/**
 * A series name as the model stores it: a reference as is, anything else as
 * a quoted literal. The empty string stays empty (no name).
 */
export function seriesNameFormula(text: string): string {
  const trimmed = text.trim();
  if (trimmed === "" || isReference(trimmed)) {
    return trimmed.replace(/^=/, "");
  }
  return `"${trimmed.replace(/"/g, '""')}"`;
}

/** The text a user edits for a stored series name: literals lose their quotes. */
export function seriesNameText(formula: string): string {
  if (formula.length >= 2 && formula.startsWith('"') && formula.endsWith('"')) {
    return formula.slice(1, -1).replace(/""/g, '"');
  }
  return formula;
}

/**
 * The single range that holds a manual source, when the categories and every
 * series are columns (or rows) of one block on one sheet: the categories
 * first, the series next to them, the names on top (or at the left).
 * Null when they do not line up.
 */
export function manualToRange(
  categories: string,
  series: Series[],
): { range: string; orientation: Orientation } | null {
  const categoryRange = parseRange(categories);
  if (!categoryRange || series.length === 0) {
    return null;
  }
  const values = series.map((s) => parseRange(s.values));
  if (values.some((v) => v === null || v.sheet !== categoryRange.sheet)) {
    return null;
  }
  const ranges = values as ParsedRange[];
  const isColumn = (r: ParsedRange) => r.columnStart === r.columnEnd;
  const isRow = (r: ParsedRange) => r.rowStart === r.rowEnd;
  if (isColumn(categoryRange) && ranges.every(isColumn)) {
    const sameRows = ranges.every(
      (r) =>
        r.rowStart === categoryRange.rowStart &&
        r.rowEnd === categoryRange.rowEnd,
    );
    const contiguous = ranges.every(
      (r, index) => r.columnStart === categoryRange.columnStart + index + 1,
    );
    if (sameRows && contiguous && categoryRange.rowStart > 1) {
      return {
        range: formatAbsoluteRange({
          sheet: categoryRange.sheet,
          rowStart: categoryRange.rowStart - 1,
          columnStart: categoryRange.columnStart,
          rowEnd: categoryRange.rowEnd,
          columnEnd: categoryRange.columnStart + ranges.length,
        }),
        orientation: "Columns",
      };
    }
  }
  if (isRow(categoryRange) && ranges.every(isRow)) {
    const sameColumns = ranges.every(
      (r) =>
        r.columnStart === categoryRange.columnStart &&
        r.columnEnd === categoryRange.columnEnd,
    );
    const contiguous = ranges.every(
      (r, index) => r.rowStart === categoryRange.rowStart + index + 1,
    );
    if (sameColumns && contiguous && categoryRange.columnStart > 1) {
      return {
        range: formatAbsoluteRange({
          sheet: categoryRange.sheet,
          rowStart: categoryRange.rowStart,
          columnStart: categoryRange.columnStart - 1,
          rowEnd: categoryRange.rowStart + ranges.length,
          columnEnd: categoryRange.columnEnd,
        }),
        orientation: "Rows",
      };
    }
  }
  return null;
}

/** The data range of a chart as a manual source, given its expanded series. */
export function rangeToManual(
  categories: string,
  series: { name: string; values: string }[],
): DataRange {
  return {
    type: "Manual",
    categories,
    series: series.map(({ name, values }) => ({ name, values })),
  };
}

/** The color of a series without one: the theme accents in turn. */
export function automaticSeriesColor(
  index: number,
  theme: IronCalcTheme,
): string {
  const accents = [
    theme.accent1,
    theme.accent2,
    theme.accent3,
    theme.accent4,
    theme.accent5,
    theme.accent6,
  ];
  return accents[index % accents.length];
}

/** The model's `Color | null` as the color picker's `Color`. */
export function toPickerColor(color: Color | null | undefined): Color {
  return color ?? undefined;
}

/** The color picker's `Color` as the model stores it. */
export function fromPickerColor(color: Color): Color | null {
  return color === undefined || color === "" ? null : color;
}
