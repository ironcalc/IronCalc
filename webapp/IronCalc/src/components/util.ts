import type { Area, Cell } from "./types";

import { type SelectedView, columnNameFromNumber } from "@ironcalc/wasm";
import { LAST_COLUMN, LAST_ROW } from "./WorksheetCanvas/constants";

// FIXME: Use the `quoteName` function from the wasm module
function nameNeedsQuoting(name: string): boolean {
  // it contains any of these characters: ()'$,;-+{} or space
  for (const char of name) {
    if (" ()'$,;-+{}".includes(char)) {
      return true;
    }
  }

  // TODO:
  // - cell reference in A1 notation, e.g. B1048576 is quoted, B1048577 is not
  // - cell reference in R1C1 notation, e.g. RC, RC2, R5C, R-4C, RC-8, R, C
  // - integers

  return false;
}

/**
 * Quotes a string sheet name if it needs to
 * NOTE: Invalid characters in a sheet name: \, /, *, [, ], :, ?
 */
export function quoteName(name: string): string {
  if (nameNeedsQuoting(name)) {
    return `'${name.replace(/'/g, "''")}'`;
  }
  return name;
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

export const getCellAddress = (selectedArea: Area, selectedCell: Cell) => {
  const isSingleCell =
    selectedArea.rowStart === selectedArea.rowEnd &&
    selectedArea.columnEnd === selectedArea.columnStart;

  if (isSingleCell) {
    return `${columnNameFromNumber(selectedCell.column)}${selectedCell.row}`;
  }
  if (selectedArea.rowStart === 1 && selectedArea.rowEnd === LAST_ROW) {
    return `${columnNameFromNumber(selectedArea.columnStart)}:${columnNameFromNumber(
      selectedArea.columnEnd,
    )}`;
  }
  if (
    selectedArea.columnStart === 1 &&
    selectedArea.columnEnd === LAST_COLUMN
  ) {
    return `${selectedArea.rowStart}:${selectedArea.rowEnd}`;
  }
  return `${columnNameFromNumber(selectedArea.columnStart)}${
    selectedArea.rowStart
  }:${columnNameFromNumber(selectedArea.columnEnd)}${selectedArea.rowEnd}`;
};

export function rangeToStr(
  range: {
    sheet: number;
    rowStart: number;
    rowEnd: number;
    columnStart: number;
    columnEnd: number;
  },
  referenceSheet: number,
  referenceName: string,
): string {
  const { sheet, rowStart, rowEnd, columnStart, columnEnd } = range;
  const sheetName =
    sheet === referenceSheet ? "" : `${quoteName(referenceName)}!`;
  if (rowStart === rowEnd && columnStart === columnEnd) {
    return `${sheetName}${columnNameFromNumber(columnStart)}${rowStart}`;
  }
  return `${sheetName}${columnNameFromNumber(
    columnStart,
  )}${rowStart}:${columnNameFromNumber(columnEnd)}${rowEnd}`;
}

// Returns the full range of the selected view as a string in absolute form
// e.g. 'Sheet1!$A$1:$B$2' or 'Sheet1!$A$1'
export function getFullRangeToString(
  selectedView: SelectedView,
  worksheetNames: string[],
): string {
  const [rowStart, columnStart, rowEnd, columnEnd] = selectedView.range;
  const sheetName = quoteName(worksheetNames[selectedView.sheet]);

  if (rowStart === rowEnd && columnStart === columnEnd) {
    return `${sheetName}!$${columnNameFromNumber(columnStart)}$${rowStart}`;
  }
  return `${sheetName}!$${columnNameFromNumber(
    columnStart,
  )}$${rowStart}:$${columnNameFromNumber(columnEnd)}$${rowEnd}`;
}
