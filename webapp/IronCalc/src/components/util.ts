import {
  columnNameFromNumber,
  quoteName,
  type SelectedView,
} from "@ironcalc/wasm";
import type { Area, Cell } from "./types";
import { LAST_COLUMN, LAST_ROW } from "./WorksheetCanvas/constants";

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
