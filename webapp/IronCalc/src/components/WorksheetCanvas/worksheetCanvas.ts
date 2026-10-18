import type { CellLink, CellStyle, MergedCell, Model } from "@ironcalc/wasm";
import { columnNameFromNumber } from "@ironcalc/wasm";
import { getColor } from "../Editor/util";
import type { Cell } from "../types";
import type { WorkbookState } from "../workbookState";
import { Axis } from "./axis";
import { CellLinks, type LinkHoverCell } from "./cellLinks";
import {
  drawBorder,
  drawBorderLine,
  ICON_AREA_WIDTH,
  renderDataBar,
  renderIcon,
  renderRating,
} from "./cfRenderer";
import {
  cellPadding,
  headerColumnWidth,
  headerRowHeight,
  LAST_COLUMN,
  LAST_ROW,
} from "./constants";
import { attachOutlineHandle } from "./outlineHandle";
import {
  computeWrappedLines,
  hexToRGBA10Percent,
  readThemeFromCSS,
  type Theme,
} from "./util";

export interface CanvasSettings {
  model: Model;
  width: number;
  height: number;
  workbookState: WorkbookState;
  elements: {
    canvas: HTMLCanvasElement;
    cellOutline: HTMLDivElement;
    areaOutline: HTMLDivElement;
    cellArrayStructure: HTMLDivElement;
    extendToOutline: HTMLDivElement;
    columnGuide: HTMLDivElement;
    rowGuide: HTMLDivElement;
    columnHeaders: HTMLDivElement;
    editor: HTMLDivElement;
  };
  onColumnWidthChanges: (sheet: number, column: number, width: number) => void;
  onRowHeightChanges: (sheet: number, row: number, height: number) => void;
  onLinkHover?: (cell: LinkHoverCell | null) => void;
  onHideLinkTooltip?: () => void;
  // Called with the engine error when an autofill (fill handle drag or
  // double-click) is rejected
  onAutofillError?: (message: string) => void;
  linkTooltipCell: LinkHoverCell | null;
  refresh: () => void;
}

export const devicePixelRatio = window.devicePixelRatio || 1;

// The last row or column with any part before `edge` (an offset along the
// axis): the one holding the edge, or the one ending exactly at it
function lastIndexBefore(axis: Axis, edge: number): number {
  const index = axis.at(edge);
  return index > 1 && axis.start(index) >= edge ? index - 1 : index;
}

export const frozenSeparatorWidth = 3;

interface TextProperties {
  row: number;
  column: number;
  x: number;
  y: number;
  width: number;
  height: number;
  fontSize: number;
  textColor: string;
  font: string;
  underlined: boolean;
  strike: boolean;
  lines: [string, number, number, number][];
  link: CellLink | null;
  // The pane the cell was laid out in; nothing is painted outside of it
  clip: PaneRect;
}

// Canvas rectangle (x, y, width, height) of one of the four panes
type PaneRect = [number, number, number, number];

export default class WorksheetCanvas {
  width: number;

  height: number;

  ctx: CanvasRenderingContext2D;

  canvas: HTMLCanvasElement;

  editor: HTMLDivElement;

  areaOutline: HTMLDivElement;

  cellArrayStructure: HTMLDivElement;

  cellOutline: HTMLDivElement;

  cellOutlineHandle: HTMLDivElement;

  extendToOutline: HTMLDivElement;

  workbookState: WorkbookState;

  model: Model;

  rowGuide: HTMLDivElement;

  columnHeaders: HTMLDivElement;

  columnGuide: HTMLDivElement;

  onColumnWidthChanges: (sheet: number, column: number, width: number) => void;

  onRowHeightChanges: (sheet: number, row: number, height: number) => void;

  onAutofillError?: (message: string) => void;

  refresh: () => void;

  cells: TextProperties[];
  spills: Map<string, number>;

  // Merged cells of the selected sheet, and a lookup from every cell of a
  // merged range ("row-column") to its merge. Rebuilt on every renderSheet.
  private mergedCells: MergedCell[];
  private mergeMap: Map<string, MergedCell>;
  // Merged ranges are painted once per pane per render even though every
  // visible cell of the range is a paint site (so they render when the anchor
  // is scrolled out of view). Keyed by "pane-anchorRow-anchorColumn".
  private renderedMergedStyles: Set<string>;
  private renderedMergedTexts: Set<string>;
  // The pane (frozen corner, frozen rows, frozen columns, main) whose cells
  // are being laid out or painted. A cell never draws outside its pane: a
  // merged range reaching into another pane is clipped to this one.
  private paneIndex: number;
  private paneClip: PaneRect;

  private cellLinks: CellLinks;

  // The row and column axes (sizes and offsets) of the selected sheet.
  // Rebuilt on every render, since any edit may have changed the sizes, and
  // whenever the selected sheet changes in between.
  private axes: { sheet: number; rows: Axis; columns: Axis } | null;

  // The scroll offset of the scrolling pane, in sheet pixels from the pane's
  // origin (the first row and column after the frozen ones), together with
  // the engine's top-left cell it was last synced with. The engine keeps a
  // whole cell as its view position: when it moves that cell on its own
  // (keyboard navigation, following a link, a sheet switch) the offset
  // follows it, see `getScroll`.
  private scroll: {
    sheet: number;
    topRow: number;
    leftColumn: number;
    x: number;
    y: number;
  } | null;

  theme: Theme;

  constructor(options: CanvasSettings) {
    this.model = options.model;
    this.axes = null;
    this.scroll = null;
    this.canvas = options.elements.canvas;
    this.width = options.width;
    this.height = options.height;
    this.ctx = this.setContext();
    this.workbookState = options.workbookState;
    this.editor = options.elements.editor;
    this.refresh = options.refresh;

    const rootRef = this.canvas.closest(".ic-root");
    if (!rootRef) {
      throw new Error("WorksheetCanvas must be rendered within an .ic-root");
    }
    this.theme = readThemeFromCSS(rootRef);

    this.cellOutline = options.elements.cellOutline;
    this.cellArrayStructure = options.elements.cellArrayStructure;
    this.areaOutline = options.elements.areaOutline;
    this.extendToOutline = options.elements.extendToOutline;
    this.rowGuide = options.elements.rowGuide;
    this.columnGuide = options.elements.columnGuide;
    this.columnHeaders = options.elements.columnHeaders;

    this.onColumnWidthChanges = options.onColumnWidthChanges;
    this.onRowHeightChanges = options.onRowHeightChanges;
    this.onAutofillError = options.onAutofillError;
    this.resetHeaders();
    this.cellOutlineHandle = attachOutlineHandle(this);

    // a cell marked as "spill" means its left border should be skipped
    this.spills = new Map<string, number>();
    this.cells = [];

    this.mergedCells = [];
    this.mergeMap = new Map();
    this.renderedMergedStyles = new Set();
    this.renderedMergedTexts = new Set();
    this.paneIndex = 0;
    this.paneClip = [0, 0, 0, 0];

    this.cellLinks = new CellLinks(this, {
      onLinkHover: options.onLinkHover,
      onHideTooltip: options.onHideLinkTooltip,
      tooltipCell: options.linkTooltipCell,
    });
  }

  // Follows a cell link: opens external targets in a new tab, navigates to
  // internal references
  followLink(link: CellLink): void {
    this.cellLinks.followLink(link);
  }

  // The axes of the selected sheet, built from the engine's sizes when
  // missing or stale
  private getAxes(): { rows: Axis; columns: Axis } {
    const sheet = this.model.getSelectedSheet();
    if (!this.axes || this.axes.sheet !== sheet) {
      this.axes = {
        sheet,
        rows: Axis.fromSizes(this.model.getRowSizes(sheet), LAST_ROW),
        columns: Axis.fromSizes(this.model.getColumnSizes(sheet), LAST_COLUMN),
      };
    }
    return this.axes;
  }

  // The sheet offset where the scrolling pane starts: the leading edge of the
  // first row and column after the frozen ones
  private getOrigin(): [number, number] {
    const sheet = this.model.getSelectedSheet();
    const { rows, columns } = this.getAxes();
    return [
      columns.start(this.model.getFrozenColumnsCount(sheet) + 1),
      rows.start(this.model.getFrozenRowsCount(sheet) + 1),
    ];
  }

  // The scrolling pane on the canvas (the bottom-right one)
  private getScrollPane(): PaneRect {
    return this.getPanes()[3];
  }

  // The size of the scrolling pane: the cells area less the headers and the
  // frozen panes
  getScrollPaneSize(): { width: number; height: number } {
    const [, , width, height] = this.getScrollPane();
    return { width, height };
  }

  // The scroll offset of the scrolling pane, in sheet pixels from its origin.
  // When the engine's top-left cell is not the one the offset was synced
  // with, the engine moved the view on its own and the offset follows: the
  // cell goes to the top-left corner of the pane.
  getScroll(): { x: number; y: number } {
    const view = this.model.getSelectedView();
    const { scroll } = this;
    if (
      scroll &&
      scroll.sheet === view.sheet &&
      scroll.topRow === view.top_row &&
      scroll.leftColumn === view.left_column
    ) {
      return { x: scroll.x, y: scroll.y };
    }
    const { rows, columns } = this.getAxes();
    const [originX, originY] = this.getOrigin();
    const frozenRows = this.model.getFrozenRowsCount(view.sheet);
    const frozenColumns = this.model.getFrozenColumnsCount(view.sheet);
    const x =
      columns.start(Math.max(view.left_column, frozenColumns + 1)) - originX;
    const y = rows.start(Math.max(view.top_row, frozenRows + 1)) - originY;
    this.scroll = {
      sheet: view.sheet,
      topRow: view.top_row,
      leftColumn: view.left_column,
      x,
      y,
    };
    return { x, y };
  }

  // The largest scroll offset: the end of the sheet at the end of the pane
  getMaxScroll(): { x: number; y: number } {
    const { rows, columns } = this.getAxes();
    const [originX, originY] = this.getOrigin();
    const [, , paneWidth, paneHeight] = this.getScrollPane();
    return {
      x: Math.max(0, columns.total() - originX - paneWidth),
      y: Math.max(0, rows.total() - originY - paneHeight),
    };
  }

  // Scrolls the pane to the offset (clamped to the sheet). The engine's
  // top-left cell follows: the row and column at the offset.
  setScroll(x: number, y: number): void {
    const { rows, columns } = this.getAxes();
    const [originX, originY] = this.getOrigin();
    const max = this.getMaxScroll();
    const sheet = this.model.getSelectedSheet();
    const frozenRows = this.model.getFrozenRowsCount(sheet);
    const frozenColumns = this.model.getFrozenColumnsCount(sheet);
    const scrollX = Math.min(Math.max(x, 0), max.x);
    const scrollY = Math.min(Math.max(y, 0), max.y);
    const topRow = Math.max(rows.at(originY + scrollY), frozenRows + 1);
    const leftColumn = Math.max(
      columns.at(originX + scrollX),
      frozenColumns + 1,
    );
    this.model.setTopLeftVisibleCell(topRow, leftColumn);
    this.scroll = { sheet, topRow, leftColumn, x: scrollX, y: scrollY };
  }

  // Scrolls to the DOM scroller's position, measured from the pane's origin
  setScrollPosition(scrollPosition: { left: number; top: number }): void {
    this.setScroll(scrollPosition.left, scrollPosition.top);
  }

  // Scrolls as little as needed for the cell (its merged range, if it is in
  // one) to be fully inside the scrolling pane. Cells in the frozen rows or
  // columns are always on screen along that axis. Returns whether the
  // offset changed.
  reveal(row: number, column: number): boolean {
    const sheet = this.model.getSelectedSheet();
    const frozenRows = this.model.getFrozenRowsCount(sheet);
    const frozenColumns = this.model.getFrozenColumnsCount(sheet);
    const { rows, columns } = this.getAxes();
    const [originX, originY] = this.getOrigin();
    const [, , paneWidth, paneHeight] = this.getScrollPane();
    // The merges are only known after a render; a reveal can come first
    this.updateMergedCells();
    const merge = this.mergeMap.get(`${row}-${column}`);
    const firstRow = merge ? merge.row : row;
    const firstColumn = merge ? merge.column : column;
    const lastRow = merge ? merge.row + merge.height - 1 : row;
    const lastColumn = merge ? merge.column + merge.width - 1 : column;
    let { x, y } = this.getScroll();
    if (lastColumn > frozenColumns) {
      const left =
        columns.start(Math.max(firstColumn, frozenColumns + 1)) - originX;
      const right = columns.start(lastColumn + 1) - originX;
      if (left < x) {
        x = left;
      } else if (right > x + paneWidth) {
        x = Math.min(Math.max(right - paneWidth, 0), left);
      }
    }
    if (lastRow > frozenRows) {
      const top = rows.start(Math.max(firstRow, frozenRows + 1)) - originY;
      const bottom = rows.start(lastRow + 1) - originY;
      if (top < y) {
        y = top;
      } else if (bottom > y + paneHeight) {
        y = Math.min(Math.max(bottom - paneHeight, 0), top);
      }
    }
    const before = this.getScroll();
    if (x === before.x && y === before.y) {
      return false;
    }
    this.setScroll(x, y);
    return true;
  }

  resetHeaders(): void {
    for (const handle of this.columnHeaders.querySelectorAll(
      ".column-resize-handle",
    )) {
      handle.remove();
    }
    for (const columnSeparator of this.columnHeaders.querySelectorAll(
      ".frozen-column-separator",
    )) {
      columnSeparator.remove();
    }
    for (const wrapper of this.columnHeaders.querySelectorAll(
      ".column-headers-scroll",
    )) {
      wrapper.remove();
    }
    for (const header of this.columnHeaders.children) {
      (header as HTMLDivElement).classList.add("column-header");
    }
  }

  setContext(): CanvasRenderingContext2D {
    const { canvas, width, height } = this;
    const context = canvas.getContext("2d");
    if (!context) {
      throw new Error(
        "This browser does not support 2-dimensional canvas rendering contexts.",
      );
    }
    // If the devicePixelRatio is 2 then the canvas is twice as large to avoid blurring.
    canvas.width = width * devicePixelRatio;
    canvas.height = height * devicePixelRatio;
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    context.scale(devicePixelRatio, devicePixelRatio);
    return context;
  }

  setSize(size: { width: number; height: number }): void {
    this.width = size.width;
    this.height = size.height;
    this.ctx = this.setContext();
  }

  /**
   * This is the height of the frozen rows including the width of the separator
   * It returns 0 if the are no frozen rows
   */
  getFrozenRowsHeight(): number {
    const frozenRows = this.model.getFrozenRowsCount(
      this.model.getSelectedSheet(),
    );
    if (frozenRows === 0) {
      return 0;
    }
    return this.getAxes().rows.start(frozenRows + 1) + frozenSeparatorWidth;
  }

  /**
   * This is the width of the frozen columns including the width of the separator
   * It returns 0 if the are no frozen columns
   */
  getFrozenColumnsWidth(): number {
    const frozenColumns = this.model.getFrozenColumnsCount(
      this.model.getSelectedSheet(),
    );
    if (frozenColumns === 0) {
      return 0;
    }
    return (
      this.getAxes().columns.start(frozenColumns + 1) + frozenSeparatorWidth
    );
  }

  // Get the visible cells (aside from the frozen rows and columns)
  // The four panes in paint order: frozen corner (top-left), frozen rows
  // (top-right), frozen columns (bottom-left) and the scrolling sheet
  // (bottom-right). Separators are left out of every pane.
  private getPanes(): PaneRect[] {
    const frozenColumnsWidth = this.getFrozenColumnsWidth();
    const frozenRowsHeight = this.getFrozenRowsHeight();
    const x0 = headerColumnWidth;
    const y0 = headerRowHeight;
    const frozenX = x0 + frozenColumnsWidth;
    const frozenY = y0 + frozenRowsHeight;
    // getFrozen*() include the separator when there are frozen rows/columns
    const cornerWidth = Math.max(0, frozenColumnsWidth - frozenSeparatorWidth);
    const cornerHeight = Math.max(0, frozenRowsHeight - frozenSeparatorWidth);
    return [
      [x0, y0, cornerWidth, cornerHeight],
      [frozenX, y0, this.width - frozenX, cornerHeight],
      [x0, frozenY, cornerWidth, this.height - frozenY],
      [frozenX, frozenY, this.width - frozenX, this.height - frozenY],
    ];
  }

  private setPane(index: number, panes: PaneRect[]): void {
    this.paneIndex = index;
    this.paneClip = panes[index];
  }

  // Restricts canvas drawing to the cells (everything but the headers) until
  // the matching context.restore()
  private clipToCells(): void {
    this.ctx.save();
    this.ctx.beginPath();
    this.ctx.rect(
      headerColumnWidth,
      headerRowHeight,
      this.width - headerColumnWidth,
      this.height - headerRowHeight,
    );
    this.ctx.clip();
  }

  // Selects the pane and restricts canvas drawing to it until the matching
  // context.restore()
  private clipToPane(index: number, panes: PaneRect[]): void {
    this.setPane(index, panes);
    const [x, y, width, height] = this.paneClip;
    this.ctx.save();
    this.ctx.beginPath();
    this.ctx.rect(x, y, width, height);
    this.ctx.clip();
  }

  // The cells of the scrolling pane with any part on screen: from the row and
  // column at the scroll offset to the ones holding the pane's bottom-right
  // corner
  getVisibleCells(): {
    topLeftCell: Cell;
    bottomRightCell: Cell;
  } {
    const selectedSheet = this.model.getSelectedSheet();
    const frozenRows = this.model.getFrozenRowsCount(selectedSheet);
    const frozenColumns = this.model.getFrozenColumnsCount(selectedSheet);
    const { rows, columns } = this.getAxes();
    const [originX, originY] = this.getOrigin();
    const { x, y } = this.getScroll();
    const [, , paneWidth, paneHeight] = this.getScrollPane();
    const rowTop = Math.max(frozenRows + 1, rows.at(originY + y));
    const columnLeft = Math.max(frozenColumns + 1, columns.at(originX + x));
    return {
      topLeftCell: { row: rowTop, column: columnLeft },
      bottomRightCell: {
        row: Math.max(rowTop, lastIndexBefore(rows, originY + y + paneHeight)),
        column: Math.max(
          columnLeft,
          lastIndexBefore(columns, originX + x + paneWidth),
        ),
      },
    };
  }

  private getFontStyle(style: CellStyle): {
    font: string;
    color: string;
    fontSize: number;
  } {
    const fontSize = style.font?.sz || 13;
    let font = `${fontSize}px ${this.theme.cellFontFamily}`;
    let color = style.fill.color
      ? this.theme.filledCellTextColor
      : this.theme.defaultTextColor;

    if (style.font) {
      // Font.color is optional: a missing color means "use the default" — fall back
      // to the theme's default text color rather than leaving it undefined.
      if (style.font.color) {
        color = this.model.resolveColor(style.font.color);
      }
      font = style.font.b ? `bold ${font}` : `400 ${font}`;
      if (style.font.i) {
        font = `italic ${font}`;
      }
    }

    return { font, color, fontSize };
  }

  private getAlignment(
    style: CellStyle,
    cellType: number,
  ): { horizontal: string; vertical: string } {
    let horizontal = style.alignment?.horizontal || "general";
    const vertical = style.alignment?.vertical || "bottom";

    if (horizontal === "general") {
      if (cellType === 1) {
        horizontal = "right";
      } else if (cellType === 4) {
        horizontal = "center";
      } else {
        horizontal = "left";
      }
    }

    return { horizontal, vertical };
  }

  // Computes the text of cells that are off the screen. They are important because their text may spill into the viewport.
  private computeCellTextLeftRight(
    selectedSheet: number,
    row: number,
    rowHeight: number,
    topLeftCell: Cell,
    bottomRightCell: Cell,
  ): void {
    const frozenColumnsCount = this.model.getFrozenColumnsCount(selectedSheet);
    // First compute the text to the left of the viewport
    const columnToTheLeft = this.model.getLastNonEmptyInRowBeforeColumn(
      selectedSheet,
      row,
      topLeftCell.column,
    );
    // If it is one of the frozen columns it is already computed
    if (columnToTheLeft && columnToTheLeft > frozenColumnsCount) {
      const columnWidth = this.getColumnWidth(selectedSheet, columnToTheLeft);
      const [textX, textY] = this.getCoordinatesByCell(row, columnToTheLeft);
      this.computeCellText(
        row,
        columnToTheLeft,
        textX,
        textY,
        columnWidth,
        rowHeight,
      );
    }
    // Compute the text to the right of the viewport
    const columnToTheRight = this.model.getFirstNonEmptyInRowAfterColumn(
      selectedSheet,
      row,
      bottomRightCell.column,
    );

    if (columnToTheRight) {
      const columnWidth = this.getColumnWidth(selectedSheet, columnToTheRight);
      const [textX, textY] = this.getCoordinatesByCell(row, columnToTheRight);
      this.computeCellText(
        row,
        columnToTheRight,
        textX,
        textY,
        columnWidth,
        rowHeight,
      );
    }
  }

  // Rebuilds the merged-cell lookup for the selected sheet
  private updateMergedCells(): void {
    this.renderedMergedStyles = new Set();
    this.renderedMergedTexts = new Set();
    const selectedSheet = this.model.getSelectedSheet();
    this.mergedCells = this.model.getMergedCells(selectedSheet);
    this.mergeMap = new Map();
    for (const merge of this.mergedCells) {
      for (let row = merge.row; row < merge.row + merge.height; row += 1) {
        for (
          let column = merge.column;
          column < merge.column + merge.width;
          column += 1
        ) {
          this.mergeMap.set(`${row}-${column}`, merge);
        }
      }
    }
  }

  // Returns the canvas rectangle (x, y, width, height) of a merged range
  private getMergedRect(merge: MergedCell): [number, number, number, number] {
    const selectedSheet = this.model.getSelectedSheet();
    const [x, y] = this.getCoordinatesByCell(merge.row, merge.column);
    let width = 0;
    for (
      let column = merge.column;
      column < merge.column + merge.width;
      column += 1
    ) {
      width += this.getColumnWidth(selectedSheet, column);
    }
    let height = 0;
    for (let row = merge.row; row < merge.row + merge.height; row += 1) {
      height += this.getRowHeight(selectedSheet, row);
    }
    return [x, y, width, height];
  }

  // Same rectangle, but positioned relative to the cell (row, column) of the
  // range being painted at (x, y) so it lines up exactly with the grid drawn
  // by the render loop of the current pane
  private getMergedRectFrom(
    merge: MergedCell,
    row: number,
    column: number,
    x: number,
    y: number,
  ): [number, number, number, number] {
    const selectedSheet = this.model.getSelectedSheet();
    const [, , width, height] = this.getMergedRect(merge);
    let mergeX = x;
    for (let c = merge.column; c < column; c += 1) {
      mergeX -= this.getColumnWidth(selectedSheet, c);
    }
    let mergeY = y;
    for (let r = merge.row; r < row; r += 1) {
      mergeY -= this.getRowHeight(selectedSheet, r);
    }
    return [mergeX, mergeY, width, height];
  }

  // Goes through all the visible cells and computes their text properties
  private computeCellsText(): void {
    const { topLeftCell, bottomRightCell } = this.getVisibleCells();
    const selectedSheet = this.model.getSelectedSheet();

    this.cells = [];

    this.cellLinks.setSheetLinks(this.model.getLinks(selectedSheet));

    const frozenColumns = this.model.getFrozenColumnsCount(selectedSheet);
    const frozenRows = this.model.getFrozenRowsCount(selectedSheet);
    const panes = this.getPanes();

    // Top-left-pane
    this.setPane(0, panes);
    let x = headerColumnWidth + 0.5;
    let y = headerRowHeight + 0.5;
    for (let row = 1; row <= frozenRows; row += 1) {
      const rowHeight = this.getRowHeight(selectedSheet, row);
      x = headerColumnWidth + 0.5;
      for (let column = 1; column <= frozenColumns; column += 1) {
        const columnWidth = this.getColumnWidth(selectedSheet, column);
        this.computeCellText(row, column, x, y, columnWidth, rowHeight);

        x += columnWidth;
      }
      y += rowHeight;
    }
    if (frozenRows === 0 && frozenColumns !== 0) {
      x = headerColumnWidth + 0.5;
      for (let column = 1; column <= frozenColumns; column += 1) {
        x += this.getColumnWidth(selectedSheet, column);
      }
    }

    // If there are frozen rows draw a separator
    if (frozenRows) {
      y += frozenSeparatorWidth;
    }

    // If there are frozen columns draw a separator
    if (frozenColumns) {
      x += frozenSeparatorWidth;
    }

    // The scrolling pane's first row and column start at the scroll offset:
    // partly out of view when it is not on a cell boundary
    const [scrollPaneX, scrollPaneY] = this.getCoordinatesByCell(
      topLeftCell.row,
      topLeftCell.column,
    );
    const scrollX0 = scrollPaneX + 0.5;
    const scrollY0 = scrollPaneY + 0.5;
    // Top-right pane
    this.setPane(1, panes);
    y = headerRowHeight + 0.5;
    for (let row = 1; row <= frozenRows; row += 1) {
      x = scrollX0;
      const rowHeight = this.getRowHeight(selectedSheet, row);
      // Cells outside the viewport whose text may spill into it belong to
      // this pane (not the frozen corner): their text must clip to it
      this.computeCellTextLeftRight(
        selectedSheet,
        row,
        rowHeight,
        topLeftCell,
        bottomRightCell,
      );
      for (
        let { column } = topLeftCell;
        column <= bottomRightCell.column;
        column += 1
      ) {
        const columnWidth = this.getColumnWidth(selectedSheet, column);
        this.computeCellText(row, column, x, y, columnWidth, rowHeight);

        x += columnWidth;
      }
      y += rowHeight;
    }

    // Bottom-left pane
    this.setPane(2, panes);
    y = scrollY0;
    for (let { row } = topLeftCell; row <= bottomRightCell.row; row += 1) {
      x = headerColumnWidth;
      const rowHeight = this.getRowHeight(selectedSheet, row);

      for (let column = 1; column <= frozenColumns; column += 1) {
        const columnWidth = this.getColumnWidth(selectedSheet, column);
        this.computeCellText(row, column, x, y, columnWidth, rowHeight);

        x += columnWidth;
      }
      y += rowHeight;
    }

    // Bottom-right pane
    this.setPane(3, panes);
    y = scrollY0;
    for (let { row } = topLeftCell; row <= bottomRightCell.row; row += 1) {
      x = scrollX0;
      const rowHeight = this.getRowHeight(selectedSheet, row);

      // Cells outside the viewport whose text may spill into it belong to
      // this pane (not the frozen columns): their text must clip to it
      this.computeCellTextLeftRight(
        selectedSheet,
        row,
        rowHeight,
        topLeftCell,
        bottomRightCell,
      );
      for (
        let { column } = topLeftCell;
        column <= bottomRightCell.column;
        column += 1
      ) {
        const columnWidth = this.getColumnWidth(selectedSheet, column);
        this.computeCellText(row, column, x, y, columnWidth, rowHeight);

        x += columnWidth;
      }
      y += rowHeight;
    }
  }

  // Compute the text properties for a cell. Every cell of a merged range is a
  // paint site for the whole range (drawn once, from the anchor, over the full
  // merged rectangle), so the range still renders when its anchor is scrolled
  // out of view.
  private computeCellText(
    row: number,
    column: number,
    x: number,
    y: number,
    width: number,
    height: number,
  ) {
    const merge = this.mergeMap.get(`${row}-${column}`);
    if (merge) {
      const key = `${this.paneIndex}-${merge.row}-${merge.column}`;
      if (this.renderedMergedTexts.has(key)) {
        return;
      }
      this.renderedMergedTexts.add(key);
      const [mergeX, mergeY, mergeWidth, mergeHeight] = this.getMergedRectFrom(
        merge,
        row,
        column,
        x,
        y,
      );
      this.computeCellTextImpl(
        merge.row,
        merge.column,
        mergeX,
        mergeY,
        mergeWidth,
        mergeHeight,
        true,
      );
      return;
    }
    this.computeCellTextImpl(row, column, x, y, width, height, false);
  }

  private computeCellTextImpl(
    row: number,
    column: number,
    x: number,
    y: number,
    width: number,
    height: number,
    isMergedCell: boolean,
  ) {
    if (width <= 0 || height <= 0) {
      return;
    }
    const selectedSheet = this.model.getSelectedSheet();

    const extended = this.model.getCellStyle(selectedSheet, row, column);
    const style = extended.style;

    // Suppress text when the CF rule hides the value
    if (extended.icon && !extended.icon.show_value) {
      return;
    }
    if (extended.data_bar && !extended.data_bar.show_value) {
      return;
    }
    if (extended.rating && !extended.rating.show_value) {
      return;
    }

    const { font, color: textColor, fontSize } = this.getFontStyle(style);

    // The link is only used for the hover tooltip: it does not affect how the
    // cell looks. The link styling (color, underline) is part of the cell style.
    const link = this.cellLinks.getLink(row, column);

    // Number = 1,
    // Text = 2,
    // LogicalValue = 4,
    // ErrorValue = 16,
    // Array = 64,
    // CompoundData = 128,
    const cellType = this.model.getCellType(selectedSheet, row, column);
    const { horizontal: horizontalAlign, vertical: verticalAlign } =
      this.getAlignment(style, cellType);

    const wrapText = style.alignment?.wrap_text || false;

    const context = this.ctx;
    context.font = font;
    context.fillStyle = textColor;
    const fullText = this.model.getFormattedCellValue(
      selectedSheet,
      row,
      column,
    );

    // Is there any better to determine the line height?
    const lineHeight = fontSize * 1.5;
    const lines = computeWrappedLines(
      fullText,
      wrapText,
      context,
      width - cellPadding,
    );
    const lineCount = lines.length;
    let maxWidth = 0;
    // When an icon is shown alongside the value, shift text right by the icon area.
    const ratingIconOffset = extended.rating?.show_value
      ? extended.rating.count * (style.font?.sz ? style.font.sz * 1.2 : 16)
      : 0;
    const iconOffset = extended.icon?.show_value
      ? ICON_AREA_WIDTH
      : ratingIconOffset;
    let minX = x + iconOffset;
    let maxX = x + width;
    const textProperties = {
      row,
      column,
      x: minX,
      y,
      width: width - iconOffset,
      height,
      fontSize,
      textColor,
      font,
      underlined: style.font?.u || false,
      strike: style.font?.strike || false,
      lines: [] as [string, number, number, number][],
      link,
      clip: this.paneClip,
    };

    lines.forEach((text, line) => {
      const textWidth = context.measureText(text).width;
      let textX: number;
      let textY: number;
      // The idea is that in the present font-size and default row heigh,
      // top/bottom and center horizontalAlign coincide
      const verticalPadding = 6.5; // default row height is 25 and font size is 13, so (25 - 13) / 2
      if (horizontalAlign === "right") {
        textX = width - cellPadding + x - textWidth / 2;
      } else if (horizontalAlign === "center") {
        textX = x + width / 2;
      } else {
        // left aligned
        textX = cellPadding + x + textWidth / 2;
      }
      if (verticalAlign === "bottom") {
        textY =
          y +
          height -
          fontSize / 2 -
          verticalPadding +
          (line - lineCount + 1) * lineHeight;
      } else if (verticalAlign === "center") {
        textY = y + height / 2 + (line + (1 - lineCount) / 2) * lineHeight;
      } else {
        // aligned top
        textY = y + fontSize / 2 + verticalPadding + line * lineHeight;
      }
      textProperties.lines.push([text, textX, textY, textWidth]);
      minX = Math.min(minX, textX - textWidth / 2);
      maxX = Math.max(maxX, textX + textWidth / 2);
      maxWidth = Math.max(maxWidth, textX + textWidth / 2 - x);
    });
    // we need to see if the text spills to the right of the cell
    // (the text of a merged cell never overflows the merged rectangle, and
    // text never overflows into a merged range)
    let leftColumnX = x;
    let rightColumnX = x + width;
    if (
      !isMergedCell &&
      maxX > rightColumnX &&
      column < LAST_COLUMN &&
      (this.model.getFormattedCellValue(selectedSheet, row, column + 1) ===
        "" ||
        this.getColumnWidth(selectedSheet, column + 1) === 0)
    ) {
      let spillColumn = column + 1;
      // Keep expanding the spill to the right until:
      // 1. There is a non-empty cell (skipping hidden columns)
      // 2. Reaches the end of the row
      // 3. There is the end of frozen columns
      const frozenColumns = this.model.getFrozenColumnsCount(selectedSheet);
      while (
        rightColumnX < maxX &&
        !this.mergeMap.has(`${row}-${spillColumn}`) &&
        (this.model.getFormattedCellValue(selectedSheet, row, spillColumn) ===
          "" ||
          this.getColumnWidth(selectedSheet, spillColumn) === 0) &&
        spillColumn <= LAST_COLUMN &&
        ((column < frozenColumns && spillColumn <= frozenColumns) ||
          column > frozenColumns)
      ) {
        rightColumnX += this.getColumnWidth(selectedSheet, spillColumn);
        // marks (row, spillColumn) as spilling so we don't draw a border to the left
        this.spills.set(`${row}-${spillColumn}`, 1);
        spillColumn += 1;
      }
    }
    // Same thing in the other direction, to the left of the cell
    const frozenColumnsCount = this.model.getFrozenColumnsCount(selectedSheet);
    if (
      !isMergedCell &&
      minX < leftColumnX &&
      column > 1 &&
      (this.model.getFormattedCellValue(selectedSheet, row, column - 1) ===
        "" ||
        this.getColumnWidth(selectedSheet, column - 1) === 0)
    ) {
      let spillColumn = column - 1;
      // Keep expanding the spill to the left until:
      // 1. There is a non-empty cell
      // 2. Reaches the beginning of the row
      // 3. There is the end of frozen columns
      while (
        leftColumnX > minX &&
        !this.mergeMap.has(`${row}-${spillColumn}`) &&
        (this.model.getFormattedCellValue(selectedSheet, row, spillColumn) ===
          "" ||
          this.getColumnWidth(selectedSheet, spillColumn) === 0) &&
        spillColumn >= 1 &&
        ((column <= frozenColumnsCount && spillColumn <= frozenColumnsCount) ||
          column > frozenColumnsCount)
      ) {
        leftColumnX -= this.getColumnWidth(selectedSheet, spillColumn);
        // This is tricky but correct. The reason is we only draw the left borders of the cells
        // (because the left border of a cell MUST be the right border of the one to the left).
        // So if we want to remove the right border of this cell we need to skip the left border of the next.
        this.spills.set(`${row}-${spillColumn + 1}`, 1);
        spillColumn -= 1;
      }
    }

    if (frozenColumnsCount > 0) {
      const frozenColumnsX =
        this.getCoordinatesByCell(row, frozenColumnsCount)[0] +
        this.getColumnWidth(selectedSheet, frozenColumnsCount);
      if (column > frozenColumnsCount) {
        leftColumnX = Math.max(leftColumnX, frozenColumnsX);
      } else {
        rightColumnX = Math.min(rightColumnX, frozenColumnsX);
      }
    }
    textProperties.x = leftColumnX;
    textProperties.width = rightColumnX - leftColumnX;
    this.cells.push(textProperties);
  }

  /// Renders the cell style: background color, CF overlays, and borders (but not text).
  /// A merged range is painted once, from its anchor, over the whole merged
  /// rectangle: interior gridlines and borders are thereby suppressed, while
  /// the perimeter still renders (the right and bottom edges are drawn by the
  /// neighbouring cells, which consult the styles of the covered edge cells).
  private renderCellStyle(
    row: number,
    column: number,
    x: number,
    y: number,
    width: number,
    height: number,
  ): void {
    const merge = this.mergeMap.get(`${row}-${column}`);
    if (merge) {
      const key = `${this.paneIndex}-${merge.row}-${merge.column}`;
      if (this.renderedMergedStyles.has(key)) {
        // The merged rectangle was painted when its first cell was reached,
        // before the cells to the left of the lower rows. Their background
        // fill overlaps the left edge, so it is redrawn once the loop reaches
        // each cell of the left column (its left neighbour is painted by then).
        if (column === merge.column) {
          this.renderLeftBorder(row, column, x, y, height);
        }
        return;
      }
      this.renderedMergedStyles.add(key);
      const [mergeX, mergeY, mergeWidth, mergeHeight] = this.getMergedRectFrom(
        merge,
        row,
        column,
        x,
        y,
      );
      this.renderCellStyleImpl(
        merge.row,
        merge.column,
        mergeX,
        mergeY,
        mergeWidth,
        mergeHeight,
        merge,
      );
      return;
    }
    this.renderCellStyleImpl(row, column, x, y, width, height);
  }

  private renderCellStyleImpl(
    row: number,
    column: number,
    x: number,
    y: number,
    width: number,
    height: number,
    merge?: MergedCell,
  ): void {
    if (width <= 0 || height <= 0) {
      return;
    }
    const selectedSheet = this.model.getSelectedSheet();
    const extended = this.model.getCellStyle(selectedSheet, row, column);
    const style = extended.style;

    let backgroundColor = this.theme.backgroundColor;
    if (style.fill.color) {
      backgroundColor = this.model.resolveColor(style.fill.color);
    }
    const context = this.ctx;
    context.fillStyle = backgroundColor;
    context.fillRect(x + 0.5, y + 0.5, width - 1, height - 1);

    // CF overlays rendered on top of the background.
    if (extended.data_bar) {
      renderDataBar(context, x, y, width, height, extended.data_bar, (c) =>
        this.model.resolveColor(c),
      );
    }
    if (extended.icon) {
      const iconSize = style.font?.sz ? style.font.sz * 1.2 : 16;
      renderIcon(context, x, y, height, extended.icon, iconSize, (c) =>
        this.model.resolveColor(c),
      );
    }
    if (extended.rating) {
      const iconSize = style.font?.sz ? style.font.sz * 1.2 : 16;
      renderRating(context, x, y, height, extended.rating, iconSize, (c) =>
        this.model.resolveColor(c),
      );
    }

    // Borders — we only draw left and top; the adjacent cell draws right/bottom.
    // Borders are stored per cell, so the left edge of a
    // merged range is drawn one segment per row and the top edge one segment
    // per column, each consulting its own perimeter cell and neighbour.
    if (!merge) {
      this.renderLeftBorder(row, column, x, y, height);
      this.renderTopBorder(row, column, x, y, width);
      return;
    }
    let segmentY = y;
    for (let r = merge.row; r < merge.row + merge.height; r += 1) {
      const rowHeight = this.getRowHeight(selectedSheet, r);
      this.renderLeftBorder(r, merge.column, x, segmentY, rowHeight);
      segmentY += rowHeight;
    }
    let segmentX = x;
    for (let c = merge.column; c < merge.column + merge.width; c += 1) {
      const columnWidth = this.getColumnWidth(selectedSheet, c);
      this.renderTopBorder(merge.row, c, segmentX, y, columnWidth);
      segmentX += columnWidth;
    }
  }

  // The fill color a cell shows on screen: a cell covered by a merged range
  // is painted with the fill of the merge anchor, whatever its own style says
  // (merging stamps the anchor's style on the covered cells, but merges from
  // imported files may predate that invariant). Returns the resolved color,
  // or null when the effective style has no fill.
  private getEffectiveFillColor(row: number, column: number): string | null {
    const merge = this.mergeMap.get(`${row}-${column}`);
    const [r, c] = merge ? [merge.row, merge.column] : [row, column];
    const style = this.model.getCellStyle(
      this.model.getSelectedSheet(),
      r,
      c,
    ).style;
    return style.fill.color ? this.model.resolveColor(style.fill.color) : null;
  }

  // Draws the left border of cell (row, column) from (x, y) to (x, y + height)
  // Algorithm:
  //  * we use the border if present
  //  * otherwise we use the border of the adjacent cell
  //  * otherwise we use the color of the background
  //  * otherwise we use the background color of the adjacent cell
  //  * if everything else fails we use the default grid color
  // Backgrounds are the effective (merge-aware) fills; borders are read from
  // the cells themselves — a merged range's perimeter borders come from its
  // perimeter cells.
  private renderLeftBorder(
    row: number,
    column: number,
    x: number,
    y: number,
    height: number,
  ): void {
    const selectedSheet = this.model.getSelectedSheet();
    const style = this.model.getCellStyle(selectedSheet, row, column).style;
    const leftStyle = this.model.getCellStyle(
      selectedSheet,
      row,
      column - 1,
    ).style;
    const context = this.ctx;
    const border = style.border;

    const fillColor = this.getEffectiveFillColor(row, column);
    const leftFillColor = this.getEffectiveFillColor(row, column - 1);

    // Skip the left border for spill cells.
    if (this.spills.get(`${row}-${column}`) !== 1) {
      let borderLeftColor = this.getCellGridColor(selectedSheet, fillColor);
      let borderLeftStyle = "thin";
      if (border.left) {
        borderLeftColor =
          this.model.resolveColor(border.left.color) ||
          this.theme.defaultTextColor;
        borderLeftStyle = border.left.style;
      } else if (leftStyle.border.right) {
        borderLeftColor =
          this.model.resolveColor(leftStyle.border.right.color) ||
          this.theme.defaultTextColor;
        borderLeftStyle = leftStyle.border.right.style;
      } else if (fillColor) {
        borderLeftColor = fillColor;
      } else if (leftFillColor) {
        borderLeftColor = leftFillColor;
      }

      // The left border of the first column is shared with the header separator
      if (x > headerColumnWidth + 0.51 || border.left) {
        drawBorder(
          context,
          borderLeftStyle,
          borderLeftColor,
          x,
          y,
          x,
          y + height,
          true,
        );
      }
    } else {
      // If the cell spills to the right we don't want to set the border but we still need to fill
      // the border gap with the background color
      let borderLeftColor = this.theme.backgroundColor;
      if (fillColor) {
        borderLeftColor = fillColor;
      } else if (leftFillColor) {
        borderLeftColor = leftFillColor;
      }
      if (borderLeftColor) {
        drawBorder(context, "thin", borderLeftColor, x, y, x, y + height, true);
      }
    }
  }

  // Draws the top border of cell (row, column) from (x, y) to (x + width, y)
  // Same algorithm as renderLeftBorder
  private renderTopBorder(
    row: number,
    column: number,
    x: number,
    y: number,
    width: number,
  ): void {
    const selectedSheet = this.model.getSelectedSheet();
    const style = this.model.getCellStyle(selectedSheet, row, column).style;
    const topStyle = this.model.getCellStyle(
      selectedSheet,
      row - 1,
      column,
    ).style;
    const border = style.border;
    const fillColor = this.getEffectiveFillColor(row, column);
    const topFillColor = this.getEffectiveFillColor(row - 1, column);

    let borderTopColor = this.getCellGridColor(selectedSheet, fillColor);
    let borderTopStyle = "thin";
    if (border.top) {
      borderTopColor =
        this.model.resolveColor(border.top.color) ||
        this.theme.defaultTextColor;
      borderTopStyle = border.top.style;
    } else if (topStyle.border.bottom) {
      borderTopColor =
        this.model.resolveColor(topStyle.border.bottom.color) ||
        this.theme.defaultTextColor;
      borderTopStyle = topStyle.border.bottom.style;
    } else if (fillColor) {
      borderTopColor = fillColor;
    } else if (topFillColor) {
      borderTopColor = topFillColor;
    }
    // The top border of the first row is shared with the header separator
    if (y > headerRowHeight + 0.51 || border.top) {
      drawBorder(
        this.ctx,
        borderTopStyle,
        borderTopColor,
        x,
        y,
        x + width,
        y,
        false,
      );
    }
  }

  // Grid line color for a cell: the theme grid color, or the cell's
  // effective background when grid lines are hidden
  private getCellGridColor(sheet: number, fillColor: string | null): string {
    if (this.model.getShowGridLines(sheet)) {
      return this.theme.gridColor;
    }
    return fillColor ?? this.theme.backgroundColor;
  }

  /// Renders the text in the cell.
  private renderCellText(textProperties: TextProperties) {
    const {
      x,
      y,
      width,
      height,
      font,
      underlined,
      strike,
      fontSize,
      textColor,
      lines,
      clip,
    } = textProperties;
    const context = this.ctx;

    context.font = font;
    context.fillStyle = textColor;
    // Clip to the cell (or spill) rectangle, and to the pane it belongs to
    context.save();
    context.beginPath();
    context.rect(x, y, width, height);
    context.clip();
    context.beginPath();
    context.rect(clip[0], clip[1], clip[2], clip[3]);
    context.clip();

    lines.forEach((line, _) => {
      const [textContent, textX, textY, textWidth] = line;
      context.fillText(textContent, textX, textY);

      if (underlined) {
        // There are no text-decoration in canvas. You have to do the underline yourself.
        const offset = Math.floor(fontSize / 2);
        context.beginPath();
        context.strokeStyle = textColor;
        context.lineWidth = 1;
        context.moveTo(textX - textWidth / 2, textY + offset);
        context.lineTo(textX + textWidth / 2, textY + offset);
        context.stroke();
      }
      if (strike) {
        // There are no text-decoration in canvas. You have to do the strikethrough yourself.
        context.beginPath();
        context.strokeStyle = textColor;
        context.lineWidth = 1;
        context.moveTo(textX - textWidth / 2, textY);
        context.lineTo(textX + textWidth / 2, textY);
        context.stroke();
      }
    });
    context.restore();
  }

  // Column and row headers with their handles
  private addColumnResizeHandle(
    x: number,
    column: number,
    columnWidth: number,
  ): void {
    const div = document.createElement("div");
    div.className = "column-resize-handle";
    div.style.left = `${x - 1}px`;
    div.style.height = `${headerRowHeight}px`;
    this.columnHeaders.insertBefore(div, null);

    let initPageX = 0;
    const resizeHandleMove = (event: MouseEvent): void => {
      if (columnWidth + event.pageX - initPageX > 0) {
        div.style.left = `${x + event.pageX - initPageX - 1}px`;
        this.columnGuide.style.left = `${
          headerColumnWidth + x + event.pageX - initPageX
        }px`;
      }
    };
    let resizeHandleUp = (event: MouseEvent): void => {
      div.style.opacity = "0";
      this.columnGuide.style.display = "none";
      document.removeEventListener("pointermove", resizeHandleMove);
      document.removeEventListener("pointerup", resizeHandleUp);
      const newColumnWidth = columnWidth + event.pageX - initPageX;
      if (newColumnWidth !== columnWidth) {
        this.onColumnWidthChanges(
          this.model.getSelectedSheet(),
          column,
          newColumnWidth,
        );
      }
    };
    resizeHandleUp = resizeHandleUp.bind(this);
    div.addEventListener("pointerdown", (event) => {
      div.style.opacity = "1";
      this.columnGuide.style.display = "block";
      this.columnGuide.style.left = `${headerColumnWidth + x}px`;
      initPageX = event.pageX;
      document.addEventListener("pointermove", resizeHandleMove);
      document.addEventListener("pointerup", resizeHandleUp);
    });

    div.addEventListener("dblclick", (event) => {
      // This is tough. We should have a call like this.model.setAutofitColumn(sheet, column)
      // but we can't do that because the back end knows nothing about the rendering engine.
      const sheet = this.model.getSelectedSheet();
      const rows = this.model.getRowsWithData(sheet, column);
      let width = 0;
      for (const row of rows) {
        const fullText = this.model.getFormattedCellValue(sheet, row, column);
        if (fullText === "") {
          continue;
        }
        const { style: cellStyle } = this.model.getCellStyle(
          sheet,
          row,
          column,
        );
        const fontSize = cellStyle.font.sz;
        let font = `${fontSize}px ${this.theme.cellFontFamily}`;
        font = cellStyle.font.b ? `bold ${font}` : `400 ${font}`;
        this.ctx.font = font;
        const lines = fullText.split("\n");
        for (const line of lines) {
          const textWidth = this.ctx.measureText(line).width;
          width = Math.max(width, textWidth);
        }
      }
      // If the width is 0, we do nothing
      if (width !== 0) {
        // The +8 is so that the text is in the same position regardless of the horizontal alignment
        this.model.setColumnsWidth(sheet, column, column, width + 8);
        this.refresh();
      }
      event.stopPropagation();
    });
  }

  private addRowResizeHandle(y: number, row: number, rowHeight: number): void {
    const div = document.createElement("div");
    div.className = "row-resize-handle";
    div.style.top = `${y - 1}px`;
    div.style.width = `${headerColumnWidth}px`;
    const sheet = this.model.getSelectedSheet();
    this.canvas.parentElement?.insertBefore(div, null);
    let initPageY = 0;
    /* istanbul ignore next */
    const resizeHandleMove = (event: MouseEvent): void => {
      if (rowHeight + event.pageY - initPageY > 0) {
        div.style.top = `${y + event.pageY - initPageY - 1}px`;
        this.rowGuide.style.top = `${y + event.pageY - initPageY}px`;
      }
    };
    let resizeHandleUp = (event: MouseEvent): void => {
      div.style.opacity = "0";
      this.rowGuide.style.display = "none";
      document.removeEventListener("pointermove", resizeHandleMove);
      document.removeEventListener("pointerup", resizeHandleUp);
      const newRowHeight = rowHeight + event.pageY - initPageY;
      if (newRowHeight !== rowHeight) {
        this.onRowHeightChanges(sheet, row, newRowHeight);
      }
    };
    resizeHandleUp = resizeHandleUp.bind(this);
    /* istanbul ignore next */
    div.addEventListener("pointerdown", (event) => {
      event.stopPropagation();
      div.style.opacity = "1";
      this.rowGuide.style.display = "block";
      this.rowGuide.style.top = `${y}px`;
      initPageY = event.pageY;
      document.addEventListener("pointermove", resizeHandleMove);
      document.addEventListener("pointerup", resizeHandleUp);
    });

    div.addEventListener("dblclick", (event) => {
      // This is tough. We should have a call like this.model.setAutofitRow(sheet, row)
      // but we can't do that because the back end knows nothing about the rendering engine.
      const sheet = this.model.getSelectedSheet();
      const columns = this.model.getColumnsWithData(sheet, row);
      let height = 0;
      for (const column of columns) {
        const fullText = this.model.getFormattedCellValue(sheet, row, column);
        if (fullText === "") {
          continue;
        }
        const width = this.getColumnWidth(sheet, column);
        const { style: cellStyle } = this.model.getCellStyle(
          sheet,
          row,
          column,
        );
        const fontSize = cellStyle.font.sz;
        const lineHeight = fontSize * 1.5;
        let font = `${fontSize}px ${this.theme.cellFontFamily}`;
        font = cellStyle.font.b ? `bold ${font}` : `400 ${font}`;
        this.ctx.font = font;
        const lines = computeWrappedLines(
          fullText,
          cellStyle.alignment?.wrap_text || false,
          this.ctx,
          width,
        );
        const lineCount = lines.length;
        // This is computed so that the y position of the text is independent of the vertical alignment
        const textHeight = (lineCount - 1) * lineHeight + 8 + fontSize;
        height = Math.max(height, textHeight);
      }
      // If the height is 0, we do nothing
      if (height !== 0) {
        this.model.setRowsHeight(sheet, row, row, height);
        this.refresh();
      }
      event.stopPropagation();
    });
  }

  private styleColumnHeader(
    width: number,
    div: HTMLDivElement,
    selected: boolean,
    isFullColumnSelected: boolean,
  ): void {
    div.style.boxSizing = "border-box";
    div.style.flexShrink = "0";
    div.style.width = `${width}px`;
    div.style.height = `${headerRowHeight}px`;
    div.style.backgroundColor = selected
      ? isFullColumnSelected
        ? this.theme.primaryMain
        : this.theme.headerSelectedBackground
      : this.theme.headerBackground;
    div.style.color = selected
      ? isFullColumnSelected
        ? this.theme.commonWhite
        : this.theme.headerSelectedColor
      : this.theme.headerTextColor;
    div.style.fontWeight = "bold";
    div.style.borderLeft = `1px solid ${this.theme.headerBorderColor}`;
    div.style.borderTop = `1px solid ${this.theme.headerBorderColor}`;
    if (selected) {
      div.style.borderBottom = `1px solid ${this.theme.outlineColor}`;
      div.classList.add("selected");
    } else {
      div.classList.remove("selected");
    }
  }

  private removeHandles(): void {
    const root = this.canvas.parentElement;
    if (root) {
      for (const handle of root.querySelectorAll(".row-resize-handle")) {
        handle.remove();
      }
    }
  }

  // Draws the row headers on the canvas: the frozen rows, then the rows of
  // the scrolling pane from `scrollY0` (the top of its first row, which may
  // be partly under the column headers or the frozen rows)
  private renderRowHeaders(
    frozenRows: number,
    topLeftCell: Cell,
    bottomRightCell: Cell,
    scrollY0: number,
  ): void {
    const { sheet: selectedSheet, range } = this.model.getSelectedView();
    let rowStart = range[0];
    let rowEnd = range[2];
    let columnStart = range[1];
    let columnEnd = range[3];
    if (rowStart > rowEnd) {
      [rowStart, rowEnd] = [rowEnd, rowStart];
    }
    if (columnStart > columnEnd) {
      [columnStart, columnEnd] = [columnEnd, columnStart];
    }
    const isFullRowSelected = columnStart === 1 && columnEnd === LAST_COLUMN;
    const context = this.ctx;

    const drawHeader = (row: number, y: number, rowHeight: number): void => {
      const selected = row >= rowStart && row <= rowEnd;
      context.fillStyle = this.theme.headerBorderColor;
      context.fillRect(0, y, headerColumnWidth, rowHeight);
      context.fillStyle = selected
        ? isFullRowSelected
          ? this.theme.primaryMain
          : this.theme.headerSelectedBackground
        : this.theme.headerBackground;
      context.fillRect(0, y + 0.5, headerColumnWidth, rowHeight - 1);
      if (selected) {
        context.fillStyle = this.theme.outlineColor;
        context.fillRect(headerColumnWidth - 1, y, 1, rowHeight);
      }
      context.fillStyle = selected
        ? isFullRowSelected
          ? this.theme.commonWhite
          : this.theme.headerSelectedColor
        : this.theme.headerTextColor;
      context.font = this.theme.headerFont;
      context.fillText(
        `${row}`,
        headerColumnWidth / 2,
        y + rowHeight / 2,
        headerColumnWidth,
      );
      this.addRowResizeHandle(y + rowHeight, row, rowHeight);
    };

    let y = headerRowHeight + 0.5;
    for (let row = 1; row <= frozenRows; row += 1) {
      const rowHeight = this.getRowHeight(selectedSheet, row);
      if (rowHeight <= 0) {
        continue;
      }
      drawHeader(row, y, rowHeight);
      y += rowHeight;
    }

    // The scrolling rows are clipped to their pane
    const paneTop = headerRowHeight + this.getFrozenRowsHeight();
    context.save();
    context.beginPath();
    context.rect(0, paneTop, headerColumnWidth, this.height - paneTop);
    context.clip();
    y = scrollY0;
    for (let row = topLeftCell.row; row <= bottomRightCell.row; row += 1) {
      const rowHeight = this.getRowHeight(selectedSheet, row);
      if (rowHeight <= 0) {
        continue;
      }
      drawHeader(row, y, rowHeight);
      y += rowHeight;
    }
    context.restore();
  }

  // Lays out the column headers (HTML elements over the canvas): the frozen
  // columns, the separator, then the columns of the scrolling pane in a
  // wrapper clipped to the pane. `scrollPaneX` is the canvas x of the first
  // scrolling column, which may be partly under the frozen columns or the
  // row headers.
  private renderColumnHeaders(
    frozenColumns: number,
    firstColumn: number,
    lastColumn: number,
    scrollPaneX: number,
  ): void {
    const { columnHeaders } = this;
    const { range } = this.model.getSelectedView();
    let rowStart = range[0];
    let rowEnd = range[2];
    let columnStart = range[1];
    let columnEnd = range[3];
    if (columnStart > columnEnd) {
      [columnStart, columnEnd] = [columnEnd, columnStart];
    }
    if (rowStart > rowEnd) {
      [rowStart, rowEnd] = [rowEnd, rowStart];
    }
    const isFullColumnSelected = rowStart === 1 && rowEnd === LAST_ROW;
    for (const selector of [
      ".column-header",
      ".column-resize-handle",
      ".frozen-column-separator",
      ".column-headers-scroll",
    ]) {
      for (const element of columnHeaders.querySelectorAll(selector)) {
        element.remove();
      }
    }
    columnHeaders.style.font = this.theme.headerFont;
    columnHeaders.style.height = `${headerRowHeight}px`;
    columnHeaders.style.lineHeight = `${headerRowHeight}px`;
    columnHeaders.style.left = `${headerColumnWidth}px`;
    columnHeaders.style.width = `${this.width - headerColumnWidth}px`;

    // Frozen headers; `x` is relative to the headers' container, which
    // starts at the first cell column
    let x = 0;
    for (let column = 1; column <= frozenColumns; column += 1) {
      const selected = column >= columnStart && column <= columnEnd;
      x += this.addColumnHeader(
        columnHeaders,
        x,
        column,
        selected,
        isFullColumnSelected,
        0,
      );
    }

    if (frozenColumns !== 0) {
      const div = document.createElement("div");
      div.className = "frozen-column-separator";
      div.style.width = `${frozenSeparatorWidth}px`;
      div.style.height = `${headerRowHeight}px`;
      div.style.display = "inline-block";
      div.style.flexShrink = "0";
      div.style.backgroundColor = this.theme.gridSeparatorColor;
      columnHeaders.insertBefore(div, null);
      x += frozenSeparatorWidth;
    }

    // The scrolling headers, clipped to the scrolling pane
    const scrolling = document.createElement("div");
    scrolling.className = "column-headers-scroll";
    scrolling.style.display = "flex";
    scrolling.style.overflow = "hidden";
    scrolling.style.flexShrink = "0";
    scrolling.style.width = `${this.width - headerColumnWidth - x}px`;
    scrolling.style.height = `${headerRowHeight}px`;
    columnHeaders.insertBefore(scrolling, null);
    // How far the first scrolling header is shifted out of view (0 or less)
    const shift = scrollPaneX - headerColumnWidth - x;
    x += shift;
    let firstVisible = true;
    for (let column = firstColumn; column <= lastColumn; column += 1) {
      const selected = column >= columnStart && column <= columnEnd;
      const width = this.addColumnHeader(
        scrolling,
        x,
        column,
        selected,
        isFullColumnSelected,
        firstVisible ? shift : 0,
      );
      if (width > 0) {
        firstVisible = false;
      }
      x += width;
    }
  }

  // Adds the header of a column to `parent`, at `x` from the start of the
  // headers' container, with its resize handle. Returns the column width.
  private addColumnHeader(
    parent: HTMLDivElement,
    x: number,
    column: number,
    selected: boolean,
    isFullColumnSelected: boolean,
    marginLeft: number,
  ): number {
    const columnWidth = this.getColumnWidth(
      this.model.getSelectedSheet(),
      column,
    );
    if (columnWidth <= 0) {
      return 0;
    }
    const div = document.createElement("div");
    div.className = "column-header";
    div.textContent = columnNameFromNumber(column);
    parent.insertBefore(div, null);

    this.styleColumnHeader(columnWidth, div, selected, isFullColumnSelected);
    if (marginLeft !== 0) {
      div.style.marginLeft = `${marginLeft}px`;
    }
    this.addColumnResizeHandle(x + columnWidth, column, columnWidth);
    return columnWidth;
  }

  // The size of the whole sheet with its headers, in pixels
  getSheetDimensions(): [number, number] {
    const { rows, columns } = this.getAxes();
    return [
      headerColumnWidth + columns.total(),
      headerRowHeight + rows.total(),
    ];
  }

  /**
   * Returns the css clip in the canvas of an html element
   * This is used so we do not see the outlines in the row and column headers
   * NB: A _different_ (better!) approach would be to have separate canvases for the headers
   * Then the sheet canvas would have it's own bounding box.
   * That's tomorrows problem.
   * PS: Please, do not use this function. If at all we can use the clip-path property
   */
  // The CSS clip of an overlay (an element over the canvas) at (x, y) with
  // the given size, so it never covers the headers, nor the frozen panes
  // when it belongs to the scrolling rows / columns. Empty when nothing
  // needs clipping.
  // NB: A _different_ (better!) approach would be to have separate canvases
  // for the headers. Then the sheet canvas would have it's own bounding box.
  // That's tomorrows problem.
  private getClipCSS(
    x: number,
    y: number,
    width: number,
    height: number,
    scrollRows: boolean,
    scrollColumns: boolean,
  ): string {
    const frozenColumnsWidth = scrollColumns ? this.getFrozenColumnsWidth() : 0;
    const frozenRowsHeight = scrollRows ? this.getFrozenRowsHeight() : 0;
    const yMinCanvas = headerRowHeight + frozenRowsHeight;
    const xMinCanvas = headerColumnWidth + frozenColumnsWidth;
    const xMaxCanvas = this.width;
    const yMaxCanvas = this.height;

    const topClip = y < yMinCanvas ? `${yMinCanvas - y}px` : "auto";
    const leftClip = x < xMinCanvas ? `${xMinCanvas - x}px` : "auto";
    const rightClip = x + width > xMaxCanvas ? `${xMaxCanvas - x}px` : "auto";
    const bottomClip = y + height > yMaxCanvas ? `${yMaxCanvas - y}px` : "auto";
    if ([topClip, rightClip, bottomClip, leftClip].every((c) => c === "auto")) {
      return "";
    }
    return `rect(${topClip} ${rightClip} ${bottomClip} ${leftClip})`;
  }

  private getAreaDimensions(
    startRow: number,
    startColumn: number,
    endRow: number,
    endColumn: number,
  ): [number, number] {
    const [xStart, yStart] = this.getCoordinatesByCell(startRow, startColumn);
    let [xEnd, yEnd] = this.getCoordinatesByCell(endRow, endColumn);
    xEnd += this.getColumnWidth(this.model.getSelectedSheet(), endColumn);
    yEnd += this.getRowHeight(this.model.getSelectedSheet(), endRow);
    const frozenRows = this.model.getFrozenRowsCount(
      this.model.getSelectedSheet(),
    );
    const frozenColumns = this.model.getFrozenColumnsCount(
      this.model.getSelectedSheet(),
    );
    if (frozenRows !== 0 || frozenColumns !== 0) {
      let [xFrozenEnd, yFrozenEnd] = this.getCoordinatesByCell(
        frozenRows,
        frozenColumns,
      );
      if (frozenColumns > 0) {
        xFrozenEnd += this.getColumnWidth(
          this.model.getSelectedSheet(),
          frozenColumns,
        );
      }
      if (frozenRows > 0) {
        yFrozenEnd += this.getRowHeight(
          this.model.getSelectedSheet(),
          frozenRows,
        );
      }
      if (startRow <= frozenRows && endRow > frozenRows) {
        yEnd = Math.max(yEnd, yFrozenEnd);
      }
      if (startColumn <= frozenColumns && endColumn > frozenColumns) {
        xEnd = Math.max(xEnd, xFrozenEnd);
      }
    }
    return [Math.abs(xEnd - xStart), Math.abs(yEnd - yStart)];
  }

  /**
   * Returns the coordinates relative to the canvas of the top-left corner of
   * a cell. A cell in the frozen rows or columns is laid out in its frozen
   * pane; any other cell is laid out in the scrolling pane, before its
   * top-left corner when it is scrolled out of view.
   */
  getCoordinatesByCell(row: number, column: number): [number, number] {
    const selectedSheet = this.model.getSelectedSheet();
    const frozenColumns = this.model.getFrozenColumnsCount(selectedSheet);
    const frozenRows = this.model.getFrozenRowsCount(selectedSheet);
    const { rows, columns } = this.getAxes();
    const [originX, originY] = this.getOrigin();
    const scroll = this.getScroll();
    const y =
      row <= frozenRows
        ? headerRowHeight + rows.start(row)
        : headerRowHeight +
          this.getFrozenRowsHeight() +
          rows.start(row) -
          originY -
          scroll.y;
    const x =
      column <= frozenColumns
        ? headerColumnWidth + columns.start(column)
        : headerColumnWidth +
          this.getFrozenColumnsWidth() +
          columns.start(column) -
          originX -
          scroll.x;
    return [Math.floor(x), Math.floor(y)];
  }

  /**
   * (x, y) are the relative coordinates of a cell WRT the canvas
   * getCellByCoordinates(headerColumnWidth, headerRowHeight) will return the first visible cell.
   * Note: If there are frozen rows/columns for some particular coordinates (x, y)
   * there might be two cells. This method returns the visible one.
   * Coordinates outside the cells (over the headers, or past an edge of the
   * canvas) map to the cell that would be there: over the headers that is a
   * frozen cell when there are frozen panes, and otherwise the cell just
   * before the scrolling pane (so a drag past the headers extends the
   * selection out of view).
   */
  getCellByCoordinates(
    x: number,
    y: number,
  ): { row: number; column: number } | null {
    const selectedSheet = this.model.getSelectedSheet();
    const frozenColumns = this.model.getFrozenColumnsCount(selectedSheet);
    const frozenRows = this.model.getFrozenRowsCount(selectedSheet);
    const { rows, columns } = this.getAxes();
    const [originX, originY] = this.getOrigin();
    const scroll = this.getScroll();
    let column: number;
    if (
      frozenColumns > 0 &&
      x < headerColumnWidth + this.getFrozenColumnsWidth()
    ) {
      column = Math.min(columns.at(x - headerColumnWidth), frozenColumns);
    } else {
      column = Math.max(
        columns.at(
          originX +
            scroll.x +
            (x - headerColumnWidth - this.getFrozenColumnsWidth()),
        ),
        frozenColumns + 1,
      );
    }
    let row: number;
    if (frozenRows > 0 && y < headerRowHeight + this.getFrozenRowsHeight()) {
      row = Math.min(rows.at(y - headerRowHeight), frozenRows);
    } else {
      row = Math.max(
        rows.at(
          originY +
            scroll.y +
            (y - headerRowHeight - this.getFrozenRowsHeight()),
        ),
        frozenRows + 1,
      );
    }
    return { row, column };
  }

  private drawExtendToArea(): void {
    const { extendToOutline } = this;
    const extendToArea = this.workbookState.getExtendToArea();
    if (extendToArea === null) {
      extendToOutline.style.visibility = "hidden";
      return;
    }
    extendToOutline.style.visibility = "visible";

    let { rowStart, rowEnd, columnStart, columnEnd } = extendToArea;
    if (rowStart > rowEnd) {
      [rowStart, rowEnd] = [rowEnd, rowStart];
    }
    if (columnStart > columnEnd) {
      [columnStart, columnEnd] = [columnEnd, columnStart];
    }

    const [areaX, areaY] = this.getCoordinatesByCell(rowStart, columnStart);
    const [areaWidth, areaHeight] = this.getAreaDimensions(
      rowStart,
      columnStart,
      rowEnd,
      columnEnd,
    );
    extendToOutline.style.border = `1px dashed ${this.theme.outlineColor}`;
    extendToOutline.style.borderRadius = "3px";

    extendToOutline.style.left = `${areaX}px`;
    extendToOutline.style.top = `${areaY}px`;
    extendToOutline.style.width = `${areaWidth - 1}px`;
    extendToOutline.style.height = `${areaHeight - 1}px`;
    const selectedSheet = this.model.getSelectedSheet();
    extendToOutline.style.clip = this.getClipCSS(
      areaX,
      areaY,
      areaWidth + 1,
      areaHeight + 1,
      rowStart > this.model.getFrozenRowsCount(selectedSheet),
      columnStart > this.model.getFrozenColumnsCount(selectedSheet),
    );
  }

  private getColumnWidth(sheet: number, column: number): number {
    if (sheet === this.model.getSelectedSheet()) {
      return this.getAxes().columns.size(column);
    }
    return Math.round(this.model.getColumnWidth(sheet, column));
  }

  private getRowHeight(sheet: number, row: number): number {
    if (sheet === this.model.getSelectedSheet()) {
      return this.getAxes().rows.size(row);
    }
    return Math.round(this.model.getRowHeight(sheet, row));
  }

  private drawCellEditor(): void {
    const cell = this.workbookState.getEditingCell();
    const selectedSheet = this.model.getSelectedSheet();
    const { editor } = this;
    if (!cell || cell.sheet !== selectedSheet) {
      // If the editing cell is not in the same sheet as the selected sheet
      // we take the editor out of view
      editor.style.left = "-9999px";
      editor.style.top = "-9999px";
      return;
    }
    const { row, column } = cell;
    // const style = this.model.getCellStyle(
    //   selectedSheet,
    //   selectedRow,
    //   selectedColumn
    // );
    // cellOutline.style.fontWeight = style.font.b ? "bold" : "normal";
    // cellOutline.style.fontStyle = style.font.i ? "italic" : "normal";
    // cellOutline.style.backgroundColor = style.fill.color;
    // TODO: Should we add the same color as the text?
    // Only if it is not a formula?
    // cellOutline.style.color = style.font.color;
    const [x, y] = this.getCoordinatesByCell(row, column);
    const padding = -1;
    const width = cell.editorWidth + 2 * padding;
    const height = cell.editorHeight + 2 * padding;
    // const width =
    //   this.getColumnWidth(sheet, column) + 2 * padding;
    // const height = this.getRowHeight(sheet, row) + 2 * padding;
    editor.style.left = `${x}px`;
    editor.style.top = `${y}px`;
    editor.style.width = `${width - 1}px`;
    editor.style.height = `${height - 1}px`;
    editor.style.clip = this.getClipCSS(
      x,
      y,
      width + 5,
      height + 5,
      row > this.model.getFrozenRowsCount(selectedSheet),
      column > this.model.getFrozenColumnsCount(selectedSheet),
    );
  }

  // Draws the array structure
  private drawArrayStructure(): void {
    const { cellArrayStructure } = this;
    // By default there is no "array structure"
    cellArrayStructure.style.visibility = "hidden";

    const [selectedSheet, selectedRow, selectedColumn] =
      this.model.getSelectedCell();

    // we draw the array structure if needed only if the selected cell is part of an array
    const arrayStructure = this.model.getCellArrayStructure(
      selectedSheet,
      selectedRow,
      selectedColumn,
    );
    let array = null;
    if (arrayStructure === "SingleCell") {
      // nothing to see here
      return;
    } else if ("DynamicAnchor" in arrayStructure) {
      const [arrayWidth, arrayHeight] = arrayStructure.DynamicAnchor;
      array = [selectedRow, selectedColumn, arrayWidth, arrayHeight];
    } else if ("DynamicChild" in arrayStructure) {
      array = arrayStructure.DynamicChild;
    } else if ("ArrayAnchor" in arrayStructure) {
      const [arrayWidth, arrayHeight] = arrayStructure.ArrayAnchor;
      array = [selectedRow, selectedColumn, arrayWidth, arrayHeight];
    } else if ("ArrayChild" in arrayStructure) {
      array = arrayStructure.ArrayChild;
    } else {
      return;
    }

    const [arrayX, arrayY] = this.getCoordinatesByCell(array[0], array[1]);
    const [arrayX1, arrayY1] = this.getCoordinatesByCell(
      array[0] + array[3],
      array[1] + array[2],
    );

    Object.assign(cellArrayStructure.style, {
      visibility: "visible",
      left: `${arrayX}px`,
      top: `${arrayY}px`,
      width: `${arrayX1 - arrayX}px`,
      height: `${arrayY1 - arrayY}px`,
      clip: this.getClipCSS(
        arrayX,
        arrayY,
        arrayX1 - arrayX + 2,
        arrayY1 - arrayY + 2,
        array[0] > this.model.getFrozenRowsCount(selectedSheet),
        array[1] > this.model.getFrozenColumnsCount(selectedSheet),
      ),
    });
  }

  // Draws:
  // * cell outline marking the selected cell
  // * the area outline marking the selected area (if any)
  // * the handle (the small circle/square that allows you to extend and autofill)
  private drawCellOutline(): void {
    const { cellOutline, areaOutline, cellOutlineHandle } = this;
    if (this.workbookState.getEditingCell()) {
      cellOutline.style.visibility = "hidden";
      cellOutlineHandle.style.visibility = "hidden";
    } else {
      cellOutline.style.visibility = "visible";
      cellOutlineHandle.style.visibility = this.workbookState.isSelecting()
        ? "hidden"
        : "visible";
    }

    areaOutline.style.visibility = "visible";

    const [selectedSheet, selectedRow, selectedColumn] =
      this.model.getSelectedCell();
    const { topLeftCell } = this.getVisibleCells();
    const frozenRows = this.model.getFrozenRowsCount(selectedSheet);
    const frozenColumns = this.model.getFrozenColumnsCount(selectedSheet);
    const [x, y] = this.getCoordinatesByCell(selectedRow, selectedColumn);

    const padding = -1;
    // A selected merged cell outlines the whole merged rectangle
    const selectedMerge = this.mergeMap.get(`${selectedRow}-${selectedColumn}`);
    let width: number;
    let height: number;
    // Last row and column covered by the outline
    let lastRow = selectedRow;
    let lastColumn = selectedColumn;
    if (selectedMerge) {
      lastRow = selectedMerge.row + selectedMerge.height - 1;
      lastColumn = selectedMerge.column + selectedMerge.width - 1;
      // On-screen dimensions, not the summed cell sizes: when the merged
      // range straddles a frozen pane, the rows/columns scrolled behind the
      // frozen separator take no space on screen
      const [mergeWidth, mergeHeight] = this.getAreaDimensions(
        selectedMerge.row,
        selectedMerge.column,
        lastRow,
        lastColumn,
      );
      width = mergeWidth + 2 * padding;
      height = mergeHeight + 2 * padding;
    } else {
      width = this.getColumnWidth(selectedSheet, selectedColumn) + 2 * padding;
      height = this.getRowHeight(selectedSheet, selectedRow) + 2 * padding;
    }

    // Hidden when scrolled entirely out of view (under the frozen panes).
    // For a merged cell that means every row/column of it: a merge reaching
    // from the frozen pane into the hidden band still shows its frozen part.
    if (
      (selectedRow > frozenRows && lastRow < topLeftCell.row) ||
      (selectedColumn > frozenColumns && lastColumn < topLeftCell.column)
    ) {
      cellOutline.style.visibility = "hidden";
    }
    // A merged cell that continues under the frozen separator has no visible
    // bottom/right edge: suppress that border instead of drawing it on the
    // separator as if the cell ended there
    cellOutline.style.borderBottom =
      lastRow > frozenRows && lastRow < topLeftCell.row ? "none" : "";
    cellOutline.style.borderRight =
      lastColumn > frozenColumns && lastColumn < topLeftCell.column
        ? "none"
        : "";
    // A cell partially scrolled under the headers or the frozen panes is
    // clipped
    cellOutline.style.clip = this.getClipCSS(
      x - padding - 2,
      y - padding - 2,
      width + 1,
      height + 1,
      selectedRow > frozenRows,
      selectedColumn > frozenColumns,
    );

    // Position the cell outline and clip it
    cellOutline.style.left = `${x - padding - 2}px`;
    cellOutline.style.top = `${y - padding - 2}px`;
    // Reset CSS properties
    cellOutline.style.minWidth = "";
    cellOutline.style.minHeight = "";
    cellOutline.style.maxWidth = "";
    cellOutline.style.maxHeight = "";
    cellOutline.style.overflow = "hidden";
    // New properties
    cellOutline.style.width = `${width + 1}px`;
    cellOutline.style.height = `${height + 1}px`;

    cellOutline.style.background = "none";

    // border is 2px so line-height must be height - 4
    cellOutline.style.lineHeight = `${height - 4}px`;
    let {
      range: [rowStart, columnStart, rowEnd, columnEnd],
    } = this.model.getSelectedView();
    if (rowStart > rowEnd) {
      [rowStart, rowEnd] = [rowEnd, rowStart];
    }
    if (columnStart > columnEnd) {
      [columnStart, columnEnd] = [columnEnd, columnStart];
    }

    let handleX: number;
    let handleY: number;
    // A selection of exactly one merged cell behaves like a single cell:
    // the cell outline (sized to the merged rectangle above) is enough.
    const isSingleMergedCell =
      selectedMerge !== undefined &&
      rowStart === selectedMerge.row &&
      columnStart === selectedMerge.column &&
      rowEnd === selectedMerge.row + selectedMerge.height - 1 &&
      columnEnd === selectedMerge.column + selectedMerge.width - 1;
    // Position the selected area outline
    if (
      (columnStart === columnEnd && rowStart === rowEnd) ||
      isSingleMergedCell
    ) {
      areaOutline.style.visibility = "hidden";
      // The handle sits on the on-screen bottom-right corner of the last
      // cell (for a merged cell that can be in a different pane than the
      // anchor, so it cannot be derived from the outline's origin and size)
      [handleX, handleY] = this.getCoordinatesByCell(lastRow, lastColumn);
      handleX += this.getColumnWidth(selectedSheet, lastColumn);
      handleY += this.getRowHeight(selectedSheet, lastRow);
      // hide the handle if its corner is scrolled under the frozen panes
      if (
        (lastRow > frozenRows && lastRow < topLeftCell.row) ||
        (lastColumn > frozenColumns && lastColumn < topLeftCell.column)
      ) {
        cellOutlineHandle.style.visibility = "hidden";
      }
    } else {
      const [areaX, areaY] = this.getCoordinatesByCell(rowStart, columnStart);
      const [areaWidth, areaHeight] = this.getAreaDimensions(
        rowStart,
        columnStart,
        rowEnd,
        columnEnd,
      );
      handleX = areaX + areaWidth;
      handleY = areaY + areaHeight;
      const isSelecting = this.workbookState.isSelecting();
      // Add 1px when selecting to compensate for missing border
      const borderCompensation = isSelecting ? 1 : 0;
      areaOutline.style.left = `${areaX - padding - 1}px`;
      areaOutline.style.top = `${areaY - padding - 1}px`;
      areaOutline.style.width = `${areaWidth + 2 * padding + 1 + borderCompensation}px`;
      areaOutline.style.height = `${areaHeight + 2 * padding + 1 + borderCompensation}px`;
      areaOutline.style.clip = this.getClipCSS(
        areaX - padding - 1,
        areaY - padding - 1,
        areaWidth + 2 * padding + 1 + borderCompensation,
        areaHeight + 2 * padding + 1 + borderCompensation,
        rowStart > frozenRows,
        columnStart > frozenColumns,
      );
      areaOutline.style.border = isSelecting
        ? "none"
        : `1px solid ${this.theme.outlineColor}`;
      // hide the handle if it is out of the visible area
      if (
        (rowEnd > frozenRows && rowEnd < topLeftCell.row - 1) ||
        (columnEnd > frozenColumns && columnEnd < topLeftCell.column - 1)
      ) {
        cellOutlineHandle.style.visibility = "hidden";
      }

      // This is in case the selection starts in the frozen area and ends outside of the frozen area
      // but we have scrolled out the selection.
      if (
        rowStart <= frozenRows &&
        rowEnd > frozenRows &&
        rowEnd < topLeftCell.row - 1
      ) {
        areaOutline.style.borderBottom = "None";
        cellOutlineHandle.style.visibility = "hidden";
      }
      if (
        columnStart <= frozenColumns &&
        columnEnd > frozenColumns &&
        columnEnd < topLeftCell.column - 1
      ) {
        areaOutline.style.borderRight = "None";
        cellOutlineHandle.style.visibility = "hidden";
      }
    }

    const handleBBox = cellOutlineHandle.getBoundingClientRect();
    const handleWidth = handleBBox.width;
    const handleHeight = handleBBox.height;
    cellOutlineHandle.style.left = `${handleX - handleWidth / 2 - 1}px`;
    cellOutlineHandle.style.top = `${handleY - handleHeight / 2 - 1}px`;
  }

  private drawCutRange(): void {
    const range = this.workbookState.getCutRange() || null;
    if (!range) {
      return;
    }
    const selectedSheet = this.model.getSelectedSheet();
    if (range.sheet !== selectedSheet) {
      return;
    }
    const ctx = this.ctx;
    this.clipToCells();
    ctx.setLineDash([2, 2]);

    const [xStart, yStart] = this.getCoordinatesByCell(
      range.rowStart,
      range.columnStart,
    );
    const [xEnd, yEnd] = this.getCoordinatesByCell(
      range.rowEnd + 1,
      range.columnEnd + 1,
    );
    ctx.strokeStyle = "red";
    ctx.lineWidth = 1;
    ctx.strokeRect(xStart, yStart, xEnd - xStart, yEnd - yStart);

    ctx.setLineDash([]);
    ctx.restore();
  }

  private drawActiveRanges(topLeftCell: Cell, bottomRightCell: Cell): void {
    let activeRanges = this.workbookState.getActiveRanges();
    const ctx = this.ctx;
    this.clipToCells();
    ctx.setLineDash([2, 2]);
    const referencedRange =
      this.workbookState.getEditingCell()?.referencedRange || null;
    if (referencedRange) {
      activeRanges = activeRanges.concat([
        {
          ...referencedRange.range,
          color: getColor(activeRanges.length),
        },
      ]);
    }
    const selectedSheet = this.model.getSelectedSheet();
    const activeRangesCount = activeRanges.length;
    for (let rangeIndex = 0; rangeIndex < activeRangesCount; rangeIndex += 1) {
      const range = activeRanges[rangeIndex];
      if (range.sheet !== selectedSheet) {
        continue;
      }

      const allowedOffset = 1; // to make borders look nicer
      const minRow = topLeftCell.row - allowedOffset;
      const maxRow = bottomRightCell.row + allowedOffset;
      const minColumn = topLeftCell.column - allowedOffset;
      const maxColumn = bottomRightCell.column + allowedOffset;

      if (
        minRow <= range.rowEnd &&
        range.rowStart <= maxRow &&
        minColumn <= range.columnEnd &&
        range.columnStart < maxColumn
      ) {
        // Range in the viewport.
        const displayRange: typeof range = {
          ...range,
          rowStart: Math.max(minRow, range.rowStart),
          rowEnd: Math.min(maxRow, range.rowEnd),
          columnStart: Math.max(minColumn, range.columnStart),
          columnEnd: Math.min(maxColumn, range.columnEnd),
        };
        const [xStart, yStart] = this.getCoordinatesByCell(
          displayRange.rowStart,
          displayRange.columnStart,
        );
        const [xEnd, yEnd] = this.getCoordinatesByCell(
          displayRange.rowEnd + 1,
          displayRange.columnEnd + 1,
        );
        ctx.strokeStyle = range.color;
        ctx.lineWidth = 1;
        ctx.strokeRect(xStart, yStart, xEnd - xStart, yEnd - yStart);
        ctx.fillStyle = hexToRGBA10Percent(range.color);
        ctx.fillRect(xStart, yStart, xEnd - xStart, yEnd - yStart);
      }
    }

    ctx.setLineDash([]);
    ctx.restore();
  }

  // The lines between the headers and the cells, across the whole canvas:
  // they are the right and bottom edges of the corner square as well. The
  // vertical one lies in the last pixel column of the row headers, where
  // the column headers (HTML elements over the canvas, from the first cell
  // column on) cannot cover it.
  private drawHeaderSeparators(): void {
    const context = this.ctx;
    context.save();
    context.strokeStyle = this.theme.headerBorderColor;
    context.lineWidth = 1;
    const x = headerColumnWidth - 0.5;
    const y = headerRowHeight + 0.5;
    drawBorderLine(context, 0, y, this.width, y);
    drawBorderLine(context, x, 0, x, this.height);
    context.restore();
  }

  renderSheet(): void {
    const { canvas, ctx: context } = this;
    const selectedSheet = this.model.getSelectedSheet();
    // Row heights and column widths may have changed since the last render
    this.axes = null;
    context.lineWidth = 1;
    context.textAlign = "center";
    context.textBaseline = "middle";

    // Clear the canvas
    context.clearRect(0, 0, canvas.width, canvas.height);

    this.removeHandles();

    this.updateMergedCells();

    const { topLeftCell, bottomRightCell } = this.getVisibleCells();

    // Pass 1
    this.computeCellsText();
    // The cell geometry may have changed (scroll, resize, edits): a link
    // tooltip anchored to a cell that moved or lost its link must go.
    this.cellLinks.validateTooltip();

    const frozenColumns = this.model.getFrozenColumnsCount(selectedSheet);
    const frozenRows = this.model.getFrozenRowsCount(selectedSheet);
    const panes = this.getPanes();

    let x = headerColumnWidth + 0.5;
    let y = headerRowHeight + 0.5;

    // Draw frozen rows and columns (top-left-pane)
    this.clipToPane(0, panes);
    for (let row = 1; row <= frozenRows; row += 1) {
      const rowHeight = this.getRowHeight(selectedSheet, row);
      x = headerColumnWidth + 0.5;
      for (let column = 1; column <= frozenColumns; column += 1) {
        const columnWidth = this.getColumnWidth(selectedSheet, column);
        this.renderCellStyle(row, column, x, y, columnWidth, rowHeight);
        x += columnWidth;
      }
      y += rowHeight;
    }
    context.restore();
    if (frozenRows === 0 && frozenColumns !== 0) {
      x = headerColumnWidth + 0.5;
      for (let column = 1; column <= frozenColumns; column += 1) {
        x += this.getColumnWidth(selectedSheet, column);
      }
    }

    const frozenOffset = frozenSeparatorWidth / 2;
    // If there are frozen rows draw a separator
    if (frozenRows) {
      // Grid line of the last frozen row, no cell below it draws it
      context.fillStyle = this.theme.gridSeparatorColor;
      context.fillRect(0, Math.max(0, Math.round(y) - 1), this.width, 1);
      context.beginPath();
      context.lineWidth = frozenSeparatorWidth;
      context.strokeStyle = this.theme.gridSeparatorColor;
      context.moveTo(0, y + frozenOffset);
      context.lineTo(this.width, y + frozenOffset);
      y += frozenSeparatorWidth;
      context.stroke();
      context.lineWidth = 1;
    }

    // If there are frozen columns draw a separator
    if (frozenColumns) {
      // Grid line of the last frozen column, no cell to its right draws it.
      context.fillStyle = this.theme.gridSeparatorColor;
      context.fillRect(Math.max(0, Math.round(x) - 2), 0, 2, this.height);
      context.beginPath();
      context.lineWidth = frozenSeparatorWidth;
      context.strokeStyle = this.theme.gridSeparatorColor;
      context.moveTo(x + frozenOffset, 0);
      context.lineTo(x + frozenOffset, this.height);
      x += frozenSeparatorWidth;
      context.stroke();
      context.lineWidth = 1;
    }

    // The scrolling pane's first row and column start at the scroll offset:
    // partly out of view when it is not on a cell boundary
    const [scrollPaneX, scrollPaneY] = this.getCoordinatesByCell(
      topLeftCell.row,
      topLeftCell.column,
    );
    const scrollX0 = scrollPaneX + 0.5;
    const scrollY0 = scrollPaneY + 0.5;
    // Draw frozen rows (top-right pane)
    this.clipToPane(1, panes);
    y = headerRowHeight + 0.5;
    for (let row = 1; row <= frozenRows; row += 1) {
      x = scrollX0;
      const rowHeight = this.getRowHeight(selectedSheet, row);
      for (
        let { column } = topLeftCell;
        column <= bottomRightCell.column;
        column += 1
      ) {
        const columnWidth = this.getColumnWidth(selectedSheet, column);
        this.renderCellStyle(row, column, x, y, columnWidth, rowHeight);
        x += columnWidth;
      }
      y += rowHeight;
    }
    context.restore();

    // Draw frozen columns (bottom-left pane)
    this.clipToPane(2, panes);
    y = scrollY0;
    for (let { row } = topLeftCell; row <= bottomRightCell.row; row += 1) {
      x = headerColumnWidth;
      const rowHeight = this.getRowHeight(selectedSheet, row);

      for (let column = 1; column <= frozenColumns; column += 1) {
        const columnWidth = this.getColumnWidth(selectedSheet, column);
        this.renderCellStyle(row, column, x, y, columnWidth, rowHeight);

        x += columnWidth;
      }
      y += rowHeight;
    }
    context.restore();

    // Render all remaining cells (bottom-right pane)
    this.clipToPane(3, panes);
    y = scrollY0;
    for (let { row } = topLeftCell; row <= bottomRightCell.row; row += 1) {
      x = scrollX0;
      const rowHeight = this.getRowHeight(selectedSheet, row);

      for (
        let { column } = topLeftCell;
        column <= bottomRightCell.column;
        column += 1
      ) {
        const columnWidth = this.getColumnWidth(selectedSheet, column);
        this.renderCellStyle(row, column, x, y, columnWidth, rowHeight);

        x += columnWidth;
      }
      y += rowHeight;
    }
    context.restore();

    // Render all cell texts
    for (const cell of this.cells) {
      this.renderCellText(cell);
    }

    // Draw column headers
    this.renderColumnHeaders(
      frozenColumns,
      topLeftCell.column,
      bottomRightCell.column,
      scrollPaneX,
    );

    // Draw row headers
    this.renderRowHeaders(frozenRows, topLeftCell, bottomRightCell, scrollY0);

    // square in the top left corner
    // (extends 0.5px right and down to meet the headers, which start at +0.5)
    context.fillStyle = this.theme.headerCornerBackground;
    context.fillRect(0, 0, headerColumnWidth + 0.5, headerRowHeight + 0.5);
    context.beginPath();
    context.strokeStyle = this.theme.gridSeparatorColor;
    context.moveTo(0, 0.5);
    context.lineTo(x + headerColumnWidth, 0.5);
    context.stroke();

    // Separators between the headers and the cells, over the cells: the
    // first row and column of the scrolling pane slide under the headers
    this.drawHeaderSeparators();

    // Overlays drawn on top of everything else
    this.drawCellOutline();
    this.drawArrayStructure();
    this.drawCellEditor();
    this.drawExtendToArea();
    this.drawActiveRanges(topLeftCell, bottomRightCell);
    this.drawCutRange();
  }
}
