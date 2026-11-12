// This are properties of the workbook that are not permanently stored
// They only happen at 'runtime' while the workbook is being used:
//
// * What are we editing
// * Are we copying styles?
// * Are we extending a cell? (by pulling the cell outline handle down, for instance)
// * The zoom of the sheet
//
// Editing the cell is the most complex operation.
//
// * What cell are we editing?
// * Are we doing that from the cell editor or the formula editor?
// * What is the text content of the cell right now
// * The active ranges can technically be computed from the text.
//   Those are the ranges or cells that appear in the formula

import type { CellStyle } from "@ironcalc/wasm";

export interface SheetRange {
  sheet: number;
  rowStart: number;
  rowEnd: number;
  columnStart: number;
  columnEnd: number;
}

export type CutRange = SheetRange;

export enum AreaType {
  rowsDown = 0,
  columnsRight = 1,
  rowsUp = 2,
  columnsLeft = 3,
}

export interface Area {
  type: AreaType;
  rowStart: number;
  rowEnd: number;
  columnStart: number;
  columnEnd: number;
}

// Active ranges are ranges in the sheet that are highlighted when editing a formula
export interface ActiveRange {
  sheet: number;
  rowStart: number;
  rowEnd: number;
  columnStart: number;
  columnEnd: number;
  color: string;
}

export interface ReferencedRange {
  range: SheetRange;
  str: string;
  anchorRow: number;
  anchorColumn: number;
}

type Focus = "cell" | "formula-bar";
type EditorMode = "accept" | "edit";

// In "edit" mode arrow keys will move you around the text in the editor
// In "accept" mode arrow keys will accept the content and move to the next cell or select another cell

// The cell that we are editing
export interface EditingCell {
  sheet: number;
  row: number;
  column: number;
  // raw text in the editor
  text: string;
  // position of the cursor
  cursorStart: number;
  cursorEnd: number;
  // referenced range
  referencedRange: ReferencedRange | null;
  focus: Focus;
  activeRanges: ActiveRange[];
  mode: EditorMode;
  editorWidth: number;
  editorHeight: number;
}

// Those are styles that are copied
type AreaStyles = CellStyle[][];

// The zoom of the sheet, in percent (Excel's range)
export const MIN_ZOOM = 10;
export const MAX_ZOOM = 400;
const ZOOM_STEP = 10;

export function clampZoom(zoom: number): number {
  return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom));
}

// The next zoom levels in steps of 10%, from the current one: 105% goes to
// 110% and to 100%
export function zoomInLevel(zoom: number): number {
  return clampZoom((Math.floor(zoom / ZOOM_STEP) + 1) * ZOOM_STEP);
}

export function zoomOutLevel(zoom: number): number {
  return clampZoom((Math.ceil(zoom / ZOOM_STEP) - 1) * ZOOM_STEP);
}

export class WorkbookState {
  private extendToArea: Area | null;
  private copyStyles: AreaStyles | null;
  private cell: EditingCell | null;
  private cutRange: CutRange | null;
  private selecting: boolean;
  private zoom: number;

  constructor() {
    // the extendTo area is the area we are covering
    this.extendToArea = null;
    this.copyStyles = null;
    this.cell = null;
    this.cutRange = null;
    this.selecting = false;
    this.zoom = 100;
  }

  // The zoom of the sheet, in percent
  getZoom(): number {
    return this.zoom;
  }

  setZoom(zoom: number): void {
    this.zoom = clampZoom(zoom);
  }

  isSelecting(): boolean {
    return this.selecting;
  }

  setSelecting(value: boolean): void {
    this.selecting = value;
  }

  getExtendToArea(): Area | null {
    return this.extendToArea;
  }

  clearExtendToArea(): void {
    this.extendToArea = null;
  }

  setExtendToArea(area: Area): void {
    this.extendToArea = area;
  }

  setCopyStyles(styles: AreaStyles | null): void {
    this.copyStyles = styles;
  }

  getCopyStyles(): AreaStyles | null {
    return this.copyStyles;
  }

  setActiveRanges(activeRanges: ActiveRange[]) {
    if (!this.cell) {
      return;
    }
    this.cell.activeRanges = activeRanges;
  }

  getActiveRanges(): ActiveRange[] {
    return this.cell?.activeRanges || [];
  }

  getEditingCell(): EditingCell | null {
    return this.cell;
  }

  setEditingCell(cell: EditingCell) {
    this.cell = cell;
  }

  clearEditingCell() {
    this.cell = null;
  }

  isCellEditorActive(): boolean {
    if (this.cell) {
      return this.cell.focus === "cell";
    }
    return false;
  }

  isFormulaEditorActive(): boolean {
    if (this.cell) {
      return this.cell.focus === "formula-bar";
    }
    return false;
  }

  getEditingText(): string {
    const cell = this.cell;
    if (cell) {
      const str = cell.referencedRange?.str || "";
      if (str === "") {
        return cell.text;
      } else {
        // insert str at the cursor position
        const cursorPos = cell.cursorStart;
        return (
          cell.text.substring(0, cursorPos) +
          str +
          cell.text.substring(cursorPos)
        );
      }
    }
    return "";
  }

  setCutRange(range: CutRange): void {
    this.cutRange = range;
  }

  clearCutRange(): void {
    this.cutRange = null;
  }

  getCutRange(): CutRange | null {
    return this.cutRange;
  }
}
