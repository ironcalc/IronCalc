import type { HorizontalAlignment, VerticalAlignment } from "@ironcalc/wasm";
import type { MergeCellsOperation } from "../Toolbar/Toolbar";

/**
 * Actions a host application can run on the workbook through the widget's
 * imperative handle (`IronCalcHandle.workbook`).
 *
 * Every action operates on the current selection, mutates the model and
 * repaints the widget. They are the same functions the toolbar and the
 * keyboard shortcuts use, so the host gets identical behaviour. Mutating
 * actions are ignored when the widget is not editable (`canEdit` is false).
 */
export interface WorkbookActions {
  refresh: () => void;
  undo: () => void;
  redo: () => void;
  copy: () => void;
  cut: () => void;
  paste: () => void;
  toggleBold: () => void;
  toggleItalic: () => void;
  toggleUnderline: () => void;
  toggleStrikethrough: () => void;
  setHorizontalAlignment: (alignment: HorizontalAlignment) => void;
  setVerticalAlignment: (alignment: VerticalAlignment) => void;
  setWrapText: (wrap: boolean) => void;
  setNumberFormat: (numberFormat: string) => void;
  setFontSize: (size: number) => void;
  increaseFontSize: (delta: number) => void;
  clearFormatting: () => void;
  insertRowsAbove: () => void;
  insertRowsBelow: () => void;
  insertColumnsLeft: () => void;
  insertColumnsRight: () => void;
  openConditionalFormatting: () => void;
  openNamedRanges: () => void;
  openThemes: () => void;
  openNamedStyles: () => void;
  openLinkDialog: () => void;
  mergeCells: (operation: MergeCellsOperation) => void;
  getMergeCellsState: () => {
    canMerge: boolean;
    canMergeAcross: boolean;
    canMergeDown: boolean;
    canUnmerge: boolean;
  };
  openCustomNumberFormat: () => void;
  downloadSelectionAsPng: () => void;
}
