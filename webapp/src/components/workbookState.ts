import type { CellStyle } from "@ironcalc/wasm";

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

type AreaStyles = CellStyle[][];

export class WorkbookState {
  private extendToArea: Area | null;
  private copyStyles: AreaStyles | null;

  constructor() {
    this.extendToArea = null;
    this.copyStyles = null;
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
}
