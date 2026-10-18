import type { Model } from "@ironcalc/wasm";
import {
  type PointerEvent,
  type RefObject,
  useCallback,
  useEffect,
  useRef,
} from "react";
import { isInReferenceMode } from "../Editor/util";
import type { Cell } from "../types";
import {
  growRangeOverMergedCells,
  mergedCellContaining,
  rangeToStr,
} from "../util";
import {
  headerColumnWidth,
  headerRowHeight,
  LAST_COLUMN,
  LAST_ROW,
} from "../WorksheetCanvas/constants";
import type WorksheetCanvas from "../WorksheetCanvas/worksheetCanvas";
import type { WorkbookState } from "../workbookState";

interface PointerSettings {
  canvasElement: RefObject<HTMLCanvasElement | null>;
  worksheetCanvas: RefObject<WorksheetCanvas | null>;
  worksheetElement: RefObject<HTMLDivElement | null>;
  onCellSelected: (cell: Cell, event: React.MouseEvent) => void;
  onRowSelected: (row: number, shift: boolean) => void;
  onColumnSelected: (column: number, shift: boolean) => void;
  onAllSheetSelected: () => void;
  onAreaSelecting: (cell: Cell) => void;
  onAreaSelected: () => void;
  model: Model;
  workbookState: WorkbookState;
  refresh: () => void;
}

interface PointerEvents {
  onPointerDown: (event: PointerEvent) => void;
  onPointerMove: (event: PointerEvent) => void;
  onPointerUp: (event: PointerEvent) => void;
  onPointerCancel: (event: PointerEvent) => void;
  /** True while a finger (or pen) is down on the sheet */
  isTouchDown: () => boolean;
}

/**
 * On touch devices a finger on the sheet pans it, like in the native
 * spreadsheet apps, so the gesture is only resolved once we know what it is:
 *
 * - a tap (lift without moving) selects the cell, row or column under it,
 * - a drag pans the sheet (the browser's native scroll),
 * - a hold of `TOUCH_HOLD_MS` without moving selects the cell and turns the
 *   rest of the touch into a selection drag, as a mouse drag would.
 *
 * `begin` is the mouse `pointerdown` logic, run when the gesture resolves to
 * a tap or a hold.
 */
interface TouchGesture {
  pointerId: number;
  x: number;
  y: number;
  timer: ReturnType<typeof setTimeout>;
  begin: () => void;
}

const TOUCH_HOLD_MS = 500;
// A finger never holds perfectly still: moves up to this distance are a hold
const TOUCH_SLOP_PX = 10;

const usePointer = (options: PointerSettings): PointerEvents => {
  const isSelecting = useRef(false);
  const isInsertingRef = useRef(false);
  const isSelectingRows = useRef(false);
  const initialRowRef = useRef<number | null>(null);
  const isSelectingColumns = useRef(false);
  const initialColumnRef = useRef<number | null>(null);
  const touchGesture = useRef<TouchGesture | null>(null);
  const touchDown = useRef(false);

  const isDragging = useCallback(
    (): boolean =>
      isSelecting.current ||
      isInsertingRef.current ||
      isSelectingRows.current ||
      isSelectingColumns.current,
    [],
  );

  const cancelTouchGesture = useCallback((): void => {
    const gesture = touchGesture.current;
    if (gesture) {
      clearTimeout(gesture.timer);
      touchGesture.current = null;
    }
  }, []);

  const releasePointer = useCallback(
    (pointerId: number): void => {
      const element = options.worksheetElement.current;
      // A tap selects on pointerup, when the touch pointer is already going
      // away and may have lost (or never got) its capture
      if (element?.hasPointerCapture(pointerId)) {
        element.releasePointerCapture(pointerId);
      }
    },
    [options],
  );

  useEffect(() => {
    const element = options.worksheetElement.current;
    if (!element) {
      return;
    }
    // Once a hold has turned the touch into a selection drag the finger must
    // not pan the sheet. `touch-action` cannot be changed mid-gesture, but
    // cancelling the touch moves keeps the browser from starting the pan.
    // React registers touch listeners as passive, hence the native listener.
    const onTouchMove = (event: TouchEvent): void => {
      if (isDragging() && event.cancelable) {
        event.preventDefault();
      }
    };
    element.addEventListener("touchmove", onTouchMove, { passive: false });
    return () => {
      element.removeEventListener("touchmove", onTouchMove);
      cancelTouchGesture();
    };
  }, [options.worksheetElement, isDragging, cancelTouchGesture]);

  const onPointerMove = useCallback(
    (event: PointerEvent): void => {
      const gesture = touchGesture.current;
      if (gesture && gesture.pointerId === event.pointerId) {
        // The finger moved before the hold resolved: this is a pan, which the
        // browser handles (and confirms with a pointercancel)
        if (
          Math.hypot(event.clientX - gesture.x, event.clientY - gesture.y) >
          TOUCH_SLOP_PX
        ) {
          cancelTouchGesture();
        }
        return;
      }

      if (!isDragging()) {
        return;
      }
      const { canvasElement, model, worksheetCanvas, refresh } = options;
      const canvas = canvasElement.current;
      const worksheet = worksheetCanvas.current;
      // Silence the linter
      if (!worksheet || !canvas) {
        return;
      }
      const canvasRect = canvas.getBoundingClientRect();
      const x = event.clientX - canvasRect.x;
      const y = event.clientY - canvasRect.y;

      if (isSelectingRows.current) {
        // Prevent text selection during row dragging
        event.preventDefault();
        // Handle row selection dragging
        if (initialRowRef.current === null) {
          return;
        }
        let targetRow: number | null = null;
        if (x >= 0 && x < headerColumnWidth && y >= headerRowHeight) {
          const cell = worksheet.getCellByCoordinates(headerColumnWidth, y);
          if (cell) {
            targetRow = cell.row;
          }
        } else if (x >= headerColumnWidth && y >= headerRowHeight) {
          const cell = worksheet.getCellByCoordinates(x, y);
          if (cell) {
            targetRow = cell.row;
          }
        }

        if (targetRow !== null) {
          const initialRow = initialRowRef.current;
          model.setSelectedCell(Math.min(initialRow, targetRow), 1);
          model.setSelectedRange(
            Math.min(initialRow, targetRow),
            1,
            Math.max(initialRow, targetRow),
            LAST_COLUMN,
          );
          refresh();
        }
        return;
      }

      if (isSelectingColumns.current) {
        // Prevent text selection during column dragging
        event.preventDefault();
        // Handle column selection dragging
        if (initialColumnRef.current === null) {
          return;
        }
        let targetColumn: number | null = null;
        if (x >= headerColumnWidth && y >= 0 && y < headerRowHeight) {
          const cell = worksheet.getCellByCoordinates(x, headerRowHeight);
          if (cell) {
            targetColumn = cell.column;
          }
        } else if (x >= headerColumnWidth && y >= headerRowHeight) {
          const cell = worksheet.getCellByCoordinates(x, y);
          if (cell) {
            targetColumn = cell.column;
          }
        }

        if (targetColumn !== null) {
          const initialColumn = initialColumnRef.current;
          model.setSelectedCell(1, Math.min(initialColumn, targetColumn));
          model.setSelectedRange(
            1,
            Math.min(initialColumn, targetColumn),
            LAST_ROW,
            Math.max(initialColumn, targetColumn),
          );
          refresh();
        }
        return;
      }

      const cell = worksheet.getCellByCoordinates(x, y);
      if (!cell) {
        return;
      }

      if (isSelecting.current) {
        options.onAreaSelecting(cell);
      } else if (isInsertingRef.current) {
        const { workbookState } = options;
        const editingCell = workbookState.getEditingCell();
        if (!editingCell?.referencedRange) {
          return;
        }
        const { anchorRow, anchorColumn } = editingCell.referencedRange;
        const range = editingCell.referencedRange.range;
        range.rowStart = anchorRow;
        range.columnStart = anchorColumn;
        range.rowEnd = cell.row;
        range.columnEnd = cell.column;
        // A reference can never cover part of a merged cell
        Object.assign(
          range,
          growRangeOverMergedCells(model.getMergedCells(range.sheet), range),
        );

        const sheetNames = model.getWorksheetsProperties().map((s) => s.name);

        editingCell.referencedRange.str = rangeToStr(
          range,
          editingCell.sheet,
          sheetNames[range.sheet],
        );
        workbookState.setEditingCell(editingCell);
        refresh();
      }
    },
    [options, isDragging, cancelTouchGesture],
  );

  const onPointerUp = useCallback(
    (event: PointerEvent): void => {
      const gesture = touchGesture.current;
      if (gesture && gesture.pointerId === event.pointerId) {
        // A tap: select what is under the finger, as a click would. The
        // drag state it sets is cleared right below, as for a mouse click
        // released in place.
        cancelTouchGesture();
        gesture.begin();
      }
      if (event.pointerType !== "mouse") {
        touchDown.current = false;
      }
      if (isSelecting.current) {
        isSelecting.current = false;
        releasePointer(event.pointerId);
        options.onAreaSelected();
      } else if (isInsertingRef.current) {
        isInsertingRef.current = false;
        releasePointer(event.pointerId);
      } else if (isSelectingRows.current) {
        isSelectingRows.current = false;
        initialRowRef.current = null;
        releasePointer(event.pointerId);
      } else if (isSelectingColumns.current) {
        isSelectingColumns.current = false;
        initialColumnRef.current = null;
        releasePointer(event.pointerId);
      }
    },
    [options, cancelTouchGesture, releasePointer],
  );

  const onPointerCancel = useCallback(
    (event: PointerEvent): void => {
      // The browser took the touch for a pan (or the system interrupted it)
      cancelTouchGesture();
      onPointerUp(event);
    },
    [onPointerUp, cancelTouchGesture],
  );

  // The mouse `pointerdown`: selects what is under the pointer and starts the
  // drag that may follow. On touch it runs once the gesture is resolved.
  const beginPointerDown = useCallback(
    (event: PointerEvent): void => {
      const target = event.target as HTMLElement;
      if (target.className === "column-resize-handle") {
        // we are resizing a column
        return;
      }
      if (target.className.includes("ironcalc-cell-handle")) {
        // we are extending values
        return;
      }
      if (event.button === 2) {
        return;
      }
      let x = event.clientX;
      let y = event.clientY;
      const {
        canvasElement,
        model,
        refresh,
        worksheetElement,
        worksheetCanvas,
        workbookState,
        onRowSelected,
        onColumnSelected,
        onAllSheetSelected,
      } = options;
      const worksheet = worksheetCanvas.current;
      const canvas = canvasElement.current;
      const worksheetWrapper = worksheetElement.current;
      // Silence the linter
      if (!canvas || !worksheet || !worksheetWrapper) {
        return;
      }
      const canvasRect = canvas.getBoundingClientRect();
      x -= canvasRect.x;
      y -= canvasRect.y;
      // Makes sure is in the sheet area
      if (
        x > canvasRect.width ||
        x < headerColumnWidth ||
        y < headerRowHeight ||
        y > canvasRect.height
      ) {
        if (x < headerColumnWidth && y < headerRowHeight) {
          // Click on the top left corner
          onAllSheetSelected();
        } else if (
          x > 0 &&
          x < headerColumnWidth &&
          y > headerRowHeight &&
          y < canvasRect.height
        ) {
          // Click on a row number
          const cell = worksheet.getCellByCoordinates(headerColumnWidth, y);
          if (cell) {
            if (event.shiftKey) {
              // Shift+click: extend selection
              onRowSelected(cell.row, true);
            } else {
              // Regular click: start drag selection
              event.preventDefault();
              initialRowRef.current = cell.row;
              isSelectingRows.current = true;
              worksheetWrapper.setPointerCapture(event.pointerId);
              onRowSelected(cell.row, false);
            }
          }
        } else if (
          x > headerColumnWidth &&
          x < canvasRect.width &&
          y > 0 &&
          y < headerRowHeight
        ) {
          // Click on a column letter
          const cell = worksheet.getCellByCoordinates(x, headerRowHeight);
          if (cell) {
            if (event.shiftKey) {
              // Shift+click: extend selection
              onColumnSelected(cell.column, true);
            } else {
              // Regular click: start drag selection
              event.preventDefault();
              initialColumnRef.current = cell.column;
              isSelectingColumns.current = true;
              worksheetWrapper.setPointerCapture(event.pointerId);
              onColumnSelected(cell.column, false);
            }
          }
        }
        return;
      }

      const editingCell = workbookState.getEditingCell();
      const cell = worksheet.getCellByCoordinates(x, y);
      if (cell) {
        if (editingCell) {
          if (
            model.getSelectedSheet() === editingCell.sheet &&
            cell.row === editingCell.row &&
            cell.column === editingCell.column
          ) {
            // We are clicking on the cell we are editing
            // we do nothing
            return;
          }
          // now we are editing one cell and we click in another one
          // If we can insert a range we do that
          const text = editingCell.text;
          if (isInReferenceMode(model, text, editingCell.cursorEnd)) {
            const sheet = model.getSelectedSheet();
            // Clicking inside a merged cell references its anchor
            const merge = mergedCellContaining(
              model.getMergedCells(sheet),
              cell.row,
              cell.column,
            );
            const row = merge ? merge.row : cell.row;
            const column = merge ? merge.column : cell.column;
            const range = {
              sheet,
              rowStart: row,
              rowEnd: row,
              columnStart: column,
              columnEnd: column,
            };
            const sheetNames = model
              .getWorksheetsProperties()
              .map((s) => s.name);
            editingCell.referencedRange = {
              range,
              str: rangeToStr(
                range,
                editingCell.sheet,
                sheetNames[range.sheet],
              ),
              anchorRow: range.rowStart,
              anchorColumn: range.columnStart,
            };
            workbookState.setEditingCell(editingCell);
            event.stopPropagation();
            event.preventDefault();
            isInsertingRef.current = true;
            worksheetWrapper.setPointerCapture(event.pointerId);
            refresh();
            return;
          }
          // We are clicking away but we are not in reference mode
          // We finish the editing
          model.setUserInput(
            editingCell.sheet,
            editingCell.row,
            editingCell.column,
            workbookState.getEditingText(),
          );
          workbookState.clearEditingCell();
          // we continue to select the new cell
        }
        if (event.shiftKey) {
          // We are extending the selection
          options.onAreaSelecting(cell);
          options.onAreaSelected();
        } else {
          // We are selecting a single cell
          options.onCellSelected(cell, event);
          isSelecting.current = true;
          worksheetWrapper.setPointerCapture(event.pointerId);
        }
      }
    },
    [options],
  );

  const onPointerDown = useCallback(
    (event: PointerEvent): void => {
      if (event.pointerType !== "mouse") {
        touchDown.current = true;
        cancelTouchGesture();
        touchGesture.current = {
          pointerId: event.pointerId,
          x: event.clientX,
          y: event.clientY,
          timer: setTimeout(() => {
            if (touchGesture.current?.pointerId !== event.pointerId) {
              return;
            }
            // A hold: select now and keep the drag state so that moving the
            // finger extends the selection
            touchGesture.current = null;
            beginPointerDown(event);
          }, TOUCH_HOLD_MS),
          begin: () => beginPointerDown(event),
        };
        return;
      }
      beginPointerDown(event);
    },
    [beginPointerDown, cancelTouchGesture],
  );

  return {
    onPointerDown,
    onPointerMove,
    onPointerUp,
    onPointerCancel,
    isTouchDown: () => touchDown.current,
  };
};

export default usePointer;
