import { columnNameFromNumber, type Model } from "@ironcalc/wasm";
import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import Editor from "../Editor/Editor";
import type { Cell } from "../types";
import { getEditorSize } from "../util";
import type { WorkbookActions } from "../Workbook/workbookActions";
import type { LinkHoverCell } from "../WorksheetCanvas/cellLinks";
import {
  headerColumnWidth,
  headerRowHeight,
  LAST_COLUMN,
  LAST_ROW,
} from "../WorksheetCanvas/constants";
import WorksheetCanvas from "../WorksheetCanvas/worksheetCanvas";
import type { WorkbookState } from "../workbookState";
import CellContextMenu from "./ContextMenus/Cell";
import ColumnHeaderContextMenu from "./ContextMenus/ColumnHeader";
import RowHeaderContextMenu from "./ContextMenus/RowHeader";
import LinkTooltip from "./LinkTooltip";
import usePointer from "./usePointer";
import "./worksheet.css";
import { Alert, Prompt } from "../Modal";

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
      canEdit: boolean;
      // Shared with the toolbar, the keyboard and the host application
      actions: WorkbookActions;
      onEditLink?: (row: number, column: number) => void;
      onDeleteLink?: (row: number, column: number) => void;
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
    const cellArrayStructure = useRef<HTMLDivElement>(null);
    const extendToOutline = useRef<HTMLDivElement>(null);
    const columnResizeGuide = useRef<HTMLDivElement>(null);
    const rowResizeGuide = useRef<HTMLDivElement>(null);
    const columnHeaders = useRef<HTMLDivElement>(null);
    const worksheetCanvas = useRef<WorksheetCanvas | null>(null);

    const [cellContextMenuOpen, setCellContextMenuOpen] = useState(false);
    const [colHeaderContextMenuOpen, setColHeaderContextMenuOpen] =
      useState(false);
    const [rowHeaderContextMenuOpen, setRowHeaderContextMenuOpen] =
      useState(false);
    const [contextMenuPosition, setContextMenuPosition] = useState<{
      top: number;
      left: number;
    } | null>(null);
    const [rowColErrorTitle, setRowColErrorTitle] = useState<string | null>(
      null,
    );
    const [columnWidthDialogOpen, setColumnWidthDialogOpen] = useState(false);
    const [columnWidthDefault, setColumnWidthDefault] = useState("");
    // The raw engine error of a rejected autofill (localized when rendered)
    const [autofillError, setAutofillError] = useState<string | null>(null);
    const [rowHeightDialogOpen, setRowHeightDialogOpen] = useState(false);
    const [rowHeightDefault, setRowHeightDefault] = useState("");

    const ignoreScrollEventRef = useRef(false);
    // The animation frame that will repaint the canvas after a scroll, if one
    // is pending. Scroll events can arrive several times per frame and a
    // repaint is only useful once per frame, so onScroll coalesces them.
    const scrollFrame = useRef<number | null>(null);
    const renderOnNextFrame = (): void => {
      if (scrollFrame.current !== null) {
        return;
      }
      scrollFrame.current = requestAnimationFrame(() => {
        scrollFrame.current = null;
        worksheetCanvas.current?.renderSheet();
      });
    };
    const cancelPendingRender = (): void => {
      if (scrollFrame.current !== null) {
        cancelAnimationFrame(scrollFrame.current);
        scrollFrame.current = null;
      }
    };
    useEffect(
      () => () => {
        if (scrollFrame.current !== null) {
          cancelAnimationFrame(scrollFrame.current);
        }
      },
      [],
    );
    // The engine's view (its top-left cell) as of the last time it was
    // synced with the DOM scroller, in either direction (see the render
    // effect and onScroll)
    const syncedView = useRef<{
      sheet: number;
      topRow: number;
      leftColumn: number;
    } | null>(null);
    const currentView = (): {
      sheet: number;
      topRow: number;
      leftColumn: number;
    } => {
      const view = model.getSelectedView();
      return {
        sheet: view.sheet,
        topRow: view.top_row,
        leftColumn: view.left_column,
      };
    };
    // The selection as of the last render: the active cell and the
    // normalized range
    const lastSelection = useRef<{
      sheet: number;
      row: number;
      column: number;
      range: [number, number, number, number];
    } | null>(null);
    const currentSelection = (): NonNullable<typeof lastSelection.current> => {
      const { sheet, row, column, range } = model.getSelectedView();
      const [a, b, c, d] = range;
      return {
        sheet,
        row,
        column,
        range: [Math.min(a, c), Math.min(b, d), Math.max(a, c), Math.max(b, d)],
      };
    };
    // The scroll offset of every sheet shown so far, by sheet id (stable
    // across adding, deleting and moving sheets), with the selection as of
    // the last time the sheet was shown. The engine remembers only the cell
    // at the offset; this keeps the sub-cell part across sheet switches.
    const sheetScroll = useRef(
      new Map<
        number,
        { x: number; y: number; selection: typeof lastSelection.current }
      >(),
    );
    const selectedSheetId = (): number =>
      model.getWorksheetsProperties()[model.getSelectedSheet()].sheet_id;
    // The frozen rows and columns as of the last render: the scroll offset
    // counts from the first unfrozen cell, so it resets when they change
    const lastFrozen = useRef<{
      sheet: number;
      rows: number;
      columns: number;
    } | null>(null);
    // The cell to bring into view after a render, if the selection moved
    // since the last one: the active cell when it moved (keyboard
    // navigation, a click, the end of an edit, a link, ...); otherwise the
    // corner of the range opposite the active cell, the one Shift+arrows
    // and a drag move. Without a previous selection (the first render, a
    // sheet shown for the first time) the active cell. Whole rows and
    // columns (header clicks, select all) never scroll.
    const revealTarget = (
      previous: typeof lastSelection.current,
      selection: NonNullable<typeof lastSelection.current>,
    ): { row: number; column: number } | null => {
      const [r0, c0, r1, c1] = selection.range;
      if ((r0 === 1 && r1 === LAST_ROW) || (c0 === 1 && c1 === LAST_COLUMN)) {
        return null;
      }
      if (
        !previous ||
        previous.row !== selection.row ||
        previous.column !== selection.column
      ) {
        return { row: selection.row, column: selection.column };
      }
      if (previous.range.some((v, i) => v !== selection.range[i])) {
        return {
          row: r1 > selection.row ? r1 : r0,
          column: c1 > selection.column ? c1 : c0,
        };
      }
      return null;
    };

    // The spacer makes the scroller scrollable: as wide and tall as the
    // sheet. Browsers cap the scrollable extent at some tens of millions of
    // pixels, less than a million rows, so it only reaches well past the
    // current position and grows as the user scrolls.
    const resizeSpacer = (
      canvas: WorksheetCanvas,
      left: number,
      top: number,
    ): void => {
      const scroller = scrollElement.current;
      const spacer = spacerElement.current;
      if (!scroller || !spacer) {
        return;
      }
      const max = canvas.getMaxScroll();
      const width = `${Math.min(max.x, left + 100_000) + scroller.clientWidth}px`;
      const height = `${Math.min(max.y, top + 500_000) + scroller.clientHeight}px`;
      if (spacer.style.width !== width) {
        spacer.style.width = width;
      }
      if (spacer.style.height !== height) {
        spacer.style.height = height;
      }
    };

    // The cell whose link tooltip is shown (null if hidden). The canvas does
    // the hover hit-testing and reports it through onLinkHover; the tooltip
    // itself is the LinkTooltip component. Hiding is delayed so the pointer
    // can travel from the cell into the tooltip without dismissing it.
    const [linkTooltipCell, setLinkTooltipCell] =
      useState<LinkHoverCell | null>(null);
    const linkTooltipHideTimeout = useRef<ReturnType<typeof setTimeout> | null>(
      null,
    );
    const cancelHideLinkTooltip = (): void => {
      if (linkTooltipHideTimeout.current !== null) {
        clearTimeout(linkTooltipHideTimeout.current);
        linkTooltipHideTimeout.current = null;
      }
    };
    const hideLinkTooltip = (): void => {
      cancelHideLinkTooltip();
      setLinkTooltipCell(null);
    };
    const scheduleHideLinkTooltip = (): void => {
      if (linkTooltipHideTimeout.current !== null) {
        return;
      }
      linkTooltipHideTimeout.current = setTimeout(() => {
        linkTooltipHideTimeout.current = null;
        setLinkTooltipCell(null);
      }, 300);
    };

    const { model, workbookState, refresh, canEdit, actions } = props;
    const { t } = useTranslation();
    // Re-renders (and so re-creates the canvas, below) on window resizes
    useWindowSize();

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
      const arrayStructure = cellArrayStructure.current;
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
        !editor ||
        !arrayStructure ||
        !canvasRef.closest(".ic-root")
      ) {
        return;
      }
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
          cellArrayStructure: arrayStructure,
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
        onLinkHover: (cell) => {
          if (cell) {
            cancelHideLinkTooltip();
            // keep the state (and the tooltip) when still on the same cell
            setLinkTooltipCell((previous) =>
              previous &&
              previous.row === cell.row &&
              previous.column === cell.column
                ? previous
                : cell,
            );
          } else if (linkTooltipCell !== null) {
            scheduleHideLinkTooltip();
          }
        },
        onHideLinkTooltip: hideLinkTooltip,
        onAutofillError: (message) => {
          // the known rejections get a dialog; anything else stays silent
          if (
            message.includes("merged cell") ||
            message.includes("array formula")
          ) {
            setAutofillError(message);
          }
        },
        linkTooltipCell,
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
      // The engine's own scrolling (page up / down, ...) works in the
      // scrolling pane
      const pane = canvas.getScrollPaneSize();
      model.setWindowWidth(pane.width);
      model.setWindowHeight(pane.height);

      // The scroll position. The DOM scroller is the source of truth while
      // the user scrolls; the engine only keeps the cell at the offset, and
      // a canvas (re-created on every render) takes the offset from the
      // scroller. When the selection moved, the view scrolls as little as
      // needed to show it (`reveal`); the engine's own scrolling in whole
      // cells, which comes with its navigation, is ignored. When the engine
      // moved its view without moving the selection (a request to show a
      // cell, ...) the scroller follows the engine along the axis that
      // moved. Comparing cells rather than pixel positions lets the
      // scroller sit mid-cell without being "corrected" to the cell
      // boundary by every unrelated re-render (e.g. the link tooltip on
      // pointer moves). On a sheet switch the sheet's remembered offset is
      // restored, as long as the engine still has the same cell at it.
      const scroller = scrollElement.current;
      const view = currentView();
      const synced = syncedView.current;
      const selection = currentSelection();
      const sheetId = selectedSheetId();
      const remembered = sheetScroll.current.get(sheetId);
      const sameSheet = synced !== null && synced.sheet === view.sheet;
      const frozen = {
        sheet: view.sheet,
        rows: model.getFrozenRowsCount(view.sheet),
        columns: model.getFrozenColumnsCount(view.sheet),
      };
      const frozenRowsChanged =
        sameSheet &&
        lastFrozen.current !== null &&
        lastFrozen.current.rows !== frozen.rows;
      const frozenColumnsChanged =
        sameSheet &&
        lastFrozen.current !== null &&
        lastFrozen.current.columns !== frozen.columns;
      let target = revealTarget(
        sameSheet ? lastSelection.current : (remembered?.selection ?? null),
        selection,
      );
      if (!target && (frozenRowsChanged || frozenColumnsChanged)) {
        target = { row: selection.row, column: selection.column };
      }
      const engine = canvas.getScroll();
      let x = engine.x;
      let y = engine.y;
      if (sameSheet) {
        if (frozenColumnsChanged) {
          x = 0;
        } else if (target || synced.leftColumn === view.leftColumn) {
          x = scroller.scrollLeft;
        }
        if (frozenRowsChanged) {
          y = 0;
        } else if (target || synced.topRow === view.topRow) {
          y = scroller.scrollTop;
        }
      } else if (remembered) {
        // The remembered offset, where the engine's cell is still the one
        // at it (it is not when the engine's view moved since: a row
        // resized above it in a shared workbook, say)
        canvas.setScroll(remembered.x, remembered.y);
        const at = currentView();
        if (at.leftColumn === view.leftColumn) {
          x = remembered.x;
        }
        if (at.topRow === view.topRow) {
          y = remembered.y;
        }
      }
      canvas.setScroll(x, y);
      if (target) {
        canvas.reveal(target.row, target.column);
      }
      lastSelection.current = selection;
      lastFrozen.current = frozen;
      ({ x, y } = canvas.getScroll());
      sheetScroll.current.set(sheetId, { x, y, selection });
      resizeSpacer(canvas, x, y);
      const previousLeft = scroller.scrollLeft;
      const previousTop = scroller.scrollTop;
      scroller.scrollLeft = x;
      scroller.scrollTop = y;
      if (
        scroller.scrollLeft !== previousLeft ||
        scroller.scrollTop !== previousTop
      ) {
        // the write fires a single (asynchronous) scroll event, consumed in
        // onScroll; when the browser quantizes the value back to the current
        // position no event fires, so the flag must stay clear
        ignoreScrollEventRef.current = true;
      }
      syncedView.current = currentView();

      // This render supersedes any repaint a scroll had scheduled
      cancelPendingRender();
      canvas.renderSheet();
      worksheetCanvas.current = canvas;
    });

    const {
      onPointerMove,
      onPointerDown,
      onPointerUp,
      onPointerCancel,
      isTouchDown,
    } = usePointer({
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
        // Consume the one scroll event fired by the programmatic sync in the
        // render effect. (Clearing the flag on a timeout instead would lose
        // the race against the asynchronous event and process the sync as a
        // user scroll.)
        ignoreScrollEventRef.current = false;
        return;
      }
      const left = scrollElement.current.scrollLeft;
      const top = scrollElement.current.scrollTop;

      const canvas = worksheetCanvas.current;
      canvas.setScrollPosition({ left, top });
      resizeSpacer(canvas, left, top);
      // The engine and the spacer follow every event (cheap, and the pointer
      // handlers rely on the engine's view being current); the repaint waits
      // for the next frame
      renderOnNextFrame();
      // The engine's cell followed the offset: the next render must not take
      // that for a move of the engine's own
      syncedView.current = currentView();
      const { x, y } = canvas.getScroll();
      sheetScroll.current.set(selectedSheetId(), {
        x,
        y,
        selection: lastSelection.current,
      });
    };

    return (
      <div
        ref={scrollElement}
        onScroll={onScroll}
        className="ic-worksheet-wrapper scroll"
      >
        <div className="ic-worksheet-spacer" ref={spacerElement} />
        {/* biome-ignore lint/a11y/noStaticElementInteractions: spreadsheet surface handles pointer interactions */}
        <div
          className="ic-worksheet-sheet-container"
          ref={worksheetElement}
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerCancel={onPointerCancel}
          onContextMenu={(event) => {
            event.preventDefault();
            event.stopPropagation();

            // Android raises a context menu on a long press, which on touch
            // is the gesture that starts a selection drag instead.
            if (isTouchDown()) {
              return;
            }

            // The context menus only offer editing actions (insert/delete,
            // cut/paste), so they are not shown in read-only mode.
            if (!canEdit) {
              return;
            }

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
              // Cell area: select the clicked cell
              const cell = worksheetCanvas.current?.getCellByCoordinates(x, y);
              if (cell) {
                const { range } = model.getSelectedView();
                const [rowStart, columnStart, rowEnd, columnEnd] = range;
                if (
                  !(
                    rowStart <= cell.row &&
                    cell.row <= rowEnd &&
                    columnStart <= cell.column &&
                    cell.column <= columnEnd
                  )
                ) {
                  model.setSelectedCell(cell.row, cell.column);
                  refresh();
                }
              }
            }

            setCellContextMenuOpen(true);
          }}
          onDoubleClick={(event) => {
            // Starts editing cell
            if (!canEdit) {
              return;
            }
            const { sheet, row, column } = model.getSelectedView();
            const text = model.getCellContent(sheet, row, column);
            const { width: editorWidth, height: editorHeight } = getEditorSize(
              model,
              sheet,
              row,
              column,
            );
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
          <canvas className="ic-worksheet-sheet-canvas" ref={canvasElement} />
          <div
            className="ic-worksheet-cell-array-structure"
            ref={cellArrayStructure}
          />
          <div className="ic-worksheet-cell-outline" ref={cellOutline} />
          <div className="ic-worksheet-editor-wrapper" ref={editorElement}>
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
              canEdit={canEdit}
            />
          </div>
          <div className="ic-worksheet-area-outline" ref={areaOutline} />
          <div
            className="ic-worksheet-extend-to-outline"
            ref={extendToOutline}
          />
          <div
            className="ic-worksheet-column-resize-guide"
            ref={columnResizeGuide}
          />
          <div className="ic-worksheet-row-resize-guide" ref={rowResizeGuide} />
          <div className="ic-worksheet-column-headers" ref={columnHeaders} />
          {linkTooltipCell ? (
            <LinkTooltip
              cell={linkTooltipCell}
              onFollow={(link) => {
                worksheetCanvas.current?.followLink(link);
              }}
              onEdit={props.onEditLink}
              onDelete={props.onDeleteLink}
              onHide={hideLinkTooltip}
              onPointerEnter={cancelHideLinkTooltip}
              onPointerLeave={scheduleHideLinkTooltip}
            />
          ) : null}
        </div>
        <CellContextMenu
          open={cellContextMenuOpen}
          onClose={() => setCellContextMenuOpen(false)}
          anchorPosition={
            contextMenuPosition
              ? { x: contextMenuPosition.left, y: contextMenuPosition.top }
              : null
          }
          column={columnNameFromNumber(model.getSelectedView().column)}
          row={model.getSelectedView().row}
          onCut={actions.cut}
          onCopy={actions.copy}
          onPaste={actions.paste}
          onInsertColumnLeft={(): void => {
            setCellContextMenuOpen(false);
            actions.insertColumnsLeft();
          }}
          onInsertColumnRight={(): void => {
            setCellContextMenuOpen(false);
            actions.insertColumnsRight();
          }}
          onInsertRowAbove={(): void => {
            setCellContextMenuOpen(false);
            actions.insertRowsAbove();
          }}
          onInsertRowBelow={(): void => {
            setCellContextMenuOpen(false);
            actions.insertRowsBelow();
          }}
          onDeleteColumn={(): void => {
            const view = model.getSelectedView();
            try {
              model.deleteColumns(view.sheet, view.column, 1);
            } catch {
              setRowColErrorTitle(t("error_dialog.error_deleting_columns"));
            }
            setCellContextMenuOpen(false);
            refresh();
          }}
          onDeleteRow={(): void => {
            const view = model.getSelectedView();
            try {
              model.deleteRows(view.sheet, view.row, 1);
            } catch {
              setRowColErrorTitle(t("error_dialog.error_deleting_rows"));
            }
            setCellContextMenuOpen(false);
            refresh();
          }}
        />
        <ColumnHeaderContextMenu
          open={colHeaderContextMenuOpen}
          onClose={() => setColHeaderContextMenuOpen(false)}
          anchorPosition={
            contextMenuPosition
              ? { x: contextMenuPosition.left, y: contextMenuPosition.top }
              : null
          }
          onInsertColumnsLeft={(): void => {
            setColHeaderContextMenuOpen(false);
            actions.insertColumnsLeft();
          }}
          onInsertColumnsRight={(): void => {
            setColHeaderContextMenuOpen(false);
            actions.insertColumnsRight();
          }}
          onMoveColumnsLeft={(): void => {
            const view = model.getSelectedView();
            const columnStart = view.range[1];
            const columnEnd = view.range[3];
            try {
              model.moveColumns(
                view.sheet,
                columnStart,
                columnEnd - columnStart + 1,
                -1,
              );
            } catch {
              setRowColErrorTitle(t("error_dialog.error_moving_columns"));
            }
            setColHeaderContextMenuOpen(false);
          }}
          onMoveColumnsRight={(): void => {
            const view = model.getSelectedView();
            const columnStart = view.range[1];
            const columnEnd = view.range[3];
            try {
              model.moveColumns(
                view.sheet,
                columnStart,
                columnEnd - columnStart + 1,
                1,
              );
            } catch {
              setRowColErrorTitle(t("error_dialog.error_moving_columns"));
            }
            setColHeaderContextMenuOpen(false);
          }}
          onSetColumnWidth={(): void => {
            const view = model.getSelectedView();
            const width = model.getColumnWidth(view.sheet, view.range[1]);
            setColumnWidthDefault(`${width}`);
            setColumnWidthDialogOpen(true);
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
            try {
              model.deleteColumns(
                view.sheet,
                columnStart,
                columnEnd - columnStart + 1,
              );
            } catch {
              setRowColErrorTitle(t("error_dialog.error_deleting_columns"));
            }
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
          anchorPosition={
            contextMenuPosition
              ? { x: contextMenuPosition.left, y: contextMenuPosition.top }
              : null
          }
          onInsertRowsAbove={(): void => {
            setRowHeaderContextMenuOpen(false);
            actions.insertRowsAbove();
          }}
          onInsertRowsBelow={(): void => {
            setRowHeaderContextMenuOpen(false);
            actions.insertRowsBelow();
          }}
          onMoveRowsUp={(): void => {
            const view = model.getSelectedView();
            const rowStart = view.range[0];
            const rowEnd = view.range[2];
            try {
              model.moveRows(view.sheet, rowStart, rowEnd - rowStart + 1, -1);
            } catch {
              setRowColErrorTitle(t("error_dialog.error_moving_rows"));
            }
            setRowHeaderContextMenuOpen(false);
          }}
          onMoveRowsDown={(): void => {
            const view = model.getSelectedView();
            const rowStart = view.range[0];
            const rowEnd = view.range[2];
            try {
              model.moveRows(view.sheet, rowStart, rowEnd - rowStart + 1, 1);
            } catch {
              setRowColErrorTitle(t("error_dialog.error_moving_rows"));
            }
            setRowHeaderContextMenuOpen(false);
          }}
          onSetRowHeight={(): void => {
            const view = model.getSelectedView();
            const height = model.getRowHeight(view.sheet, view.range[0]);
            setRowHeightDefault(`${height}`);
            setRowHeightDialogOpen(true);
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
            try {
              model.deleteRows(view.sheet, rowStart, rowEnd - rowStart + 1);
            } catch {
              setRowColErrorTitle(t("error_dialog.error_deleting_rows"));
            }
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
        <Alert
          open={rowColErrorTitle !== null}
          onClose={() => setRowColErrorTitle(null)}
          title={rowColErrorTitle ?? ""}
          message={rowColErrorTitle ?? ""}
        />
        <Alert
          open={autofillError !== null}
          onClose={() => setAutofillError(null)}
          title={t("error_dialog.error_autofill")}
          message={
            autofillError?.includes("merged cell")
              ? t("error_dialog.error_autofill_merged")
              : t("error_dialog.error_autofill_array")
          }
        />
        <Prompt
          open={columnWidthDialogOpen}
          onClose={() => setColumnWidthDialogOpen(false)}
          title={t("context_menu.column_header.set_column_width")}
          confirmLabel={t("common.apply")}
          cancelLabel={t("common.cancel")}
          defaultValue={columnWidthDefault}
          inputProps={{
            type: "number",
            min: 0,
            step: "any",
            endAdornment: "px",
          }}
          onSubmit={(value): void => {
            const width = Number.parseFloat(value);
            if (!Number.isFinite(width) || width < 0) {
              return;
            }
            const view = model.getSelectedView();
            const columnStart = view.range[1];
            const columnEnd = view.range[3];
            model.setColumnsWidth(view.sheet, columnStart, columnEnd, width);
            worksheetCanvas.current?.renderSheet();
          }}
        />
        <Prompt
          open={rowHeightDialogOpen}
          onClose={() => setRowHeightDialogOpen(false)}
          title={t("context_menu.row_header.set_row_height")}
          confirmLabel={t("common.apply")}
          cancelLabel={t("common.cancel")}
          defaultValue={rowHeightDefault}
          inputProps={{
            type: "number",
            min: 0,
            step: "any",
            endAdornment: "px",
          }}
          onSubmit={(value): void => {
            const height = Number.parseFloat(value);
            if (!Number.isFinite(height) || height < 0) {
              return;
            }
            const view = model.getSelectedView();
            const rowStart = view.range[0];
            const rowEnd = view.range[2];
            model.setRowsHeight(view.sheet, rowStart, rowEnd, height);
            worksheetCanvas.current?.renderSheet();
          }}
        />
      </div>
    );
  },
);

export default Worksheet;
