import { columnNameFromNumber, type Model } from "@ironcalc/wasm";
import { styled } from "@mui/material/styles";
import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { FORMULA_BAR_HEIGHT, NAVIGATION_HEIGHT } from "../constants";
import Editor from "../Editor/Editor";
import type { Cell } from "../types";
import {
  COLUMN_WIDTH_SCALE,
  LAST_COLUMN,
  LAST_ROW,
  outlineBackgroundColor,
  outlineColor,
  outlineEditingColor,
  ROW_HEIGH_SCALE,
} from "../WorksheetCanvas/constants";
import WorksheetCanvas, {
  headerColumnWidth,
  headerRowHeight,
} from "../WorksheetCanvas/worksheetCanvas";
import type { WorkbookState } from "../workbookState";
import ColumnHeaderContextMenu from "./ContextMenus/ColumnHeader";
import RowHeaderContextMenu from "./ContextMenus/RowHeader";
import usePointer from "./usePointer";

function useWindowSize() {
  const [size, setSize] = useState([0, 0]);
  useLayoutEffect(() => {
    function updateSize() {
      setSize([window.innerWidth, window.innerHeight]);
    }
    window.addEventListener("resize", updateSize);
    updateSize();
    return () => window.removeEventListener("resize", updateSize);
  }, []);
  return size;
}

const Worksheet = forwardRef(
  (
    props: {
      model: Model;
      workbookState: WorkbookState;
      refresh: () => void;
    },
    ref,
  ) => {
    const canvasElement = useRef<HTMLCanvasElement>(null);

    const worksheetElement = useRef<HTMLDivElement>(null);
    const scrollElement = useRef<HTMLDivElement>(null);

    const editorElement = useRef<HTMLDivElement>(null);
    const spacerElement = useRef<HTMLDivElement>(null);
    const cellOutline = useRef<HTMLDivElement>(null);
    const areaOutline = useRef<HTMLDivElement>(null);
    const extendToOutline = useRef<HTMLDivElement>(null);
    const columnResizeGuide = useRef<HTMLDivElement>(null);
    const rowResizeGuide = useRef<HTMLDivElement>(null);
    const columnHeaders = useRef<HTMLDivElement>(null);
    const worksheetCanvas = useRef<WorksheetCanvas | null>(null);

    const [colHeaderContextMenuOpen, setColHeaderContextMenuOpen] =
      useState(false);
    const [rowHeaderContextMenuOpen, setRowHeaderContextMenuOpen] =
      useState(false);
    const [contextMenuPosition, setContextMenuPosition] = useState<{
      top: number;
      left: number;
    } | null>(null);

    const ignoreScrollEventRef = useRef(false);

    const { model, workbookState, refresh } = props;
    const [clientWidth, clientHeight] = useWindowSize();

    useImperativeHandle(ref, () => ({
      getCanvas: () => worksheetCanvas.current,
    }));

    useEffect(() => {
      const canvasRef = canvasElement.current;
      const columnGuideRef = columnResizeGuide.current;
      const rowGuideRef = rowResizeGuide.current;
      const columnHeadersRef = columnHeaders.current;
      const worksheetRef = worksheetElement.current;

      const outline = cellOutline.current;
      const area = areaOutline.current;
      const extendTo = extendToOutline.current;
      const editor = editorElement.current;

      if (
        !canvasRef ||
        !columnGuideRef ||
        !rowGuideRef ||
        !columnHeadersRef ||
        !worksheetRef ||
        !outline ||
        !area ||
        !extendTo ||
        !scrollElement.current ||
        !editor
      )
        return;
      // FIXME: This two need to be computed.
      model.setWindowWidth(clientWidth - 37);
      model.setWindowHeight(clientHeight - 190);
      const canvas = new WorksheetCanvas({
        width: worksheetRef.clientWidth,
        height: worksheetRef.clientHeight,
        model,
        workbookState,
        elements: {
          canvas: canvasRef,
          columnGuide: columnGuideRef,
          rowGuide: rowGuideRef,
          columnHeaders: columnHeadersRef,
          cellOutline: outline,
          areaOutline: area,
          extendToOutline: extendTo,
          editor: editor,
        },
        onColumnWidthChanges(sheet, column, width) {
          if (width < 0) {
            return;
          }
          const { range } = model.getSelectedView();
          let columnStart = column;
          let columnEnd = column;
          const fullColumn = range[0] === 1 && range[2] === LAST_ROW;
          const fullRow = range[1] === 1 && range[3] === LAST_COLUMN;
          if (
            fullColumn &&
            column >= range[1] &&
            column <= range[3] &&
            !fullRow
          ) {
            columnStart = Math.min(range[1], column, range[3]);
            columnEnd = Math.max(range[1], column, range[3]);
          }
          model.setColumnsWidth(sheet, columnStart, columnEnd, width);
          worksheetCanvas.current?.renderSheet();
        },
        onRowHeightChanges(sheet, row, height) {
          if (height < 0) {
            return;
          }
          const { range } = model.getSelectedView();
          let rowStart = row;
          let rowEnd = row;
          const fullColumn = range[0] === 1 && range[2] === LAST_ROW;
          const fullRow = range[1] === 1 && range[3] === LAST_COLUMN;
          if (fullRow && row >= range[0] && row <= range[2] && !fullColumn) {
            rowStart = Math.min(range[0], row, range[2]);
            rowEnd = Math.max(range[0], row, range[2]);
          }
          model.setRowsHeight(sheet, rowStart, rowEnd, height);
          worksheetCanvas.current?.renderSheet();
        },
        refresh,
      });
      const scrollX = model.getScrollX();
      const scrollY = model.getScrollY();
      const [sheetWidth, sheetHeight] = [scrollX + 100_000, scrollY + 500_000];
      if (spacerElement.current) {
        spacerElement.current.style.height = `${sheetHeight}px`;
        spacerElement.current.style.width = `${sheetWidth}px`;
      }
      const left = scrollElement.current.scrollLeft;
      const top = scrollElement.current.scrollTop;
      if (scrollX !== left) {
        ignoreScrollEventRef.current = true;
        scrollElement.current.scrollLeft = scrollX;
        setTimeout(() => {
          ignoreScrollEventRef.current = false;
        }, 0);
      }

      if (scrollY !== top) {
        ignoreScrollEventRef.current = true;
        scrollElement.current.scrollTop = scrollY;
        setTimeout(() => {
          ignoreScrollEventRef.current = false;
        }, 0);
      }

      canvas.renderSheet();
      worksheetCanvas.current = canvas;
    });

    const { onPointerMove, onPointerDown, onPointerUp } = usePointer({
      model,
      workbookState,
      refresh,
      onColumnSelected: (column: number, shift: boolean) => {
        let firstColumn = column;
        let lastColumn = column;
        if (shift) {
          const { range } = model.getSelectedView();
          firstColumn = Math.min(range[1], column, range[3]);
          lastColumn = Math.max(range[3], column, range[1]);
        }
        model.setSelectedCell(1, firstColumn);
        model.setSelectedRange(1, firstColumn, LAST_ROW, lastColumn);
        refresh();
      },
      onRowSelected: (row: number, shift: boolean) => {
        let firstRow = row;
        let lastRow = row;
        if (shift) {
          const { range } = model.getSelectedView();
          firstRow = Math.min(range[0], row, range[2]);
          lastRow = Math.max(range[2], row, range[0]);
        }
        model.setSelectedCell(firstRow, 1);
        model.setSelectedRange(firstRow, 1, lastRow, LAST_COLUMN);
        refresh();
      },
      onAllSheetSelected: () => {
        model.setSelectedCell(1, 1);
        model.setSelectedRange(1, 1, LAST_ROW, LAST_COLUMN);
        refresh();
      },
      onCellSelected: (cell: Cell, event: React.MouseEvent) => {
        event.preventDefault();
        event.stopPropagation();
        model.setSelectedCell(cell.row, cell.column);
        refresh();
      },
      onAreaSelecting: (cell: Cell) => {
        const canvas = worksheetCanvas.current;
        if (!canvas) {
          return;
        }
        workbookState.setSelecting(true);
        const { row, column } = cell;
        model.onAreaSelecting(row, column);
        canvas.renderSheet();
        refresh();
      },
      onAreaSelected: () => {
        workbookState.setSelecting(false);
        const styles = workbookState.getCopyStyles();
        if (styles?.length) {
          model.onPasteStyles(styles);
          const canvas = worksheetCanvas.current;
          if (!canvas) {
            return;
          }
          canvas.renderSheet();
        }
        workbookState.setCopyStyles(null);
        if (worksheetElement.current) {
          worksheetElement.current.style.cursor = "auto";
        }
        refresh();
      },
      canvasElement,
      worksheetElement,
      worksheetCanvas,
    });

    const onScroll = (): void => {
      if (!scrollElement.current || !worksheetCanvas.current) {
        return;
      }
      if (ignoreScrollEventRef.current) {
        // Programmatic scroll ignored
        return;
      }
      const left = scrollElement.current.scrollLeft;
      const top = scrollElement.current.scrollTop;

      worksheetCanvas.current.setScrollPosition({ left, top });
      worksheetCanvas.current.renderSheet();
    };

    return (
      <Wrapper ref={scrollElement} onScroll={onScroll} className="scroll">
        <Spacer ref={spacerElement} />
        <SheetContainer
          className="sheet-container"
          ref={worksheetElement}
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onContextMenu={(event) => {
            event.preventDefault();
            event.stopPropagation();

            // Store mouse position for menu placement
            setContextMenuPosition({
              top: event.clientY,
              left: event.clientX,
            });

            // Detect if right-click is on column header or row header
            const canvas = canvasElement.current;
            if (canvas) {
              const canvasRect = canvas.getBoundingClientRect();
              const x = event.clientX - canvasRect.x;
              const y = event.clientY - canvasRect.y;

              // Check if click is in column header area
              if (
                x > headerColumnWidth &&
                x < canvasRect.width &&
                y > 0 &&
                y < headerRowHeight
              ) {
                const view = model.getSelectedView();
                const rowStart = view.range[0];
                const columnStart = view.range[1];
                const rowEnd = view.range[2];
                const columnEnd = view.range[3];
                const cell = worksheetCanvas.current?.getCellByCoordinates(
                  x,
                  headerRowHeight,
                );
                const column = cell?.column ?? view.column;
                if (!(rowStart === 1 && rowEnd === LAST_ROW)) {
                  // There are no columns selected, so select the column we clicked on
                  model.setSelectedCell(1, column);
                  model.setSelectedRange(1, column, LAST_ROW, column);
                  refresh();
                }
                if (!(columnStart <= column && column <= columnEnd)) {
                  // We clicked outside current selection, so select the column
                  const cell = worksheetCanvas.current?.getCellByCoordinates(
                    x,
                    headerRowHeight,
                  );
                  const column = cell?.column ?? view.column;
                  model.setSelectedCell(1, column);
                  model.setSelectedRange(1, column, LAST_ROW, column);
                  refresh();
                }
                setColHeaderContextMenuOpen(true);
                return;
              }

              // Check if click is in row header area
              if (
                x > 0 &&
                x < headerColumnWidth &&
                y > headerRowHeight &&
                y < canvasRect.height
              ) {
                const view = model.getSelectedView();
                const rowStart = view.range[0];
                const columnStart = view.range[1];
                const rowEnd = view.range[2];
                const columnEnd = view.range[3];
                const cell = worksheetCanvas.current?.getCellByCoordinates(
                  headerColumnWidth,
                  y,
                );
                const row = cell?.row ?? view.row;
                if (!(columnStart === 1 && columnEnd === LAST_COLUMN)) {
                  // There are no rows selected, so select the row we clicked on
                  model.setSelectedCell(row, 1);
                  model.setSelectedRange(row, 1, row, LAST_COLUMN);
                  refresh();
                }
                if (!(rowStart <= row && row <= rowEnd)) {
                  // We clicked outside current selection, so select the row
                  const cell = worksheetCanvas.current?.getCellByCoordinates(
                    headerColumnWidth,
                    y,
                  );
                  const row = cell?.row ?? view.row;
                  model.setSelectedCell(row, 1);
                  model.setSelectedRange(row, 1, row, LAST_COLUMN);
                  refresh();
                }
                setRowHeaderContextMenuOpen(true);
                return;
              }
            }

            // setContextMenuOpen(true);
          }}
          onDoubleClick={(event) => {
            // Starts editing cell
            const { sheet, row, column } = model.getSelectedView();
            const text = model.getCellContent(sheet, row, column);
            const editorWidth =
              model.getColumnWidth(sheet, column) * COLUMN_WIDTH_SCALE;
            const editorHeight =
              model.getRowHeight(sheet, row) * ROW_HEIGH_SCALE;
            workbookState.setEditingCell({
              sheet,
              row,
              column,
              text,
              cursorStart: text.length,
              cursorEnd: text.length,
              focus: "cell",
              referencedRange: null,
              activeRanges: [],
              mode: "accept",
              editorWidth,
              editorHeight,
            });
            event.stopPropagation();
            // event.preventDefault();
            props.refresh();
          }}
        >
          <SheetCanvas ref={canvasElement} />
          <CellOutline ref={cellOutline} />
          <EditorWrapper ref={editorElement}>
            <Editor
              originalText={workbookState.getEditingText()}
              onEditEnd={(): void => {
                props.refresh();
              }}
              onTextUpdated={(): void => {
                props.refresh();
              }}
              model={model}
              workbookState={workbookState}
              type={"cell"}
            />
          </EditorWrapper>
          <AreaOutline ref={areaOutline} />
          <ExtendToOutline ref={extendToOutline} />
          <ColumnResizeGuide ref={columnResizeGuide} />
          <RowResizeGuide ref={rowResizeGuide} />
          <ColumnHeaders ref={columnHeaders} />
        </SheetContainer>
        <ColumnHeaderContextMenu
          open={colHeaderContextMenuOpen}
          onClose={() => setColHeaderContextMenuOpen(false)}
          anchorPosition={contextMenuPosition}
          onInsertColumnsLeft={(): void => {
            const view = model.getSelectedView();
            const columnStart = view.range[1];
            const columnEnd = view.range[3];
            model.insertColumns(
              view.sheet,
              view.column,
              columnEnd - columnStart + 1,
            );
            setColHeaderContextMenuOpen(false);
          }}
          onInsertColumnsRight={(): void => {
            const view = model.getSelectedView();
            const columnStart = view.range[1];
            const columnEnd = view.range[3];
            model.insertColumns(
              view.sheet,
              columnEnd + 1,
              columnEnd - columnStart + 1,
            );
            setColHeaderContextMenuOpen(false);
          }}
          onMoveColumnsLeft={(): void => {
            const view = model.getSelectedView();
            const columnStart = view.range[1];
            const columnEnd = view.range[3];
            model.moveColumns(
              view.sheet,
              columnStart,
              columnEnd - columnStart + 1,
              -1,
            );
            setColHeaderContextMenuOpen(false);
          }}
          onMoveColumnsRight={(): void => {
            const view = model.getSelectedView();
            const columnStart = view.range[1];
            const columnEnd = view.range[3];
            model.moveColumns(
              view.sheet,
              columnStart,
              columnEnd - columnStart + 1,
              1,
            );
            setColHeaderContextMenuOpen(false);
          }}
          onFreezeColumns={(): void => {
            const view = model.getSelectedView();
            model.setFrozenColumnsCount(view.sheet, view.column);
            setColHeaderContextMenuOpen(false);
          }}
          onUnfreezeColumns={(): void => {
            const sheet = model.getSelectedSheet();
            model.setFrozenColumnsCount(sheet, 0);
            setColHeaderContextMenuOpen(false);
          }}
          onDeleteColumns={(): void => {
            const view = model.getSelectedView();
            const columnStart = view.range[1];
            const columnEnd = view.range[3];

            model.deleteColumns(
              view.sheet,
              columnStart,
              columnEnd - columnStart + 1,
            );
            setColHeaderContextMenuOpen(false);
          }}
          range={(() => {
            const range = model.getSelectedView().range;
            return {
              rowStart: range[0],
              columnStart: columnNameFromNumber(range[1]),
              rowEnd: range[2],
              columnEnd: columnNameFromNumber(range[3]),
              columnCount: range[3] - range[1] + 1,
            };
          })()}
          frozenColumnsCount={model.getFrozenColumnsCount(
            model.getSelectedSheet(),
          )}
          onHideColumns={(): void => {
            const view = model.getSelectedView();
            const columnStart = view.range[1];
            const columnEnd = view.range[3];
            model.setColumnsHidden(view.sheet, columnStart, columnEnd, true);
            setColHeaderContextMenuOpen(false);
          }}
          onShowHiddenColumns={(): void => {
            const view = model.getSelectedView();
            const columnStart = view.range[1];
            const columnEnd = view.range[3];
            model.setColumnsHidden(view.sheet, columnStart, columnEnd, false);
            setColHeaderContextMenuOpen(false);
          }}
          hiddenColumnsCount={(() => {
            const hiddenColumns = [];
            const view = model.getSelectedView();
            const columnStart = view.range[1];
            const columnEnd = view.range[3];
            for (let column = columnStart; column <= columnEnd; column++) {
              if (model.getColumnWidth(view.sheet, column) === 0) {
                hiddenColumns.push(column);
              }
            }
            return hiddenColumns.length;
          })()}
        />
        <RowHeaderContextMenu
          open={rowHeaderContextMenuOpen}
          onClose={() => setRowHeaderContextMenuOpen(false)}
          anchorPosition={contextMenuPosition}
          onInsertRowsAbove={(): void => {
            const view = model.getSelectedView();
            const rowStart = view.range[0];
            const rowEnd = view.range[2];
            model.insertRows(view.sheet, view.row, rowEnd - rowStart + 1);
            setRowHeaderContextMenuOpen(false);
          }}
          onInsertRowsBelow={(): void => {
            const view = model.getSelectedView();
            const rowStart = view.range[0];
            const rowEnd = view.range[2];
            model.insertRows(view.sheet, view.row + 1, rowEnd - rowStart + 1);
            setRowHeaderContextMenuOpen(false);
          }}
          onMoveRowsUp={(): void => {
            const view = model.getSelectedView();
            const rowStart = view.range[0];
            const rowEnd = view.range[2];
            model.moveRows(view.sheet, rowStart, rowEnd - rowStart + 1, -1);
            setRowHeaderContextMenuOpen(false);
          }}
          onMoveRowsDown={(): void => {
            const view = model.getSelectedView();
            const rowStart = view.range[0];
            const rowEnd = view.range[2];
            model.moveRows(view.sheet, rowStart, rowEnd - rowStart + 1, 1);
            setRowHeaderContextMenuOpen(false);
          }}
          onFreezeRows={(): void => {
            const view = model.getSelectedView();
            model.setFrozenRowsCount(view.sheet, view.row);
            setRowHeaderContextMenuOpen(false);
          }}
          onUnfreezeRows={(): void => {
            const sheet = model.getSelectedSheet();
            model.setFrozenRowsCount(sheet, 0);
            setRowHeaderContextMenuOpen(false);
          }}
          onDeleteRows={(): void => {
            const view = model.getSelectedView();
            const rowStart = view.range[0];
            const rowEnd = view.range[2];
            model.deleteRows(view.sheet, rowStart, rowEnd - rowStart + 1);
            setRowHeaderContextMenuOpen(false);
          }}
          onHideRows={(): void => {
            const view = model.getSelectedView();
            const rowStart = view.range[0];
            const rowEnd = view.range[2];
            model.setRowsHidden(view.sheet, rowStart, rowEnd, true);
            setRowHeaderContextMenuOpen(false);
          }}
          onShowHiddenRows={(): void => {
            const view = model.getSelectedView();
            const rowStart = view.range[0];
            const rowEnd = view.range[2];
            model.setRowsHidden(view.sheet, rowStart, rowEnd, false);
            setRowHeaderContextMenuOpen(false);
          }}
          range={(() => {
            const range = model.getSelectedView().range;
            return {
              rowStart: range[0],
              columnStart: columnNameFromNumber(range[1]),
              rowEnd: range[2],
              columnEnd: columnNameFromNumber(range[3]),
              columnCount: range[3] - range[1] + 1,
            };
          })()}
          frozenRowsCount={model.getFrozenRowsCount(model.getSelectedSheet())}
        />
      </Wrapper>
    );
  },
);

const Spacer = styled("div")`
  position: absolute;
  height: 5000px;
  width: 5000px;
`;

const SheetContainer = styled("div")`
  position: sticky;
  top: 0px;
  left: 0px;
  height: 100%;

  .column-resize-handle {
    position: absolute;
    top: 0px;
    width: 3px;
    opacity: 0;
    background: ${outlineColor};
    border-radius: 5px;
    cursor: col-resize;
  }

  .column-resize-handle:hover {
    opacity: 1;
  }
  .row-resize-handle {
    position: absolute;
    left: 0px;
    height: 3px;
    opacity: 0;
    background: ${outlineColor};
    border-radius: 5px;
    cursor: row-resize;
  }

  .row-resize-handle:hover {
    opacity: 1;
  }
`;

const Wrapper = styled("div")({
  position: "absolute",
  overflow: "scroll",
  top: FORMULA_BAR_HEIGHT + 1,
  left: 0,
  right: 0,
  bottom: NAVIGATION_HEIGHT + 1,
  overscrollBehavior: "none",
});

const SheetCanvas = styled("canvas")`
  position: relative;
  top: 0px;
  left: 0px;
  right: 0px;
  bottom: 40px;
`;

const ColumnResizeGuide = styled("div")`
  position: absolute;
  top: 0px;
  display: none;
  height: 100%;
  width: 0px;
  border-left: 1px dashed ${outlineColor};
`;

const ColumnHeaders = styled("div")`
  position: absolute;
  left: 0px;
  top: 0px;
  overflow: hidden;
  display: flex;
  & .column-header {
    display: inline-block;
    text-align: center;
    overflow: hidden;
    height: 100%;
    user-select: none;
  }
`;

const RowResizeGuide = styled("div")`
  position: absolute;
  display: none;
  left: 0px;
  height: 0px;
  width: 100%;
  border-top: 1px dashed ${outlineColor};
`;

const AreaOutline = styled("div")`
  position: absolute;
  border: 0px solid ${outlineColor};
  border-radius: 1px;
  background-color: ${outlineBackgroundColor};
`;

const CellOutline = styled("div")`
  position: absolute;
  border: 2px solid ${outlineColor};
  border-radius: 3px;
  word-break: break-word;
  font-size: 13px;
  display: flex;
  box-shadow: inset 0 0 0 1px white;
`;

const ExtendToOutline = styled("div")`
  position: absolute;
  border: 1px dashed ${outlineColor};
  border-radius: 3px;
`;

const EditorWrapper = styled("div")`
  position: absolute;
  width: 100%;
  padding: 0px;
  border-width: 0px;
  outline: none;
  resize: none;
  white-space: pre-wrap;
  vertical-align: bottom;
  overflow: hidden;
  text-align: left;
  outline: 3px solid ${outlineEditingColor};
  z-index: 1000;
  span {
    min-width: 1px;
  }
  font-family: monospace;
  border: 2px solid ${outlineColor};
`;

export default Worksheet;
