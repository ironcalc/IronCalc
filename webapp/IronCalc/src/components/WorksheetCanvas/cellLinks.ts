import type { CellLink, Link } from "@ironcalc/wasm";
import { columnNumberFromName } from "@ironcalc/wasm";
import { ALLOWED_SCHEMES } from "../LinkDialog/util";
import {
  headerColumnWidth,
  headerRowHeight,
  LAST_COLUMN,
  LAST_ROW,
} from "./constants";
import type WorksheetCanvas from "./worksheetCanvas";

export interface LinkTooltipTexts {
  copyLink: string;
  editLink: string;
  breakLink: string;
}

export interface CellLinksOptions {
  tooltip: HTMLDivElement;
  onEditLink?: (row: number, column: number, link: Link) => void;
  onDeleteLink?: (row: number, column: number) => void;
  texts: LinkTooltipTexts;
}

// The geometry of a rendered cell with a link (a subset of the worksheet
// canvas cell text properties).
interface LinkCell {
  row: number;
  column: number;
  x: number;
  y: number;
  width: number;
  height: number;
  link: CellLink | null;
}

// Handles the cell links of the selected sheet: the pointer cursor, the hover
// tooltip (with its copy/edit/break actions) and following a link.
export class CellLinks {
  private worksheet: WorksheetCanvas;
  private tooltip: HTMLDivElement;
  private onEditLink?: (row: number, column: number, link: Link) => void;
  private onDeleteLink?: (row: number, column: number) => void;
  private texts: LinkTooltipTexts;
  // links in the selected sheet keyed by "row-column"
  private links: Map<string, CellLink>;
  // the cell whose link tooltip is currently shown (null if hidden)
  private tooltipCell: { row: number; column: number } | null;
  private hideTooltipTimeout: ReturnType<typeof setTimeout> | null;

  constructor(worksheet: WorksheetCanvas, options: CellLinksOptions) {
    this.worksheet = worksheet;
    this.tooltip = options.tooltip;
    this.onEditLink = options.onEditLink;
    this.onDeleteLink = options.onDeleteLink;
    this.texts = options.texts;
    this.links = new Map<string, CellLink>();
    this.tooltipCell = null;
    this.hideTooltipTimeout = null;
    this.attachHandlers();
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
    container.onpointermove = (event) => {
      if (this.tooltip.contains(event.target as Node)) {
        this.cancelHideTooltip();
        return;
      }
      const rect = canvas.getBoundingClientRect();
      const cell = this.getLinkCellAt(
        event.clientX - rect.left,
        event.clientY - rect.top,
      );
      if (cell?.link) {
        canvas.style.cursor = "pointer";
        this.cancelHideTooltip();
        if (
          this.tooltipCell?.row !== cell.row ||
          this.tooltipCell?.column !== cell.column
        ) {
          this.showTooltip(cell);
        }
      } else {
        canvas.style.cursor = "";
        this.scheduleHideTooltip();
      }
    };
    container.onpointerleave = () => {
      canvas.style.cursor = "";
      this.scheduleHideTooltip();
    };
    // Keep the tooltip open while the pointer is over it and make sure
    // interacting with it does not select cells underneath.
    this.tooltip.onpointerenter = () => {
      this.cancelHideTooltip();
    };
    this.tooltip.onpointerleave = () => {
      this.scheduleHideTooltip();
    };
    this.tooltip.onpointerdown = (event) => {
      event.stopPropagation();
    };
  }

  // Returns the visible cell with a link at canvas coordinates (x, y), if any
  private getLinkCellAt(x: number, y: number): LinkCell | null {
    for (const cell of this.worksheet.cells) {
      if (
        cell.link &&
        x >= cell.x &&
        x <= cell.x + cell.width &&
        y >= cell.y &&
        y <= cell.y + cell.height
      ) {
        return cell;
      }
    }
    return null;
  }

  private linkLabel(link: Link): string {
    if (link.type === "Internal") {
      return link.location;
    }
    const target = link.target;
    try {
      const url = new URL(target);
      if (url.protocol === "http:" || url.protocol === "https:") {
        return url.hostname.replace(/^www\./, "");
      }
      if (url.protocol === "mailto:") {
        return url.pathname;
      }
    } catch {
      // not a valid URL: fall through and show the raw target
    }
    return target;
  }

  private followLink(cell: LinkCell): void {
    const link = cell.link;
    if (!link) {
      return;
    }
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
    this.hideTooltip();
    worksheet.refresh();
    worksheet.renderSheet();
  }

  private showTooltip(cell: LinkCell): void {
    const link = cell.link;
    if (!link) {
      return;
    }
    this.tooltipCell = { row: cell.row, column: cell.column };
    const tooltip = this.tooltip;
    tooltip.replaceChildren();

    const linkIcon = document.createElement("span");
    linkIcon.className = "ic-worksheet-link-tooltip-icon";
    linkIcon.innerHTML =
      '<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>';
    tooltip.appendChild(linkIcon);

    const label = document.createElement("button");
    label.type = "button";
    label.className = "ic-worksheet-link-tooltip-label";
    label.textContent = this.linkLabel(link);
    label.title = link.tooltip || "";
    label.addEventListener("click", () => {
      this.followLink(cell);
    });
    tooltip.appendChild(label);

    const copyButton = document.createElement("button");
    copyButton.type = "button";
    copyButton.className = "ic-worksheet-link-tooltip-button";
    copyButton.title = this.texts.copyLink;
    copyButton.innerHTML =
      '<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>';
    copyButton.addEventListener("click", () => {
      const text = link.type === "External" ? link.target : link.location;
      navigator.clipboard?.writeText(text).catch(() => {
        // clipboard access denied or unavailable: nothing to do
      });
    });
    tooltip.appendChild(copyButton);

    // Dynamic links (created by formulas like HYPERLINK) cannot be edited:
    // only the formula itself can change them.
    if (this.onEditLink && !link.dynamic) {
      const editButton = document.createElement("button");
      editButton.type = "button";
      editButton.className = "ic-worksheet-link-tooltip-button";
      editButton.title = this.texts.editLink;
      editButton.innerHTML =
        '<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z"/></svg>';
      editButton.addEventListener("click", () => {
        this.hideTooltip();
        this.onEditLink?.(cell.row, cell.column, link);
      });
      tooltip.appendChild(editButton);
    }

    if (this.onDeleteLink && !link.dynamic) {
      const breakButton = document.createElement("button");
      breakButton.type = "button";
      breakButton.className = "ic-worksheet-link-tooltip-button";
      breakButton.title = this.texts.breakLink;
      breakButton.innerHTML =
        '<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 17H7A5 5 0 0 1 7 7"/><path d="M15 7h2a5 5 0 0 1 4 8"/><line x1="8" x2="12" y1="12" y2="12"/><line x1="2" x2="22" y1="2" y2="22"/></svg>';
      breakButton.addEventListener("click", () => {
        this.hideTooltip();
        this.onDeleteLink?.(cell.row, cell.column);
      });
      tooltip.appendChild(breakButton);
    }

    // Position the tooltip above the cell (or below if there is no room),
    // flush with the cell edge: any gap in between would make the pointer
    // cross a neighboring cell on its way to the tooltip, hiding it (or
    // showing the neighbor's tooltip) before it can be reached.
    tooltip.style.visibility = "hidden";
    tooltip.style.display = "flex";
    const tooltipHeight = tooltip.offsetHeight || 26;
    let top = cell.y - tooltipHeight + 1;
    if (top < headerRowHeight) {
      top = cell.y + cell.height - 1;
    }
    const left = Math.max(headerColumnWidth + 2, cell.x);
    tooltip.style.left = `${left}px`;
    tooltip.style.top = `${top}px`;
    tooltip.style.visibility = "visible";
  }

  hideTooltip(): void {
    this.cancelHideTooltip();
    this.tooltipCell = null;
    this.tooltip.style.display = "none";
  }

  private scheduleHideTooltip(): void {
    if (this.tooltipCell === null || this.hideTooltipTimeout !== null) {
      return;
    }
    this.hideTooltipTimeout = setTimeout(() => {
      this.hideTooltipTimeout = null;
      this.hideTooltip();
    }, 300);
  }

  private cancelHideTooltip(): void {
    if (this.hideTooltipTimeout !== null) {
      clearTimeout(this.hideTooltipTimeout);
      this.hideTooltipTimeout = null;
    }
  }
}
