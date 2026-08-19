import type { CellLink } from "@ironcalc/wasm";
import { columnNumberFromName } from "@ironcalc/wasm";
import { ALLOWED_SCHEMES } from "../LinkDialog/util";
import { LAST_COLUMN, LAST_ROW } from "./constants";
import type WorksheetCanvas from "./worksheetCanvas";

// The rendered cell (with its canvas rectangle) whose link the pointer is
// hovering. The link tooltip (a React component) is anchored to it.
export interface LinkHoverCell {
  row: number;
  column: number;
  x: number;
  y: number;
  width: number;
  height: number;
  link: CellLink;
}

export interface CellLinksOptions {
  // The pointer is over a link cell (or over none when null). Drives the
  // link tooltip shown by the React side.
  onLinkHover?: (cell: LinkHoverCell | null) => void;
  // The shown tooltip no longer matches the sheet geometry: hide it now.
  onHideTooltip?: () => void;
  // The cell whose link tooltip is currently shown (null if hidden)
  tooltipCell: LinkHoverCell | null;
}

// Pointer cursor while Ctrl/Cmd+click would follow a link
const LINK_CURSOR_CLASS = "ic-worksheet-link-cursor";
let linkContainer: HTMLElement | null = null;
let linkHovered = false;
let modifierDown = false;
let modifierTracked = false;

function updateLinkCursor(): void {
  linkContainer?.classList.toggle(
    LINK_CURSOR_CLASS,
    linkHovered && modifierDown,
  );
}

// The container is never focused so Ctrl/Cmd is tracked on the window
function trackModifierKey(): void {
  if (modifierTracked || typeof window === "undefined") {
    return;
  }
  modifierTracked = true;
  const onKey = (event: KeyboardEvent): void => {
    modifierDown = event.ctrlKey || event.metaKey;
    updateLinkCursor();
  };
  window.addEventListener("keydown", onKey);
  window.addEventListener("keyup", onKey);
  window.addEventListener("blur", () => {
    modifierDown = false;
    updateLinkCursor();
  });
}

// Handles the cell links of the selected sheet: hit-testing the rendered
// cells on pointer moves (hover reporting) and following a link.
// The tooltip itself is rendered by React (see Worksheet/LinkTooltip.tsx).
export class CellLinks {
  private worksheet: WorksheetCanvas;
  private options: CellLinksOptions;
  // links in the selected sheet keyed by "row-column"
  private links: Map<string, CellLink>;

  constructor(worksheet: WorksheetCanvas, options: CellLinksOptions) {
    this.worksheet = worksheet;
    this.options = options;
    this.links = new Map<string, CellLink>();
    this.attachHandlers();
    trackModifierKey();
  }

  setSheetLinks(cellLinks: CellLink[]): void {
    this.links.clear();
    for (const cellLink of cellLinks) {
      this.links.set(`${cellLink.row}-${cellLink.column}`, cellLink);
    }
  }

  getLink(row: number, column: number): CellLink | null {
    return this.links.get(`${row}-${column}`) || null;
  }

  private attachHandlers(): void {
    // Listen on the sheet container rather than the canvas: the overlay divs
    // (cell outline, area outline, ...) sit on top of the canvas and would
    // otherwise swallow the pointer events.
    //
    // NB: A new WorksheetCanvas is created on every React render over the same
    // DOM elements, so the handlers are assigned (`onpointermove = ...`) rather
    // than added with addEventListener: assigning replaces the handlers of the
    // previous instance instead of accumulating stale ones.
    const canvas = this.worksheet.canvas;
    const container = canvas.parentElement ?? canvas;
    linkContainer = container;
    container.onpointermove = (event) => {
      // While the pointer is over the tooltip it is not hovering any cell:
      // the tooltip component handles its own pointer events.
      const target = event.target as Element;
      if (target.closest?.(".ic-worksheet-link-tooltip")) {
        return;
      }
      const rect = canvas.getBoundingClientRect();
      const cell = this.getLinkCellAt(
        event.clientX - rect.left,
        event.clientY - rect.top,
      );
      linkHovered = cell !== null;
      modifierDown = event.ctrlKey || event.metaKey;
      updateLinkCursor();
      this.options.onLinkHover?.(cell);
    };
    container.onpointerleave = () => {
      linkHovered = false;
      updateLinkCursor();
      this.options.onLinkHover?.(null);
    };
    // Ctrl+click (Cmd+click on Mac) on a link cell follows the link. Only the
    // primary button: on Mac Ctrl+click is a right click (context menu).
    container.onpointerdown = (event) => {
      if (event.button !== 0 || !(event.ctrlKey || event.metaKey)) {
        return;
      }
      const target = event.target as Element;
      if (target.closest?.(".ic-worksheet-link-tooltip")) {
        return;
      }
      const rect = canvas.getBoundingClientRect();
      const cell = this.getLinkCellAt(
        event.clientX - rect.left,
        event.clientY - rect.top,
      );
      if (!cell) {
        return;
      }
      // The cell selection handler is a React listener on an ancestor:
      // stopping propagation here keeps the click from also selecting the cell.
      event.stopPropagation();
      event.preventDefault();
      this.options.onHideTooltip?.();
      this.followLink(cell.link);
    };
  }

  // Returns the visible cell with a link at canvas coordinates (x, y), if any
  private getLinkCellAt(x: number, y: number): LinkHoverCell | null {
    for (const cell of this.worksheet.cells) {
      if (
        cell.link &&
        x >= cell.x &&
        x <= cell.x + cell.width &&
        y >= cell.y &&
        y <= cell.y + cell.height
      ) {
        const { row, column, width, height, link } = cell;
        return { row, column, x: cell.x, y: cell.y, width, height, link };
      }
    }
    return null;
  }

  // The tooltip is anchored to a cell rectangle computed at hover time: after
  // a re-render (scroll, resize, edits, sheet switch) the cell may no longer
  // sit at that rectangle with that link, leaving the tooltip stale. Called
  // after the cell geometry is recomputed on every sheet render.
  validateTooltip(): void {
    const shown = this.options.tooltipCell;
    if (!shown || !this.options.onHideTooltip) {
      return;
    }
    const cell = this.worksheet.cells.find(
      (candidate) =>
        candidate.row === shown.row && candidate.column === shown.column,
    );
    if (
      !cell?.link ||
      cell.x !== shown.x ||
      cell.y !== shown.y ||
      cell.width !== shown.width ||
      cell.height !== shown.height ||
      JSON.stringify(cell.link) !== JSON.stringify(shown.link)
    ) {
      this.options.onHideTooltip();
    }
  }

  followLink(link: CellLink): void {
    if (link.type === "External") {
      // Links can come from imported files or HYPERLINK formulas, so the
      // scheme is not necessarily safe (e.g. "javascript:"): only open
      // allowlisted schemes.
      try {
        if (!ALLOWED_SCHEMES.includes(new URL(link.target).protocol)) {
          return;
        }
      } catch {
        // not a valid absolute URL
        return;
      }
      window.open(link.target, "_blank", "noopener,noreferrer");
      return;
    }
    this.followInternalLink(link.location);
  }

  // `location` is a cell reference like "Sheet1!A30", "'My Sheet'!A30" or a defined name
  private followInternalLink(location: string): void {
    const worksheet = this.worksheet;
    let sheetName: string;
    let cellRef: string;
    const separator = location.lastIndexOf("!");
    if (separator !== -1) {
      sheetName = location.slice(0, separator);
      cellRef = location.slice(separator + 1);
    } else {
      // a defined name: resolve it to its formula (e.g. "Sheet1!$B$5")
      const definedName = worksheet.model
        .getDefinedNameList()
        .find((entry) => entry.name.toLowerCase() === location.toLowerCase());
      if (!definedName) {
        return;
      }
      const formula = definedName.formula;
      const formulaSeparator = formula.lastIndexOf("!");
      if (formulaSeparator === -1) {
        return;
      }
      sheetName = formula.slice(0, formulaSeparator).replace(/^=/, "");
      cellRef = formula.slice(formulaSeparator + 1);
    }
    sheetName = sheetName.replace(/^'(.*)'$/, "$1");
    // a single cell ("A30") or a range of cells ("A1:B5")
    const parts = cellRef.replace(/\$/g, "").split(":");
    const firstCell = parts[0].match(/^([A-Za-z]+)([0-9]+)$/);
    if (!firstCell) {
      return;
    }
    // if the second part of the range cannot be parsed, select the first cell
    const lastCell =
      (parts.length === 2 && parts[1].match(/^([A-Za-z]+)([0-9]+)$/)) ||
      firstCell;
    const sheetIndex = worksheet.model
      .getWorksheetsProperties()
      .findIndex((sheet: { name: string }) => sheet.name === sheetName);
    if (sheetIndex === -1) {
      return;
    }
    let row: number;
    let column: number;
    let rowEnd: number;
    let columnEnd: number;
    try {
      column = columnNumberFromName(firstCell[1].toUpperCase());
      row = Number.parseInt(firstCell[2], 10);
      columnEnd = columnNumberFromName(lastCell[1].toUpperCase());
      rowEnd = Number.parseInt(lastCell[2], 10);
    } catch {
      return;
    }
    // normalize so that (row, column) is the top-left corner
    [row, rowEnd] = [Math.min(row, rowEnd), Math.max(row, rowEnd)];
    [column, columnEnd] = [
      Math.min(column, columnEnd),
      Math.max(column, columnEnd),
    ];
    if (row < 1 || rowEnd > LAST_ROW || column < 1 || columnEnd > LAST_COLUMN) {
      return;
    }
    worksheet.model.setSelectedSheet(sheetIndex);
    worksheet.model.setSelectedCell(row, column);
    worksheet.model.setSelectedRange(row, column, rowEnd, columnEnd);
    // If the target is out of view, scroll so that it becomes the top-left
    // visible cell. `getVisibleCells` reads the view of the (now) selected
    // sheet; the DOM scroller follows the model view on refresh.
    const { topLeftCell, bottomRightCell } = worksheet.getVisibleCells();
    if (
      row < topLeftCell.row ||
      row > bottomRightCell.row ||
      column < topLeftCell.column ||
      column > bottomRightCell.column
    ) {
      worksheet.model.setTopLeftVisibleCell(row, column);
    }
    worksheet.refresh();
    worksheet.renderSheet();
  }
}
