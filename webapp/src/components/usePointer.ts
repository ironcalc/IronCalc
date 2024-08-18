import { type PointerEvent, type RefObject, useCallback, useRef } from "react";
import type WorksheetCanvas from "./WorksheetCanvas/worksheetCanvas";
import {
  headerColumnWidth,
  headerRowHeight,
} from "./WorksheetCanvas/worksheetCanvas";
import type { Cell } from "./types";

interface PointerSettings {
  canvasElement: RefObject<HTMLCanvasElement>;
  worksheetCanvas: RefObject<WorksheetCanvas | null>;
  worksheetElement: RefObject<HTMLDivElement>;
  onCellSelected: (cell: Cell, event: React.MouseEvent) => void;
  onAreaSelecting: (cell: Cell) => void;
  onAreaSelected: () => void;
  onExtendToCell: (cell: Cell) => void;
  onExtendToEnd: () => void;
}

interface PointerEvents {
  onPointerDown: (event: PointerEvent) => void;
  onPointerMove: (event: PointerEvent) => void;
  onPointerUp: (event: PointerEvent) => void;
  onPointerHandleDown: (event: PointerEvent) => void;
}

const usePointer = (options: PointerSettings): PointerEvents => {
  const isSelecting = useRef(false);
  const isExtending = useRef(false);

  const onPointerMove = useCallback(
    (event: PointerEvent): void => {
      // Range selections are disabled on non-mouse devices. Use touch move only
      // to scroll for now.
      if (event.pointerType !== "mouse") {
        return;
      }

      if (isSelecting.current) {
        const { canvasElement, worksheetCanvas } = options;
        const canvas = canvasElement.current;
        const worksheet = worksheetCanvas.current;
        // Silence the linter
        if (!worksheet || !canvas) {
          return;
        }
        let x = event.clientX;
        let y = event.clientY;
        const canvasRect = canvas.getBoundingClientRect();
        x -= canvasRect.x;
        y -= canvasRect.y;
        const cell = worksheet.getCellByCoordinates(x, y);
        if (cell) {
          options.onAreaSelecting(cell);
        } else {
          console.log("Failed");
        }
      } else if (isExtending.current) {
        const { canvasElement, worksheetCanvas } = options;
        const canvas = canvasElement.current;
        const worksheet = worksheetCanvas.current;
        // Silence the linter
        if (!worksheet || !canvas) {
          return;
        }
        let x = event.clientX;
        let y = event.clientY;
        const canvasRect = canvas.getBoundingClientRect();
        x -= canvasRect.x;
        y -= canvasRect.y;
        const cell = worksheet.getCellByCoordinates(x, y);
        if (!cell) {
          return;
        }
        options.onExtendToCell(cell);
      }
    },
    [options],
  );

  const onPointerUp = useCallback(
    (event: PointerEvent): void => {
      if (isSelecting.current) {
        const { worksheetElement } = options;
        isSelecting.current = false;
        worksheetElement.current?.releasePointerCapture(event.pointerId);
        options.onAreaSelected();
      } else if (isExtending.current) {
        const { worksheetElement } = options;
        isExtending.current = false;
        worksheetElement.current?.releasePointerCapture(event.pointerId);
        options.onExtendToEnd();
      }
    },
    [options],
  );

  const onPointerDown = useCallback(
    (event: PointerEvent) => {
      let x = event.clientX;
      let y = event.clientY;
      const { canvasElement, worksheetElement, worksheetCanvas } = options;
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
        if (
          x > 0 &&
          x < headerColumnWidth &&
          y > headerRowHeight &&
          y < canvasRect.height
        ) {
          // Click on a row number
          const cell = worksheet.getCellByCoordinates(headerColumnWidth, y);
          if (cell) {
            // TODO
            // Row selected
          }
        }
        return;
      }
      const cell = worksheet.getCellByCoordinates(x, y);
      if (cell) {
        options.onCellSelected(cell, event);
        isSelecting.current = true;
        worksheetWrapper.setPointerCapture(event.pointerId);
      }
    },
    [options],
  );

  const onPointerHandleDown = useCallback(
    (event: PointerEvent) => {
      const worksheetWrapper = options.worksheetElement.current;
      // Silence the linter
      if (!worksheetWrapper) {
        return;
      }
      isExtending.current = true;
      worksheetWrapper.setPointerCapture(event.pointerId);
      event.stopPropagation();
      event.preventDefault();
    },
    [options],
  );

  return {
    onPointerDown,
    onPointerMove,
    onPointerUp,
    onPointerHandleDown,
    // onContextMenu,
  };
};

export default usePointer;
