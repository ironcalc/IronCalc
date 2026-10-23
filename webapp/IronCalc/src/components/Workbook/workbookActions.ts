import type { HorizontalAlignment, VerticalAlignment } from "@ironcalc/wasm";

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
  /** Repaints the widget. Call it after changing the model directly. */
  refresh: () => void;
  undo: () => void;
  redo: () => void;
  /** Toggles bold on the selection, based on the selected cell's style. */
  toggleBold: () => void;
  toggleItalic: () => void;
  toggleUnderline: () => void;
  toggleStrikethrough: () => void;
  setHorizontalAlignment: (alignment: HorizontalAlignment) => void;
  setVerticalAlignment: (alignment: VerticalAlignment) => void;
  setWrapText: (wrap: boolean) => void;
  setNumberFormat: (numberFormat: string) => void;
  setFontSize: (size: number) => void;
  /** Changes the font size by `delta` points (negative shrinks it). */
  increaseFontSize: (delta: number) => void;
  clearFormatting: () => void;
}
