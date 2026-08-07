import type { Model } from "@ironcalc/wasm";
import { type KeyboardEvent, type RefObject, useCallback } from "react";
import {
  growRangeOverMergedCells,
  mergedCellContaining,
  rangeToStr,
} from "../util";
import { LAST_COLUMN, LAST_ROW } from "../WorksheetCanvas/constants";
import type { SheetRange, WorkbookState } from "../workbookState";
import { isInReferenceMode } from "./util";

interface Options {
  model: Model;
  onEditEnd: () => void;
  onTextUpdated: () => void;
  workbookState: WorkbookState;
  textareaRef: RefObject<HTMLTextAreaElement | null>;
}

function isValidRange(range: SheetRange): boolean {
  const { rowStart, rowEnd, columnStart, columnEnd } = range;
  if (rowStart < 1 || rowStart > LAST_ROW) {
    return false;
  }
  if (rowEnd < 1 || rowEnd > LAST_ROW) {
    return false;
  }
  if (columnStart < 1 || columnStart > LAST_COLUMN) {
    return false;
  }
  if (columnEnd < 1 || columnEnd > LAST_COLUMN) {
    return false;
  }

  return true;
}

export const useKeyDown = (
  options: Options,
): { onKeyDown: (event: KeyboardEvent) => void } => {
  const { model, onEditEnd, onTextUpdated, workbookState, textareaRef } =
    options;
  const onKeyDown = useCallback(
    (event: KeyboardEvent) => {
      const { key, shiftKey, altKey, ctrlKey } = event;
      const textarea = textareaRef.current;
      const cell = workbookState.getEditingCell();
      if (!textarea || !cell) {
        return;
      }
      switch (key) {
        case "Enter": {
          if (altKey) {
            // new line
            const start = textarea.selectionStart;
            const end = textarea.selectionEnd;
            const value = textarea.value;
            const newText = `${value.slice(0, start)}\n${value.slice(end)}`;
            cell.text = newText;
            workbookState.setEditingCell(cell);
            setTimeout(() => {
              textarea.setSelectionRange(start + 1, start + 1);
            }, 0);
            event.stopPropagation();
            event.preventDefault();
            onTextUpdated();
            return;
          }
          event.stopPropagation();
          event.preventDefault();
          if (ctrlKey && shiftKey) {
            const [rowStart, colStart, rowEnd, colEnd] =
              model.getSelectedView().range;
            const width = colEnd - colStart + 1;
            const height = rowEnd - rowStart + 1;
            model.setUserArrayFormula(
              cell.sheet,
              cell.row,
              cell.column,
              width,
              height,
              workbookState.getEditingText(),
            );
            model.onArrowDown();
          } else {
            // end edit and select cell bellow (or above if shiftKey)
            model.setUserInput(
              cell.sheet,
              cell.row,
              cell.column,
              workbookState.getEditingText(),
            );
            if (shiftKey) {
              model.onArrowUp();
            } else {
              model.onArrowDown();
            }
          }
          workbookState.clearEditingCell();
          onEditEnd();
          return;
        }
        case "Tab": {
          // end edit and select cell to the right (or left if ShiftKey)
          model.setUserInput(
            cell.sheet,
            cell.row,
            cell.column,
            workbookState.getEditingText(),
          );
          workbookState.clearEditingCell();
          if (shiftKey) {
            model.onArrowLeft();
          } else {
            model.onArrowRight();
          }
          if (textareaRef.current) {
            textareaRef.current.value = "";
          }
          event.stopPropagation();
          event.preventDefault();

          onEditEnd();
          return;
        }
        case "Escape": {
          // quit editing without modifying the cell
          const cell = workbookState.getEditingCell();
          if (cell) {
            model.setSelectedSheet(cell.sheet);
          }
          workbookState.clearEditingCell();
          onEditEnd();
          return;
        }
        // TODO: Arrow keys navigate in Excel
        case "ArrowRight": {
          if (cell.mode === "edit") {
            // just edit
            return;
          }
          event.stopPropagation();
          event.preventDefault();

          if (cell.referencedRange) {
            // There is already a reference range we move it to the right
            // (or expand if shift is pressed)
            const sheetNames = model
              .getWorksheetsProperties()
              .map((s) => s.name);
            const range = cell.referencedRange.range;
            const mergedCells = model.getMergedCells(range.sheet);
            let { anchorRow, anchorColumn } = cell.referencedRange;
            if (shiftKey) {
              const activeColumn =
                anchorColumn === range.columnStart
                  ? range.columnEnd
                  : range.columnStart;
              const newActiveColumn = activeColumn + 1;
              if (newActiveColumn > LAST_COLUMN) {
                return;
              }
              range.columnStart = Math.min(anchorColumn, newActiveColumn);
              range.columnEnd = Math.max(anchorColumn, newActiveColumn);
              // A reference can never cover part of a merged cell
              Object.assign(
                range,
                growRangeOverMergedCells(mergedCells, range),
              );
            } else {
              const activeRow =
                anchorRow === range.rowStart ? range.rowEnd : range.rowStart;
              const activeColumn =
                anchorColumn === range.columnStart
                  ? range.columnEnd
                  : range.columnStart;
              // Leaving a merged cell starts past its last column
              const leaving = mergedCellContaining(
                mergedCells,
                activeRow,
                activeColumn,
              );
              let column = leaving
                ? leaving.column + leaving.width
                : activeColumn + 1;
              let row = activeRow;
              if (column > LAST_COLUMN) {
                return;
              }
              // Landing inside a merged cell selects its anchor
              const landing = mergedCellContaining(mergedCells, row, column);
              if (landing) {
                row = landing.row;
                column = landing.column;
              }
              range.columnStart = column;
              range.columnEnd = column;
              range.rowStart = row;
              range.rowEnd = row;
              anchorRow = row;
              anchorColumn = column;
            }
            cell.referencedRange = {
              range,
              str: rangeToStr(range, cell.sheet, sheetNames[range.sheet]),
              anchorRow,
              anchorColumn,
            };
            workbookState.setEditingCell(cell);
            onTextUpdated();
            return;
          }
          if (isInReferenceMode(model, cell.text, cell.cursorStart)) {
            // there is not a referenced Range but we are in reference mode
            // we select the next cell
            const sheetNames = model
              .getWorksheetsProperties()
              .map((s) => s.name);
            const mergedCells = model.getMergedCells(cell.sheet);
            // Leaving a merged cell starts past its last column
            const leaving = mergedCellContaining(
              mergedCells,
              cell.row,
              cell.column,
            );
            const column = leaving
              ? leaving.column + leaving.width
              : cell.column + 1;
            const range = {
              sheet: cell.sheet,
              rowStart: cell.row,
              rowEnd: cell.row,
              columnStart: column,
              columnEnd: column,
            };
            if (!isValidRange(range)) {
              return;
            }
            // Landing inside a merged cell selects its anchor
            const landing = mergedCellContaining(mergedCells, cell.row, column);
            if (landing) {
              range.rowStart = landing.row;
              range.rowEnd = landing.row;
              range.columnStart = landing.column;
              range.columnEnd = landing.column;
            }
            cell.referencedRange = {
              range,
              str: rangeToStr(range, cell.sheet, sheetNames[range.sheet]),
              anchorRow: range.rowStart,
              anchorColumn: range.columnStart,
            };
            workbookState.setEditingCell(cell);
            onTextUpdated();
            return;
          }
          // at this point we finish editing and select the cell to the right
          // (or left if ShiftKey is pressed)
          workbookState.clearEditingCell();
          model.setUserInput(cell.sheet, cell.row, cell.column, cell.text);
          model.setSelectedSheet(cell.sheet);
          if (shiftKey) {
            // TODO: ShiftKey
          } else {
            model.onArrowRight();
          }
          if (textareaRef.current) {
            textareaRef.current.value = "";
          }

          onEditEnd();
          return;
        }
        case "ArrowLeft": {
          if (cell.mode === "edit") {
            return;
          }
          event.stopPropagation();
          event.preventDefault();
          if (cell.referencedRange) {
            // There is already a reference range we move it to the left
            // (or expand if shift is pressed)
            const sheetNames = model
              .getWorksheetsProperties()
              .map((s) => s.name);
            const range = cell.referencedRange.range;
            const mergedCells = model.getMergedCells(range.sheet);
            let { anchorRow, anchorColumn } = cell.referencedRange;
            if (shiftKey) {
              const activeColumn =
                anchorColumn === range.columnStart
                  ? range.columnEnd
                  : range.columnStart;
              const newActiveColumn = activeColumn - 1;
              if (newActiveColumn < 1) {
                return;
              }
              range.columnStart = Math.min(anchorColumn, newActiveColumn);
              range.columnEnd = Math.max(anchorColumn, newActiveColumn);
              // A reference can never cover part of a merged cell
              Object.assign(
                range,
                growRangeOverMergedCells(mergedCells, range),
              );
            } else {
              const activeRow =
                anchorRow === range.rowStart ? range.rowEnd : range.rowStart;
              const activeColumn =
                anchorColumn === range.columnStart
                  ? range.columnEnd
                  : range.columnStart;
              // Leaving a merged cell starts before its first column
              const leaving = mergedCellContaining(
                mergedCells,
                activeRow,
                activeColumn,
              );
              let column = leaving ? leaving.column - 1 : activeColumn - 1;
              let row = activeRow;
              if (column < 1) {
                return;
              }
              // Landing inside a merged cell selects its anchor
              const landing = mergedCellContaining(mergedCells, row, column);
              if (landing) {
                row = landing.row;
                column = landing.column;
              }
              range.columnStart = column;
              range.columnEnd = column;
              range.rowStart = row;
              range.rowEnd = row;
              anchorRow = row;
              anchorColumn = column;
            }
            cell.referencedRange = {
              range,
              str: rangeToStr(range, cell.sheet, sheetNames[range.sheet]),
              anchorRow,
              anchorColumn,
            };
            workbookState.setEditingCell(cell);
            onTextUpdated();
            return;
          }
          if (isInReferenceMode(model, cell.text, cell.cursorStart)) {
            // there is not a referenced Range but we are in reference mode
            // we select the next cell
            const sheetNames = model
              .getWorksheetsProperties()
              .map((s) => s.name);
            const mergedCells = model.getMergedCells(cell.sheet);
            // Leaving a merged cell starts before its first column
            const leaving = mergedCellContaining(
              mergedCells,
              cell.row,
              cell.column,
            );
            const column = leaving ? leaving.column - 1 : cell.column - 1;
            const range = {
              sheet: cell.sheet,
              rowStart: cell.row,
              rowEnd: cell.row,
              columnStart: column,
              columnEnd: column,
            };
            if (!isValidRange(range)) {
              return;
            }
            // Landing inside a merged cell selects its anchor
            const landing = mergedCellContaining(mergedCells, cell.row, column);
            if (landing) {
              range.rowStart = landing.row;
              range.rowEnd = landing.row;
              range.columnStart = landing.column;
              range.columnEnd = landing.column;
            }
            cell.referencedRange = {
              range,
              str: rangeToStr(range, cell.sheet, sheetNames[range.sheet]),
              anchorRow: range.rowStart,
              anchorColumn: range.columnStart,
            };
            workbookState.setEditingCell(cell);
            onTextUpdated();
            return;
          }
          // at this point we finish editing and select the cell to the right
          // (or left if ShiftKey is pressed)
          workbookState.clearEditingCell();
          model.setUserInput(cell.sheet, cell.row, cell.column, cell.text);
          model.setSelectedSheet(cell.sheet);
          if (shiftKey) {
            // TODO: ShiftKey
          } else {
            model.onArrowLeft();
          }
          if (textareaRef.current) {
            textareaRef.current.value = "";
          }

          onEditEnd();
          return;
        }
        case "ArrowUp": {
          if (cell.mode === "edit") {
            return;
          }
          event.stopPropagation();
          event.preventDefault();
          if (cell.referencedRange) {
            // There is already a reference range we move it up
            // (or expand if shift is pressed)
            const sheetNames = model
              .getWorksheetsProperties()
              .map((s) => s.name);
            const range = cell.referencedRange.range;
            const mergedCells = model.getMergedCells(range.sheet);
            let { anchorRow, anchorColumn } = cell.referencedRange;
            if (shiftKey) {
              const activeRow =
                anchorRow === range.rowStart ? range.rowEnd : range.rowStart;
              const newActiveRow = activeRow - 1;
              if (newActiveRow < 1) {
                return;
              }
              range.rowStart = Math.min(anchorRow, newActiveRow);
              range.rowEnd = Math.max(anchorRow, newActiveRow);
              // A reference can never cover part of a merged cell
              Object.assign(
                range,
                growRangeOverMergedCells(mergedCells, range),
              );
            } else {
              const activeRow =
                anchorRow === range.rowStart ? range.rowEnd : range.rowStart;
              const activeColumn =
                anchorColumn === range.columnStart
                  ? range.columnEnd
                  : range.columnStart;
              // Leaving a merged cell starts above its first row
              const leaving = mergedCellContaining(
                mergedCells,
                activeRow,
                activeColumn,
              );
              let row = leaving ? leaving.row - 1 : activeRow - 1;
              let column = activeColumn;
              if (row < 1) {
                return;
              }
              // Landing inside a merged cell selects its anchor
              const landing = mergedCellContaining(mergedCells, row, column);
              if (landing) {
                row = landing.row;
                column = landing.column;
              }
              range.columnStart = column;
              range.columnEnd = column;
              range.rowStart = row;
              range.rowEnd = row;
              anchorRow = row;
              anchorColumn = column;
            }
            cell.referencedRange = {
              range,
              str: rangeToStr(range, cell.sheet, sheetNames[range.sheet]),
              anchorRow,
              anchorColumn,
            };
            workbookState.setEditingCell(cell);
            onTextUpdated();
            return;
          }
          if (isInReferenceMode(model, cell.text, cell.cursorStart)) {
            // there is not a referenced Range but we are in reference mode
            // we select the next cell
            const sheetNames = model
              .getWorksheetsProperties()
              .map((s) => s.name);
            const mergedCells = model.getMergedCells(cell.sheet);
            // Leaving a merged cell starts above its first row
            const leaving = mergedCellContaining(
              mergedCells,
              cell.row,
              cell.column,
            );
            const row = leaving ? leaving.row - 1 : cell.row - 1;
            const range = {
              sheet: cell.sheet,
              rowStart: row,
              rowEnd: row,
              columnStart: cell.column,
              columnEnd: cell.column,
            };
            if (!isValidRange(range)) {
              return;
            }
            // Landing inside a merged cell selects its anchor
            const landing = mergedCellContaining(mergedCells, row, cell.column);
            if (landing) {
              range.rowStart = landing.row;
              range.rowEnd = landing.row;
              range.columnStart = landing.column;
              range.columnEnd = landing.column;
            }
            cell.referencedRange = {
              range,
              str: rangeToStr(range, cell.sheet, sheetNames[range.sheet]),
              anchorRow: range.rowStart,
              anchorColumn: range.columnStart,
            };
            workbookState.setEditingCell(cell);
            onTextUpdated();
            return;
          }
          // at this point we finish editing and select the cell to the right
          // (or left if ShiftKey is pressed)
          workbookState.clearEditingCell();
          model.setUserInput(cell.sheet, cell.row, cell.column, cell.text);
          model.setSelectedSheet(cell.sheet);
          if (shiftKey) {
            // TODO: ShiftKey
          } else {
            model.onArrowUp();
          }
          if (textareaRef.current) {
            textareaRef.current.value = "";
          }

          onEditEnd();
          return;
        }
        case "ArrowDown": {
          if (cell.mode === "edit") {
            return;
          }
          event.stopPropagation();
          event.preventDefault();
          if (cell.referencedRange) {
            // There is already a reference range we move it down
            // (or expand if shift is pressed)
            const sheetNames = model
              .getWorksheetsProperties()
              .map((s) => s.name);
            const range = cell.referencedRange.range;
            const mergedCells = model.getMergedCells(range.sheet);
            let { anchorRow, anchorColumn } = cell.referencedRange;
            if (shiftKey) {
              const activeRow =
                anchorRow === range.rowStart ? range.rowEnd : range.rowStart;
              const newActiveRow = activeRow + 1;
              if (newActiveRow > LAST_ROW) {
                return;
              }
              range.rowStart = Math.min(anchorRow, newActiveRow);
              range.rowEnd = Math.max(anchorRow, newActiveRow);
              // A reference can never cover part of a merged cell
              Object.assign(
                range,
                growRangeOverMergedCells(mergedCells, range),
              );
            } else {
              const activeRow =
                anchorRow === range.rowStart ? range.rowEnd : range.rowStart;
              const activeColumn =
                anchorColumn === range.columnStart
                  ? range.columnEnd
                  : range.columnStart;
              // Leaving a merged cell starts below its last row
              const leaving = mergedCellContaining(
                mergedCells,
                activeRow,
                activeColumn,
              );
              let row = leaving ? leaving.row + leaving.height : activeRow + 1;
              let column = activeColumn;
              if (row > LAST_ROW) {
                return;
              }
              // Landing inside a merged cell selects its anchor
              const landing = mergedCellContaining(mergedCells, row, column);
              if (landing) {
                row = landing.row;
                column = landing.column;
              }
              range.columnStart = column;
              range.columnEnd = column;
              range.rowStart = row;
              range.rowEnd = row;
              anchorRow = row;
              anchorColumn = column;
            }
            cell.referencedRange = {
              range,
              str: rangeToStr(range, cell.sheet, sheetNames[range.sheet]),
              anchorRow,
              anchorColumn,
            };
            workbookState.setEditingCell(cell);
            onTextUpdated();
            return;
          }
          if (isInReferenceMode(model, cell.text, cell.cursorStart)) {
            // there is not a referenced Range but we are in reference mode
            // we select the next cell
            const sheetNames = model
              .getWorksheetsProperties()
              .map((s) => s.name);
            const mergedCells = model.getMergedCells(cell.sheet);
            // Leaving a merged cell starts below its last row
            const leaving = mergedCellContaining(
              mergedCells,
              cell.row,
              cell.column,
            );
            const row = leaving ? leaving.row + leaving.height : cell.row + 1;
            const range = {
              sheet: cell.sheet,
              rowStart: row,
              rowEnd: row,
              columnStart: cell.column,
              columnEnd: cell.column,
            };
            if (!isValidRange(range)) {
              return;
            }
            // Landing inside a merged cell selects its anchor
            const landing = mergedCellContaining(mergedCells, row, cell.column);
            if (landing) {
              range.rowStart = landing.row;
              range.rowEnd = landing.row;
              range.columnStart = landing.column;
              range.columnEnd = landing.column;
            }
            cell.referencedRange = {
              range,
              str: rangeToStr(range, cell.sheet, sheetNames[range.sheet]),
              anchorRow: range.rowStart,
              anchorColumn: range.columnStart,
            };
            workbookState.setEditingCell(cell);
            onTextUpdated();
            return;
          }
          // at this point we finish editing and select the cell to the right
          // (or left if ShiftKey is pressed)
          workbookState.clearEditingCell();
          model.setUserInput(cell.sheet, cell.row, cell.column, cell.text);
          model.setSelectedSheet(cell.sheet);
          if (shiftKey) {
            // TODO: ShiftKey
          } else {
            model.onArrowDown();
          }
          if (textareaRef.current) {
            textareaRef.current.value = "";
          }

          onEditEnd();
          return;
        }
        case "Shift": {
          return;
        }
        case "PageDown":
        case "PageUp": {
          // TODO: We can do something similar to what we do with navigation keys
          event.stopPropagation();
          event.preventDefault();
          return;
        }
        case "End":
        case "Home": {
          // Excel does something similar to what we do with navigation keys
          cell.mode = "edit";
          workbookState.setEditingCell(cell);
          return;
        }
        case "F4": {
          const value = textarea.value;
          // the model works in characters, the textarea in UTF-16 code units
          const start = Array.from(
            value.slice(0, textarea.selectionStart),
          ).length;
          const end = Array.from(value.slice(0, textarea.selectionEnd)).length;

          const [newText, newStart, newEnd] = model.cycleReference(
            value,
            start,
            end,
          );
          const newChars = Array.from(newText);
          const selectionStart = newChars.slice(0, newStart).join("").length;
          const selectionEnd = newChars.slice(0, newEnd).join("").length;

          cell.text = newText;
          workbookState.setEditingCell(cell);
          setTimeout(() => {
            textarea.setSelectionRange(selectionStart, selectionEnd);
          }, 0);
          event.stopPropagation();
          event.preventDefault();
          onTextUpdated();
          return;
        }
        default: {
          // noop
        }
      }
    },
    [model, onEditEnd, onTextUpdated, workbookState, textareaRef.current],
  );
  return { onKeyDown };
};

export default useKeyDown;
