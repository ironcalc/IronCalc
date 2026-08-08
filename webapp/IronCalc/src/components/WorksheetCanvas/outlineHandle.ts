import { AreaType } from "../workbookState";
import { LAST_COLUMN, LAST_ROW } from "./constants";
import type WorksheetCanvas from "./worksheetCanvas";

export function attachOutlineHandle(
  worksheet: WorksheetCanvas,
): HTMLDivElement {
  // There is *always* a parent
  const parent = worksheet.canvas.parentElement as HTMLDivElement;

  // Remove any existing cell outline handles
  for (const handle of parent.querySelectorAll(".ironcalc-cell-handle")) {
    handle.remove();
  }

  // Create a new cell outline handle
  const cellOutlineHandle = document.createElement("div");
  cellOutlineHandle.className = "ironcalc-cell-handle";
  parent.appendChild(cellOutlineHandle);
  worksheet.cellOutlineHandle = cellOutlineHandle;

  // cell handle events
  const resizeHandleMove = (event: MouseEvent): void => {
    const canvasRect = worksheet.canvas.getBoundingClientRect();
    const x = event.clientX - canvasRect.x;
    const y = event.clientY - canvasRect.y;

    const cell = worksheet.getCellByCoordinates(x, y);
    if (!cell) {
      return;
    }
    const { row, column } = cell;
    const {
      range: [rowStart, columnStart, rowEnd, columnEnd],
    } = worksheet.model.getSelectedView();
    // We are either extending by rows or by columns
    // And we could be doing it in the positive direction (downwards or right)
    // or the negative direction (upwards or left)

    if (
      row > rowEnd &&
      ((column <= columnEnd && column >= columnStart) ||
        (column < columnStart && columnStart - column < row - rowEnd) ||
        (column > columnEnd && column - columnEnd < row - rowEnd))
    ) {
      // rows downwards
      const area = {
        type: AreaType.rowsDown,
        rowStart: rowEnd + 1,
        rowEnd: row,
        columnStart,
        columnEnd,
      };
      worksheet.workbookState.setExtendToArea(area);
      worksheet.renderSheet();
    } else if (
      row < rowStart &&
      ((column <= columnEnd && column >= columnStart) ||
        (column < columnStart && columnStart - column < rowStart - row) ||
        (column > columnEnd && column - columnEnd < rowStart - row))
    ) {
      // rows upwards
      const area = {
        type: AreaType.rowsUp,
        rowStart: row,
        rowEnd: rowStart,
        columnStart,
        columnEnd,
      };
      worksheet.workbookState.setExtendToArea(area);
      worksheet.renderSheet();
    } else if (
      column > columnEnd &&
      ((row <= rowEnd && row >= rowStart) ||
        (row < rowStart && rowStart - row < column - columnEnd) ||
        (row > rowEnd && row - rowEnd < column - columnEnd))
    ) {
      // columns right
      const area = {
        type: AreaType.columnsRight,
        rowStart,
        rowEnd,
        columnStart: columnEnd + 1,
        columnEnd: column,
      };
      worksheet.workbookState.setExtendToArea(area);
      worksheet.renderSheet();
    } else if (
      column < columnStart &&
      ((row <= rowEnd && row >= rowStart) ||
        (row < rowStart && rowStart - row < columnStart - column) ||
        (row > rowEnd && row - rowEnd < columnStart - column))
    ) {
      // columns left
      const area = {
        type: AreaType.columnsLeft,
        rowStart,
        rowEnd,
        columnStart: column,
        columnEnd: columnStart,
      };
      worksheet.workbookState.setExtendToArea(area);
      worksheet.renderSheet();
    }
  };

  const resizeHandleUp = (_event: MouseEvent): void => {
    document.removeEventListener("pointermove", resizeHandleMove);
    document.removeEventListener("pointerup", resizeHandleUp);

    const { sheet, range } = worksheet.model.getSelectedView();
    const extendedArea = worksheet.workbookState.getExtendToArea();
    if (!extendedArea) {
      return;
    }
    const rowStart = Math.min(range[0], range[2]);
    const height = Math.abs(range[2] - range[0]) + 1;
    const width = Math.abs(range[3] - range[1]) + 1;
    const columnStart = Math.min(range[1], range[3]);

    const area = {
      sheet,
      row: rowStart,
      column: columnStart,
      width,
      height,
    };

    try {
      switch (extendedArea.type) {
        case AreaType.rowsDown:
          worksheet.model.autoFillRows(area, extendedArea.rowEnd);
          break;
        case AreaType.rowsUp: {
          worksheet.model.autoFillRows(area, extendedArea.rowStart);
          break;
        }
        case AreaType.columnsRight: {
          worksheet.model.autoFillColumns(area, extendedArea.columnEnd);
          break;
        }
        case AreaType.columnsLeft: {
          worksheet.model.autoFillColumns(area, extendedArea.columnStart);
          break;
        }
      }
    } catch (_) {
      // This could fail if, for instnace we are breaking an array formula
      // Here we fail silently
      // TODO: Could we shouw a message to the user?
    }
    const selectedRowStart = Math.min(rowStart, extendedArea.rowStart);
    const selectedColumnStart = Math.min(columnStart, extendedArea.columnStart);
    const selectedRowEnd = Math.max(rowStart + height - 1, extendedArea.rowEnd);
    const selectedColumnEnd = Math.max(
      columnStart + width - 1,
      extendedArea.columnEnd,
    );

    worksheet.model.setSelectedCell(selectedRowStart, selectedColumnStart);
    worksheet.model.setSelectedRange(
      selectedRowStart,
      selectedColumnStart,
      selectedRowEnd,
      selectedColumnEnd,
    );
    worksheet.workbookState.clearExtendToArea();
    worksheet.renderSheet();
  };

  cellOutlineHandle.addEventListener("pointerdown", () => {
    document.addEventListener("pointermove", resizeHandleMove);
    document.addEventListener("pointerup", resizeHandleUp);
  });

  cellOutlineHandle.addEventListener("dblclick", (event) => {
    // On double-click, we will auto-fill the rows below the selected range
    event.stopPropagation();
    event.preventDefault();
    const { sheet, range } = worksheet.model.getSelectedView();
    const rowStart = Math.min(range[0], range[2]);
    const rowEnd = Math.max(range[0], range[2]);
    const columnStart = Math.min(range[1], range[3]);
    const columnEnd = Math.max(range[1], range[3]);
    const width = columnEnd - columnStart + 1;
    const height = rowEnd - rowStart + 1;

    let lastUsedRow = rowEnd;
    let testColumn = columnStart - 1;

    // The "test column" is the column to the left of the selected range,
    // or the column to the right if the left one is unavailable or empty.
    if (
      testColumn < 1 ||
      worksheet.model.getFormattedCellValue(sheet, rowStart, testColumn) === ""
    ) {
      testColumn = columnEnd + 1;
      if (
        testColumn > LAST_COLUMN ||
        worksheet.model.getFormattedCellValue(sheet, rowStart, testColumn) ===
          ""
      ) {
        return;
      }
    }

    // Find the last used row in the "test column"
    for (let r = rowEnd + 1; r <= LAST_ROW; r += 1) {
      if (worksheet.model.getFormattedCellValue(sheet, r, testColumn) === "") {
        break;
      }
      lastUsedRow = r;
    }

    for (let r = rowEnd + 1; r <= lastUsedRow; r += 1) {
      let isAnyCellNotEmpty = false;
      for (let c = columnStart; c <= columnEnd; c += 1) {
        if (worksheet.model.getFormattedCellValue(sheet, r, c) !== "") {
          isAnyCellNotEmpty = true;
          break;
        }
      }
      if (isAnyCellNotEmpty) {
        lastUsedRow = r - 1;
        break;
      }
    }

    if (lastUsedRow <= rowEnd) {
      return;
    }

    const area = {
      sheet,
      row: rowStart,
      column: columnStart,
      width,
      height,
    };

    worksheet.model.autoFillRows(area, lastUsedRow);
    worksheet.model.setSelectedRange(
      rowStart,
      columnStart,
      lastUsedRow,
      columnEnd,
    );
    worksheet.renderSheet();
  });
  return cellOutlineHandle;
}
