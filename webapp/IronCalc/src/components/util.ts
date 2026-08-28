import {
  columnNameFromNumber,
  type MergedCell,
  type Model,
  quoteName,
  type SelectedView,
} from "@ironcalc/wasm";
import type { Area, Cell } from "./types";
import { LAST_COLUMN, LAST_ROW } from "./WorksheetCanvas/constants";

/**
 * Returns the merged cell containing (row, column), or undefined if the cell
 * is not part of any merge.
 */
export function mergedCellContaining(
  mergedCells: MergedCell[],
  row: number,
  column: number,
): MergedCell | undefined {
  return mergedCells.find(
    (m) =>
      row >= m.row &&
      row < m.row + m.height &&
      column >= m.column &&
      column < m.column + m.width,
  );
}

/**
 * Grows the range until it fully contains every merged cell it touches
 * (growing to swallow one merge can graze another, hence the fixpoint loop).
 * The orientation of the range is preserved. Mirrors the engine's
 * grow_range_over_merged_cells.
 */
export function growRangeOverMergedCells(
  mergedCells: MergedCell[],
  range: Area,
): Area {
  let minRow = Math.min(range.rowStart, range.rowEnd);
  let maxRow = Math.max(range.rowStart, range.rowEnd);
  let minColumn = Math.min(range.columnStart, range.columnEnd);
  let maxColumn = Math.max(range.columnStart, range.columnEnd);
  let changed = true;
  while (changed) {
    changed = false;
    for (const m of mergedCells) {
      const lastRow = m.row + m.height - 1;
      const lastColumn = m.column + m.width - 1;
      const intersects =
        m.row <= maxRow &&
        lastRow >= minRow &&
        m.column <= maxColumn &&
        lastColumn >= minColumn;
      if (intersects) {
        if (m.row < minRow) {
          minRow = m.row;
          changed = true;
        }
        if (lastRow > maxRow) {
          maxRow = lastRow;
          changed = true;
        }
        if (m.column < minColumn) {
          minColumn = m.column;
          changed = true;
        }
        if (lastColumn > maxColumn) {
          maxColumn = lastColumn;
          changed = true;
        }
      }
    }
  }
  const rowsAscending = range.rowStart <= range.rowEnd;
  const columnsAscending = range.columnStart <= range.columnEnd;
  return {
    rowStart: rowsAscending ? minRow : maxRow,
    rowEnd: rowsAscending ? maxRow : minRow,
    columnStart: columnsAscending ? minColumn : maxColumn,
    columnEnd: columnsAscending ? maxColumn : minColumn,
  };
}

export type FillDirection =
  | "rowsDown"
  | "rowsUp"
  | "columnsRight"
  | "columnsLeft";

/**
 * Auto-filling tiles the merged cells of the selection into the fill target
 * with period the selection height (filling by rows) or width (by columns),
 * and the engine rejects a fill whose boundary would cut a tiled merge.
 * Returns the largest extent <= `extent` whose boundary cuts no merge
 * (possibly 0). The selection bounds must be normalized and, per the
 * selection invariant, the selection fully contains every merge it touches.
 */
export function snapFillExtent(
  mergedCells: MergedCell[],
  selection: {
    rowStart: number;
    rowEnd: number;
    columnStart: number;
    columnEnd: number;
  },
  direction: FillDirection,
  extent: number,
): number {
  if (extent <= 0) {
    return 0;
  }
  const byRows = direction === "rowsDown" || direction === "rowsUp";
  const period = byRows
    ? selection.rowEnd - selection.rowStart + 1
    : selection.columnEnd - selection.columnStart + 1;
  // the merge spans along the fill axis, as 0-based offsets in the selection
  const spans: [number, number][] = [];
  for (const m of mergedCells) {
    const lastRow = m.row + m.height - 1;
    const lastColumn = m.column + m.width - 1;
    const intersects =
      m.row <= selection.rowEnd &&
      lastRow >= selection.rowStart &&
      m.column <= selection.columnEnd &&
      lastColumn >= selection.columnStart;
    if (!intersects) {
      continue;
    }
    spans.push(
      byRows
        ? [m.row - selection.rowStart, lastRow - selection.rowStart]
        : [
            m.column - selection.columnStart,
            lastColumn - selection.columnStart,
          ],
    );
  }
  if (spans.length === 0) {
    return extent;
  }
  // With `filled` cells of the last tile filled, the fill boundary sits at
  // pattern offset `filled` (filling away from the selection) or
  // `period - filled` (filling towards the sheet start, where the last tile
  // holds the tail of the pattern); a merge must not straddle it.
  const towardStart = direction === "rowsUp" || direction === "columnsLeft";
  const cutsAMerge = (filled: number): boolean => {
    const boundary = towardStart ? period - filled : filled;
    return spans.some(([start, end]) => start < boundary && end >= boundary);
  };
  let snapped = extent;
  while (
    snapped > 0 &&
    snapped % period !== 0 &&
    cutsAMerge(snapped % period)
  ) {
    snapped -= 1;
  }
  return snapped;
}

/**
 * Returns the size of the cell editor for a cell: the size of the cell itself,
 * or of the whole merged range if the cell is merged.
 */
export function getEditorSize(
  model: Model,
  sheet: number,
  row: number,
  column: number,
): { width: number; height: number } {
  const merge = mergedCellContaining(model.getMergedCells(sheet), row, column);
  if (!merge) {
    return {
      width: model.getColumnWidth(sheet, column),
      height: model.getRowHeight(sheet, row),
    };
  }
  let width = 0;
  for (let c = merge.column; c < merge.column + merge.width; c += 1) {
    width += model.getColumnWidth(sheet, c);
  }
  let height = 0;
  for (let r = merge.row; r < merge.row + merge.height; r += 1) {
    height += model.getRowHeight(sheet, r);
  }
  return { width, height };
}

/**
 *  Returns true if the keypress should start editing
 */
export function isEditingKey(key: string): boolean {
  if (key.length !== 1) {
    return false;
  }
  const code = key.codePointAt(0) ?? 0;
  if (code > 0 && code < 255) {
    return true;
  }
  return false;
}

export type NavigationKey =
  | "ArrowRight"
  | "ArrowLeft"
  | "ArrowDown"
  | "ArrowUp"
  | "Home"
  | "End";

export const isNavigationKey = (key: string): key is NavigationKey =>
  ["ArrowRight", "ArrowLeft", "ArrowDown", "ArrowUp", "Home", "End"].includes(
    key,
  );

interface FormatRangeOptions {
  // Use absolute references like '$A$1'
  absolute?: boolean;
  // Already-quoted sheet name; when set, prefixes the reference with 'Name!'
  sheetName?: string;
}

const cellToString = (
  row: number,
  column: number,
  absolute: boolean,
): string =>
  absolute
    ? `$${columnNameFromNumber(column)}$${row}`
    : `${columnNameFromNumber(column)}${row}`;

/**
 * Formats a (possibly unordered) range as an A1-style reference,
 * collapsing 1x1 ranges to a single cell.
 */
function formatRange(
  range: Area,
  { absolute = false, sheetName = "" }: FormatRangeOptions = {},
): string {
  const rowMin = Math.min(range.rowStart, range.rowEnd);
  const rowMax = Math.max(range.rowStart, range.rowEnd);
  const columnMin = Math.min(range.columnStart, range.columnEnd);
  const columnMax = Math.max(range.columnStart, range.columnEnd);
  const prefix = sheetName ? `${sheetName}!` : "";
  const start = cellToString(rowMin, columnMin, absolute);
  if (rowMin === rowMax && columnMin === columnMax) {
    return `${prefix}${start}`;
  }
  return `${prefix}${start}:${cellToString(rowMax, columnMax, absolute)}`;
}

// Returns the selection as shown in the name box,
// using 'A:B' / '1:3' shorthand for full columns/rows
export const getCellAddress = (
  selectedArea: Area,
  selectedCell: Cell,
): string => {
  const { rowStart, rowEnd, columnStart, columnEnd } = selectedArea;
  if (rowStart === rowEnd && columnStart === columnEnd) {
    return cellToString(selectedCell.row, selectedCell.column, false);
  }
  const rowMin = Math.min(rowStart, rowEnd);
  const rowMax = Math.max(rowStart, rowEnd);
  const columnMin = Math.min(columnStart, columnEnd);
  const columnMax = Math.max(columnStart, columnEnd);
  if (rowMin === 1 && rowMax === LAST_ROW) {
    return `${columnNameFromNumber(columnMin)}:${columnNameFromNumber(columnMax)}`;
  }
  if (columnMin === 1 && columnMax === LAST_COLUMN) {
    return `${rowMin}:${rowMax}`;
  }
  return formatRange(selectedArea);
};

// Returns the range as a formula reference relative to referenceSheet,
// e.g. 'A1:B2' on the same sheet or 'Sheet2!A1:B2' on another
export function rangeToStr(
  range: Area & { sheet: number },
  referenceSheet: number,
  referenceName: string,
): string {
  const sheetName =
    range.sheet === referenceSheet ? "" : quoteName(referenceName);
  return formatRange(range, { sheetName });
}

// Returns the full range of the selected view as a string in absolute form
// e.g. 'Sheet1!$A$1:$B$2' or 'Sheet1!$A$1'
export function getFullRangeToString(
  selectedView: SelectedView,
  worksheetNames: string[],
): string {
  const [rowStart, columnStart, rowEnd, columnEnd] = selectedView.range;
  return formatRange(
    { rowStart, rowEnd, columnStart, columnEnd },
    {
      absolute: true,
      sheetName: quoteName(worksheetNames[selectedView.sheet]),
    },
  );
}

/**
 * Returns all focusable elements inside a container in DOM order.
 * Used for keyboard navigation (Tab/arrow keys) and focus management.
 */
export function getFocusableElements(root: HTMLElement | null): HTMLElement[] {
  if (!root) {
    return [];
  }

  return Array.from(
    root.querySelectorAll<HTMLElement>(
      'button, input, [href], [tabindex]:not([tabindex="-1"])',
    ),
  ).filter(
    (el) =>
      !el.hasAttribute("disabled") &&
      el.getAttribute("aria-hidden") !== "true" &&
      el.tabIndex !== -1,
  );
}
