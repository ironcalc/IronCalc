import type {
  BorderOptions,
  ClipboardCell,
  IronCalcTheme,
  Link,
  Model,
  WorksheetProperties,
} from "@ironcalc/wasm";
import { type Color, getThemeList } from "@ironcalc/wasm";
import {
  type ForwardedRef,
  forwardRef,
  useCallback,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import {
  CLIPBOARD_ID_SESSION_STORAGE_KEY,
  getNewClipboardId,
} from "../clipboard";
import FormulaBar from "../FormulaBar/FormulaBar";
import type { SaveError } from "../RightDrawer/NamedStyles/EditNamedStyle";
import RightDrawer, {
  DEFAULT_DRAWER_WIDTH,
  type DrawerType,
} from "../RightDrawer/RightDrawer";
import SheetTabBar from "../SheetTabBar";
import Toolbar, { type MergeCellsOperation } from "../Toolbar/Toolbar";
import {
  getCellAddress,
  getEditorSize,
  getFullRangeToString,
  type NavigationKey,
} from "../util";
import Worksheet from "../Worksheet/Worksheet";
import { LAST_COLUMN, LAST_ROW } from "../WorksheetCanvas/constants";
import type WorksheetCanvas from "../WorksheetCanvas/worksheetCanvas";
import { devicePixelRatio } from "../WorksheetCanvas/worksheetCanvas";
import type { WorkbookState } from "../workbookState";
import useKeyboardNavigation from "./useKeyboardNavigation";
import type { WorkbookActions } from "./workbookActions";
import "./workbook.css";
import FormatPicker from "../FormatMenu/FormatPicker";
import { LinkDialog } from "../LinkDialog/LinkDialog";
import { Alert } from "../Modal";

function colorToParam(color: Color): string {
  if (color === undefined) {
    return "";
  }
  if (typeof color === "string") {
    return color;
  }
  return `[${color[0]}, ${color[1]}]`;
}

interface WorkbookProperties {
  model: Model;
  workbookState: WorkbookState;
  /** When false, the toolbar is hidden, the formula bar is read-only and all edits are blocked. */
  canEdit?: boolean;
}

// The ref exposes the `WorkbookActions` so the host application can drive
// the workbook (see `IronCalcHandle.workbook`).
const Workbook = (
  props: WorkbookProperties,
  ref: ForwardedRef<WorkbookActions>,
) => {
  const { model, workbookState, canEdit = true } = props;
  const { t } = useTranslation();
  const rootRef = useRef<HTMLDivElement | null>(null);
  const worksheetRef = useRef<{
    getCanvas: () => WorksheetCanvas | null;
  }>(null);
  const lastClipboardJson = useRef<string | null>(null);

  // Calling `setRedrawId((id) => id + 1);` forces a redraw
  // This is needed because `model` or `workbookState` can change without React being aware of it
  const setRedrawId = useState(0)[1];

  const [alertDialog, setAlertDialog] = useState<{
    title: string;
    message: string;
  } | null>(null);
  const [isFormatPickerOpen, setFormatPickerOpen] = useState(false);
  const [isDrawerOpen, setDrawerOpen] = useState(false);
  const [drawerWidth, setDrawerWidth] = useState(DEFAULT_DRAWER_WIDTH);
  const [drawerType, setDrawerType] = useState<DrawerType>("namedRanges");
  // The cell the link dialog operates on (null when the dialog is closed)
  const [linkDialogCell, setLinkDialogCell] = useState<{
    sheet: number;
    row: number;
    column: number;
  } | null>(null);

  const openDrawer = useCallback((type: DrawerType) => {
    setDrawerType(type);
    setDrawerOpen(true);
  }, []);

  const worksheets = model.getWorksheetsProperties();
  const info = worksheets.map(
    ({ name, color, sheet_id, state }: WorksheetProperties) => {
      return {
        name,
        color: model.resolveColor(color) || "var(--palette-common-white)",
        sheetId: sheet_id,
        state,
      };
    },
  );
  const focusWorkbook = useCallback(() => {
    const active = document.activeElement as HTMLElement | null;
    // FIXME: Horrible HACK for now
    // Prevents the workbook from stealing focus when the user is interacting with the color picker, border picker, advanced color picker, or a modal dialog
    if (
      active?.closest(
        ".ic-color-picker, .ic-advanced-color-picker-panel, .ic-border-picker, .ic-modal-dialog",
      )
    ) {
      return;
    }

    if (rootRef.current) {
      rootRef.current.focus({ preventScroll: true });
      // HACK: We need to select something inside the root for onCopy to work
      const selection = window.getSelection();
      if (selection) {
        selection.empty();
        const range = new Range();
        range.setStart(rootRef.current.firstChild as Node, 0);
        range.setEnd(rootRef.current.firstChild as Node, 0);
        selection.addRange(range);
      }
    }
  }, []);
  const onRedo = () => {
    model.redo();
    setRedrawId((id) => id + 1);
  };

  const onUndo = () => {
    model.undo();
    setRedrawId((id) => id + 1);
  };

  const updateRangeStyle = (stylePath: string, value: string) => {
    const {
      sheet,
      range: [rowStart, columnStart, rowEnd, columnEnd],
    } = model.getSelectedView();
    const row = Math.min(rowStart, rowEnd);
    const column = Math.min(columnStart, columnEnd);
    const range = {
      sheet,
      row,
      column,
      width: Math.abs(columnEnd - columnStart) + 1,
      height: Math.abs(rowEnd - rowStart) + 1,
    };
    model.updateRangeStyle(range, stylePath, value);
    setRedrawId((id) => id + 1);
  };

  const onToggleUnderline = (value: boolean) => {
    updateRangeStyle("font.u", `${value}`);
  };

  const onToggleItalic = (value: boolean) => {
    updateRangeStyle("font.i", `${value}`);
  };

  const onToggleBold = (value: boolean) => {
    updateRangeStyle("font.b", `${value}`);
  };

  const onToggleStrike = (value: boolean) => {
    updateRangeStyle("font.strike", `${value}`);
  };

  const onToggleHorizontalAlign = (value: string) => {
    updateRangeStyle("alignment.horizontal", value);
  };

  const onToggleVerticalAlign = (value: string) => {
    updateRangeStyle("alignment.vertical", value);
  };

  const onToggleWrapText = (value: boolean) => {
    updateRangeStyle("alignment.wrap_text", `${value}`);
  };

  // The selected area, normalized (row/column is the top-left corner)
  const getSelectedArea = () => {
    const {
      sheet,
      range: [rowStart, columnStart, rowEnd, columnEnd],
    } = model.getSelectedView();
    return {
      sheet,
      row: Math.min(rowStart, rowEnd),
      column: Math.min(columnStart, columnEnd),
      width: Math.abs(columnEnd - columnStart) + 1,
      height: Math.abs(rowEnd - rowStart) + 1,
    };
  };

  const selectionIntersectsMergedCells = () => {
    const area = getSelectedArea();
    return model
      .getMergedCells(area.sheet)
      .some(
        (m) =>
          m.row <= area.row + area.height - 1 &&
          m.row + m.height - 1 >= area.row &&
          m.column <= area.column + area.width - 1 &&
          m.column + m.width - 1 >= area.column,
      );
  };

  // The menu merges a selection of more than one cell (as a single merged
  // cell, centered, across or down) and unmerges a selection that intersects
  // merged cells.
  const onMergeCells = (operation: MergeCellsOperation) => {
    const area = getSelectedArea();
    try {
      switch (operation) {
        case "merge":
          model.mergeCells(area);
          break;
        case "merge_center":
          model.mergeCellsCenter(area);
          break;
        case "merge_across":
          model.mergeCellsAcross(area);
          break;
        case "merge_down":
          model.mergeCellsDown(area);
          break;
        case "unmerge":
          model.unmergeCells(area);
          break;
      }
    } catch (e) {
      if (`${e}`.includes("more than one cell has content")) {
        setAlertDialog({
          title: t("error_dialog.error_merging_cells"),
          message: t("error_dialog.error_merging_cells_content"),
        });
      }
      // other failures (e.g. merging over an array formula) stay silent
    }
    setRedrawId((id) => id + 1);
  };

  // A rejected paste (e.g. it would partially cover a merged cell) gets a
  // dialog; the merged-cell case is localized, the rest show the raw error.
  const showPasteError = (error: unknown) => {
    const message = `${error}`;
    setAlertDialog({
      title: t("error_dialog.error_paste"),
      message: message.includes("merged cell")
        ? t("error_dialog.error_paste_merged")
        : message,
    });
  };

  // Full-row and full-column selections can be neither merged nor unmerged.
  const getMergeCellsState = () => {
    const area = getSelectedArea();
    if (area.width >= LAST_COLUMN || area.height >= LAST_ROW) {
      return {
        canMerge: false,
        canMergeAcross: false,
        canMergeDown: false,
        canUnmerge: false,
      };
    }
    const canUnmerge = selectionIntersectsMergedCells();
    const canMerge = !canUnmerge && area.width * area.height > 1;
    return {
      canMerge,
      // merging across (down) merges each row (column) separately, so it
      // needs more than one column (row)
      canMergeAcross: canMerge && area.width > 1,
      canMergeDown: canMerge && area.height > 1,
      canUnmerge,
    };
  };

  const onTextColorPicked = (color: Color) => {
    updateRangeStyle("font.color", colorToParam(color));
  };

  const onFillColorPicked = (color: Color) => {
    updateRangeStyle("fill.color", colorToParam(color));
  };

  const onNumberFormatPicked = (numberFmt: string) => {
    updateRangeStyle("num_fmt", numberFmt);
  };

  const onIncreaseFontSize = (delta: number) => {
    updateRangeStyle("font.size_delta", `${delta}`);
  };

  const onSetFontSize = (size: number) => {
    updateRangeStyle("font.size", `${size}`);
  };

  const handlePaste = useCallback(async (): Promise<void> => {
    focusWorkbook();
    try {
      const items = await navigator.clipboard.read();
      const dt = new DataTransfer();
      for (const item of items) {
        for (const type of item.types) {
          dt.setData(type, await (await item.getType(type)).text());
        }
      }
      if (!dt.getData("application/json") && lastClipboardJson.current) {
        dt.setData("application/json", lastClipboardJson.current);
      }
      rootRef.current?.dispatchEvent(
        new ClipboardEvent("paste", {
          clipboardData: dt,
          bubbles: true,
          cancelable: true,
        }),
      );
    } catch {
      // fall back to text-only if clipboard.read() is unavailable
      try {
        const text = await navigator.clipboard.readText();
        const dt = new DataTransfer();
        dt.setData("text/plain", text);
        if (lastClipboardJson.current) {
          dt.setData("application/json", lastClipboardJson.current);
        }
        rootRef.current?.dispatchEvent(
          new ClipboardEvent("paste", {
            clipboardData: dt,
            bubbles: true,
            cancelable: true,
          }),
        );
      } catch {
        setAlertDialog({
          title: t("error_dialog.error_deleting_cells"),
          message: t("error_dialog.error_clipboard_paste"),
        });
      }
    }
  }, [focusWorkbook, t]);

  const onCopyStyles = () => {
    const {
      sheet,
      range: [rowStart, columnStart, rowEnd, columnEnd],
    } = model.getSelectedView();
    const row1 = Math.min(rowStart, rowEnd);
    const column1 = Math.min(columnStart, columnEnd);
    const row2 = Math.max(rowStart, rowEnd);
    const column2 = Math.max(columnStart, columnEnd);

    const styles = [];
    for (let row = row1; row <= row2; row++) {
      const styleRow = [];
      for (let column = column1; column <= column2; column++) {
        styleRow.push(model.getCellStyle(sheet, row, column).style);
      }
      styles.push(styleRow);
    }
    workbookState.setCopyStyles(styles);
    // FIXME: This is so that the cursor indicates there are styles to be pasted
    const el = rootRef.current?.getElementsByClassName(
      "ic-worksheet-sheet-container",
    )[0];
    if (el) {
      // Taken from lucide icons: <PaintRoller /> and rotated.
      const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-paint-roller" style="transform:rotate(-8deg)"><rect width="16" height="6" x="2" y="2" rx="2"></rect><path d="M10 16v-2a2 2 0 0 1 2-2h8a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2"></path><rect width="4" height="6" x="8" y="16" rx="1"></rect></svg>`;
      (el as HTMLElement).style.cursor =
        `url('data:image/svg+xml;utf8,${encodeURIComponent(svg)}'), auto`;
    }
  };

  const fmtSettings = model.getFmtSettings();
  const themesRef = useRef<IronCalcTheme[] | null>(null);
  if (themesRef.current === null) {
    themesRef.current = getThemeList();
  }
  const themes = themesRef.current ?? [];

  const handleThemePicked = (theme: IronCalcTheme) => {
    model.setTheme(theme);
    setRedrawId((id) => id + 1);
  };

  // Wraps a mutating action so it is a no-op when the workbook is read-only
  const whenEditable =
    <A extends unknown[]>(action: (...args: A) => void) =>
    (...args: A): void => {
      if (canEdit) {
        action(...args);
      }
    };

  // Toggles a boolean font property of the selection, based on the style of
  // the selected cell (the one the toolbar displays)
  const toggleFontProperty = (
    property: "b" | "i" | "u" | "strike",
    apply: (value: boolean) => void,
  ) => {
    const { sheet, row, column } = model.getSelectedView();
    const value = model.getCellStyle(sheet, row, column).style.font[property];
    apply(!value);
  };

  // Runs an insert on the model, showing a dialog if it fails (for instance
  // when it would split a merged cell)
  const insertRowsOrColumns = (
    insert: () => void,
    what: "rows" | "columns",
  ) => {
    try {
      insert();
    } catch {
      const title = t(`error_dialog.error_inserting_${what}`);
      setAlertDialog({ title, message: title });
    }
    setRedrawId((id) => id + 1);
  };

  // The one implementation behind the toolbar, the keyboard shortcuts and the
  // host application (through the imperative handle)
  const actions: WorkbookActions = {
    refresh: () => {
      setRedrawId((id) => id + 1);
    },
    undo: whenEditable(onUndo),
    redo: whenEditable(onRedo),
    // Copy and cut go through the browser's clipboard events handled by the
    // root element below, so the workbook needs the focus and a selection
    copy: () => {
      focusWorkbook();
      document.execCommand("copy");
    },
    cut: whenEditable(() => {
      focusWorkbook();
      document.execCommand("cut");
    }),
    paste: whenEditable(() => {
      handlePaste();
    }),
    toggleBold: whenEditable(() => toggleFontProperty("b", onToggleBold)),
    toggleItalic: whenEditable(() => toggleFontProperty("i", onToggleItalic)),
    toggleUnderline: whenEditable(() =>
      toggleFontProperty("u", onToggleUnderline),
    ),
    toggleStrikethrough: whenEditable(() =>
      toggleFontProperty("strike", onToggleStrike),
    ),
    setHorizontalAlignment: whenEditable(onToggleHorizontalAlign),
    setVerticalAlignment: whenEditable(onToggleVerticalAlign),
    setWrapText: whenEditable(onToggleWrapText),
    setNumberFormat: whenEditable(onNumberFormatPicked),
    setFontSize: whenEditable(onSetFontSize),
    increaseFontSize: whenEditable(onIncreaseFontSize),
    clearFormatting: whenEditable(() => {
      const {
        sheet,
        range: [rowStart, columnStart, rowEnd, columnEnd],
      } = model.getSelectedView();
      model.rangeClearFormatting(
        sheet,
        rowStart,
        columnStart,
        rowEnd,
        columnEnd,
      );
      setRedrawId((id) => id + 1);
    }),
    insertRowsAbove: whenEditable(() => {
      const { sheet, row, height } = getSelectedArea();
      insertRowsOrColumns(() => model.insertRows(sheet, row, height), "rows");
    }),
    insertRowsBelow: whenEditable(() => {
      const { sheet, row, height } = getSelectedArea();
      insertRowsOrColumns(
        () => model.insertRows(sheet, row + height, height),
        "rows",
      );
    }),
    insertColumnsLeft: whenEditable(() => {
      const { sheet, column, width } = getSelectedArea();
      insertRowsOrColumns(
        () => model.insertColumns(sheet, column, width),
        "columns",
      );
    }),
    insertColumnsRight: whenEditable(() => {
      const { sheet, column, width } = getSelectedArea();
      insertRowsOrColumns(
        () => model.insertColumns(sheet, column + width, width),
        "columns",
      );
    }),
    openConditionalFormatting: whenEditable(() => {
      openDrawer("conditionalFormatting");
    }),
    openNamedRanges: whenEditable(() => {
      openDrawer("namedRanges");
    }),
    openThemes: whenEditable(() => {
      openDrawer("themes");
    }),
    mergeCells: whenEditable(onMergeCells),
    getMergeCellsState,
    openCustomNumberFormat: whenEditable(() => {
      setFormatPickerOpen(true);
    }),
    openLinkDialog: whenEditable(() => {
      const { sheet, row, column } = model.getSelectedView();
      openLinkDialog(sheet, row, column);
    }),
    downloadSelectionAsPng: () => {
      // creates a new canvas element in the visible part of the the selected area
      const worksheetCanvas = worksheetRef.current?.getCanvas();
      if (!worksheetCanvas) {
        return;
      }
      const {
        range: [rowStart, columnStart, rowEnd, columnEnd],
      } = model.getSelectedView();
      // NB: cells outside of the displayed area are not rendered
      // I think the only reasonable way to do this would be server side.
      let [x, y] = worksheetCanvas.getCoordinatesByCell(rowStart, columnStart);
      const [x1, y1] = worksheetCanvas.getCoordinatesByCell(
        rowEnd + 1,
        columnEnd + 1,
      );
      const width = (x1 - x) * devicePixelRatio;
      const height = (y1 - y) * devicePixelRatio;
      x *= devicePixelRatio;
      y *= devicePixelRatio;

      const capturedCanvas = document.createElement("canvas");
      capturedCanvas.width = width;
      capturedCanvas.height = height;
      const ctx = capturedCanvas.getContext("2d");
      if (!ctx) {
        return;
      }

      ctx.drawImage(
        worksheetCanvas.canvas,
        x,
        y,
        width,
        height,
        0,
        0,
        width,
        height,
      );

      const downloadLink = document.createElement("a");
      downloadLink.href = capturedCanvas.toDataURL("image/png");
      downloadLink.download = "ironcalc.png";
      downloadLink.click();
    },
  };

  useImperativeHandle(ref, () => actions);

  // FIXME: I *think* we should have only one on onKeyPressed function that goes to
  // the Rust backend
  const { onKeyDown } = useKeyboardNavigation({
    canEdit,
    onCellsDeleted: (): void => {
      const {
        sheet,
        range: [rowStart, columnStart, rowEnd, columnEnd],
      } = model.getSelectedView();
      const row = Math.min(rowStart, rowEnd);
      const column = Math.min(columnStart, columnEnd);

      const width = Math.abs(columnEnd - columnStart);
      const height = Math.abs(rowEnd - rowStart);
      try {
        model.rangeClearContents(
          sheet,
          row,
          column,
          row + height,
          column + width,
        );
      } catch (e) {
        setAlertDialog({
          title: t("error_dialog.error_deleting_cells"),
          message: `${e}`,
        });
      }
      setRedrawId((id) => id + 1);
    },
    onExpandAreaSelectedKeyboard: (
      key: "ArrowRight" | "ArrowLeft" | "ArrowUp" | "ArrowDown",
    ): void => {
      model.onExpandSelectedRange(key);
      setRedrawId((id) => id + 1);
    },
    onEditKeyPressStart: (initText: string): void => {
      const { sheet, row, column } = model.getSelectedView();
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
        text: initText,
        cursorStart: initText.length,
        cursorEnd: initText.length,
        focus: "cell",
        referencedRange: null,
        activeRanges: [],
        mode: "accept",
        editorWidth,
        editorHeight,
      });
      setRedrawId((id) => id + 1);
    },
    onCellEditStart: (): void => {
      // User presses F2, we start editing at the edn of the text
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
        referencedRange: null,
        focus: "cell",
        activeRanges: [],
        mode: "edit",
        editorWidth,
        editorHeight,
      });
      setRedrawId((id) => id + 1);
    },
    onBold: actions.toggleBold,
    onItalic: actions.toggleItalic,
    onUnderline: actions.toggleUnderline,
    onNavigationToEdge: (direction: NavigationKey): void => {
      model.onNavigateToEdgeInDirection(direction);
      setRedrawId((id) => id + 1);
    },
    onPageDown: (): void => {
      model.onPageDown();
      setRedrawId((id) => id + 1);
    },
    onPageUp: (): void => {
      model.onPageUp();
      setRedrawId((id) => id + 1);
    },
    onArrowDown: (): void => {
      model.onArrowDown();
      setRedrawId((id) => id + 1);
    },
    onArrowUp: (): void => {
      model.onArrowUp();
      setRedrawId((id) => id + 1);
    },
    onArrowLeft: (): void => {
      model.onArrowLeft();
      setRedrawId((id) => id + 1);
    },
    onArrowRight: (): void => {
      model.onArrowRight();
      setRedrawId((id) => id + 1);
    },
    onKeyHome: (): void => {
      const cell = model.getSelectedCell();
      model.setSelectedCell(cell[1], 1);
      setRedrawId((id) => id + 1);
    },
    onKeyEnd: (): void => {
      const cell = model.getSelectedCell();
      model.setSelectedCell(cell[1], LAST_COLUMN);
      setRedrawId((id) => id + 1);
    },
    onUndo: actions.undo,
    onRedo: actions.redo,
    onNextSheet: (): void => {
      const nextSheet = model.getSelectedSheet() + 1;
      if (nextSheet >= model.getWorksheetsProperties().length) {
        model.setSelectedSheet(0);
      } else {
        model.setSelectedSheet(nextSheet);
      }
    },
    onPreviousSheet: (): void => {
      const nextSheet = model.getSelectedSheet() - 1;
      if (nextSheet < 0) {
        model.setSelectedSheet(model.getWorksheetsProperties().length - 1);
      } else {
        model.setSelectedSheet(nextSheet);
      }
    },
    onEscape: (): void => {
      workbookState.clearCutRange();
      workbookState.setCopyStyles(null);
      const el = rootRef.current?.getElementsByClassName(
        "ic-worksheet-sheet-container",
      )[0];
      if (el) {
        (el as HTMLElement).style.cursor = "auto";
      }
      setRedrawId((id) => id + 1);
    },
    onSelectColumn: (): void => {
      const { column } = model.getSelectedView();
      model.setSelectedRange(1, column, LAST_ROW, column);
      setRedrawId((id) => id + 1);
    },
    onSelectRow: (): void => {
      const { row } = model.getSelectedView();
      model.setSelectedRange(row, 1, row, LAST_COLUMN);
      setRedrawId((id) => id + 1);
    },
    root: rootRef,
  });

  // Keeps the keyboard focus on the workbook after a render, so the shortcuts
  // keep working after a click on the toolbar or a closed dialog. It must not
  // take the focus from somewhere else though: a render caused from the
  // outside (the host re-rendering or repainting after an external change)
  // would otherwise interrupt whatever the user is doing there, for instance
  // typing in another widget or in the host's own inputs.
  useEffect(() => {
    const root = rootRef.current;
    if (!root || workbookState.getEditingCell()) {
      return;
    }
    const active = document.activeElement;
    if (active && active !== document.body && !root.contains(active)) {
      return;
    }
    focusWorkbook();
  });

  const cellAddress = useCallback(() => {
    const {
      row,
      column,
      range: [rowStart, columnStart, rowEnd, columnEnd],
    } = model.getSelectedView();
    return getCellAddress(
      { rowStart, rowEnd, columnStart, columnEnd },
      { row, column },
    );
  }, [model]);

  // Returns the formula bar value and whether it can be edited
  // (false for array formulas)
  const getFormulaValue = (): [string, boolean] => {
    const cell = workbookState.getEditingCell();
    if (cell) {
      return [workbookState.getEditingText(), true];
    }
    const { sheet, row, column } = model.getSelectedView();
    const arrayStructure = model.getCellArrayStructure(sheet, row, column);
    if (arrayStructure === "SingleCell") {
      // single cell
    } else if ("DynamicAnchor" in arrayStructure) {
      // Anchor of a dynamic array
    } else if ("DynamicChild" in arrayStructure) {
      // Child of a dynamic array
    } else if ("ArrayAnchor" in arrayStructure) {
      return [`{${model.getCellContent(sheet, row, column)}}`, false];
    } else if ("ArrayChild" in arrayStructure) {
      const array = arrayStructure.ArrayChild;
      return [`{${model.getCellContent(sheet, array[0], array[1])}}`, false];
    }
    return [model.getCellContent(sheet, row, column), true];
  };

  const [formulaValue, canEditFormula] = getFormulaValue();

  const getCellStyle = useCallback(() => {
    const { sheet, row, column } = model.getSelectedView();
    return model.getCellStyle(sheet, row, column).style;
  }, [model]);

  const style = getCellStyle();
  const currentTheme = model.getTheme();

  const openLinkDialog = (sheet: number, row: number, column: number): void => {
    setLinkDialogCell({ sheet, row, column });
  };

  const onSaveLink = (link: Link, label: string): void => {
    if (!linkDialogCell) {
      return;
    }
    const { sheet, row, column } = linkDialogCell;
    // The cell content is the displayed text of the link. If the label is
    // empty, fall back to the link target/location (for emails, just the
    // address without the mailto parameters).
    const text =
      label.trim() ||
      (link.type === "External"
        ? link.target.replace(/^mailto:/, "").split("?")[0]
        : link.location);
    // Sets the link, the cell content and (for new links) the link style
    // in a single undo step.
    model.setCellLink(sheet, row, column, link, text);
    setRedrawId((id) => id + 1);
  };

  const onDeleteLink = (): void => {
    if (!linkDialogCell) {
      return;
    }
    const { sheet, row, column } = linkDialogCell;
    model.deleteCellLink(sheet, row, column);
    setRedrawId((id) => id + 1);
  };

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: This div needs to be focusable to handle keyboard events for the workbook
    <div
      className={`ic-workbook-container${canEdit ? "" : " ic-workbook-container--readonly"}`}
      ref={rootRef}
      onKeyDown={onKeyDown}
      // biome-ignore lint/a11y/noNoninteractiveTabindex: This div needs to be focusable to handle keyboard events for the workbook
      tabIndex={0}
      onClick={(event: React.MouseEvent) => {
        if (!workbookState.getEditingCell()) {
          focusWorkbook();
        } else {
          event.stopPropagation();
        }
      }}
      onPaste={(event: React.ClipboardEvent) => {
        if (!canEdit) {
          event.preventDefault();
          event.stopPropagation();
          return;
        }
        workbookState.clearCutRange();
        const { items } = event.clipboardData;
        if (!items) {
          return;
        }
        const mimeTypes = [
          "application/json",
          "text/plain",
          "text/csv",
          "text/html",
        ];
        let mimeType = null;
        let value = null;
        for (let index = 0; index < mimeTypes.length; index += 1) {
          mimeType = mimeTypes[index];
          value = event.clipboardData.getData(mimeType);
          if (value) {
            break;
          }
        }
        if (!mimeType || !value) {
          // No clipboard data to paste
          return;
        }
        if (mimeType === "application/json") {
          // We are copying from within the application
          const source = JSON.parse(value);
          // const clipboardId = sessionStorage.getItem(
          //   CLIPBOARD_ID_SESSION_STORAGE_KEY
          // );
          const data: Map<number, Map<number, ClipboardCell>> = new Map();
          const sheetData = source.sheetData;
          for (const row of Object.keys(sheetData)) {
            const dataRow = sheetData[row];
            const rowMap = new Map();
            for (const column of Object.keys(dataRow)) {
              rowMap.set(Number.parseInt(column, 10), dataRow[column]);
            }
            data.set(Number.parseInt(row, 10), rowMap);
          }
          try {
            model.pasteFromClipboard(
              source.sheet,
              source.area,
              data,
              source.type === "cut",
            );
            setRedrawId((id) => id + 1);
          } catch (e) {
            showPasteError(e);
          }
        } else if (mimeType === "text/plain") {
          const {
            sheet,
            range: [rowStart, columnStart, rowEnd, columnEnd],
          } = model.getSelectedView();
          const row = Math.min(rowStart, rowEnd);
          const column = Math.min(columnStart, columnEnd);
          const range = {
            sheet,
            row,
            column,
            width: Math.abs(columnEnd - columnStart) + 1,
            height: Math.abs(rowEnd - rowStart) + 1,
          };
          try {
            model.pasteCsvText(range, value);
            setRedrawId((id) => id + 1);
          } catch (e) {
            showPasteError(e);
          }
        } else {
          // NOT IMPLEMENTED
        }
        event.preventDefault();
        event.stopPropagation();
      }}
      onCopy={(event: React.ClipboardEvent) => {
        const data = model.copyToClipboard();
        const sheet = model.getSelectedSheet();
        // '2024-10-18T14:07:37.599Z'

        let clipboardId = sessionStorage.getItem(
          CLIPBOARD_ID_SESSION_STORAGE_KEY,
        );
        if (!clipboardId) {
          clipboardId = getNewClipboardId();
          sessionStorage.setItem(CLIPBOARD_ID_SESSION_STORAGE_KEY, clipboardId);
        }
        const sheetData: {
          [row: number]: {
            [column: number]: ClipboardCell;
          };
        } = {};
        data.data.forEach((value, row) => {
          const rowData: {
            [column: number]: ClipboardCell;
          } = {};
          value.forEach((val, column) => {
            rowData[column] = val;
          });
          sheetData[row] = rowData;
        });
        const clipboardJsonStr = JSON.stringify({
          type: "copy",
          area: data.range,
          sheetData,
          sheet,
          clipboardId,
        });
        lastClipboardJson.current = clipboardJsonStr;
        event.clipboardData.setData("text/plain", data.csv.trim());
        event.clipboardData.setData("application/json", clipboardJsonStr);
        event.preventDefault();
        event.stopPropagation();
      }}
      onCut={(event: React.ClipboardEvent) => {
        if (!canEdit) {
          event.preventDefault();
          event.stopPropagation();
          return;
        }
        const data = model.copyToClipboard();
        const sheet = model.getSelectedSheet();
        // '2024-10-18T14:07:37.599Z'

        let clipboardId = sessionStorage.getItem(
          CLIPBOARD_ID_SESSION_STORAGE_KEY,
        );
        if (!clipboardId) {
          clipboardId = getNewClipboardId();
          sessionStorage.setItem(CLIPBOARD_ID_SESSION_STORAGE_KEY, clipboardId);
        }
        const sheetData: {
          [row: number]: {
            [column: number]: ClipboardCell;
          };
        } = {};
        data.data.forEach((value, row) => {
          const rowData: {
            [column: number]: ClipboardCell;
          } = {};
          value.forEach((val, column) => {
            rowData[column] = val;
          });
          sheetData[row] = rowData;
        });
        const clipboardJsonStr = JSON.stringify({
          type: "cut",
          area: data.range,
          sheetData,
          sheet,
          clipboardId,
        });
        lastClipboardJson.current = clipboardJsonStr;
        event.clipboardData.setData("text/plain", data.csv);
        event.clipboardData.setData("application/json", clipboardJsonStr);
        workbookState.setCutRange({
          sheet: model.getSelectedSheet(),
          rowStart: data.range[0],
          rowEnd: data.range[2],
          columnStart: data.range[1],
          columnEnd: data.range[3],
        });
        event.preventDefault();
        event.stopPropagation();
        setRedrawId((id) => id + 1);
      }}
    >
      {canEdit && (
        <Toolbar
          canUndo={model.canUndo()}
          canRedo={model.canRedo()}
          onRedo={onRedo}
          onUndo={onUndo}
          onToggleUnderline={onToggleUnderline}
          onToggleBold={onToggleBold}
          onToggleItalic={onToggleItalic}
          onToggleStrike={onToggleStrike}
          onToggleHorizontalAlign={onToggleHorizontalAlign}
          onToggleVerticalAlign={onToggleVerticalAlign}
          onToggleWrapText={onToggleWrapText}
          onCopyStyles={onCopyStyles}
          onTextColorPicked={onTextColorPicked}
          onFillColorPicked={onFillColorPicked}
          onNumberFormatPicked={onNumberFormatPicked}
          onClearFormatting={actions.clearFormatting}
          onIncreaseFontSize={actions.increaseFontSize}
          onSetFontSize={actions.setFontSize}
          onDownloadPNG={actions.downloadSelectionAsPng}
          onBorderChanged={(border: BorderOptions): void => {
            const {
              sheet,
              range: [rowStart, columnStart, rowEnd, columnEnd],
            } = model.getSelectedView();
            const row = Math.min(rowStart, rowEnd);
            const column = Math.min(columnStart, columnEnd);

            const width = Math.abs(columnEnd - columnStart) + 1;
            const height = Math.abs(rowEnd - rowStart) + 1;
            const borderArea = {
              type: border.border,
              item: border,
            };
            model.setAreaWithBorder(
              { sheet, row, column, width, height },
              borderArea,
            );
            setRedrawId((id) => id + 1);
          }}
          fillColor={style.fill.color}
          fontColor={style.font.color}
          fontSize={style.font.sz}
          bold={style.font.b}
          underline={style.font.u}
          italic={style.font.i}
          strike={style.font.strike}
          horizontalAlign={
            style.alignment ? style.alignment.horizontal : "general"
          }
          verticalAlign={
            style.alignment?.vertical ? style.alignment.vertical : "bottom"
          }
          wrapText={style.alignment?.wrap_text || false}
          mergeCellsState={actions.getMergeCellsState()}
          onMergeCells={actions.mergeCells}
          canEdit={true}
          numFmt={style.num_fmt}
          showGridLines={model.getShowGridLines(model.getSelectedSheet())}
          onToggleShowGridLines={(show) => {
            const sheet = model.getSelectedSheet();
            model.setShowGridLines(sheet, show);
            setRedrawId((id) => id + 1);
          }}
          formatOptions={fmtSettings}
          onOpenConditionalFormatting={() =>
            openDrawer("conditionalFormatting")
          }
          isConditionalFormattingOpen={
            isDrawerOpen && drawerType === "conditionalFormatting"
          }
          onOpenNamedStyles={() => openDrawer("namedStyles")}
          isNamedStylesOpen={isDrawerOpen && drawerType === "namedStyles"}
          onOpenLinkDialog={actions.openLinkDialog}
          themes={themes}
          currentTheme={currentTheme}
          onThemePicked={handleThemePicked}
          onOpenThemes={actions.openThemes}
        />
      )}
      <div
        className="ic-workbook-worksheet-area-left"
        style={{
          width: isDrawerOpen ? `calc(100% - ${drawerWidth}px)` : "100%",
        }}
      >
        <FormulaBar
          cellAddress={cellAddress()}
          formulaValue={formulaValue}
          onChange={() => {
            setRedrawId((id) => id + 1);
            focusWorkbook();
          }}
          onTextUpdated={() => {
            setRedrawId((id) => id + 1);
          }}
          model={model}
          workbookState={workbookState}
          openDrawer={actions.openNamedRanges}
          canEdit={canEdit && canEditFormula}
        />
        <Worksheet
          model={model}
          workbookState={workbookState}
          refresh={(): void => {
            setRedrawId((id) => id + 1);
          }}
          ref={worksheetRef}
          canEdit={canEdit && canEditFormula}
          actions={actions}
          onEditLink={
            canEdit
              ? (row: number, column: number): void => {
                  openLinkDialog(model.getSelectedSheet(), row, column);
                }
              : undefined
          }
          onDeleteLink={
            canEdit
              ? (row: number, column: number): void => {
                  model.deleteCellLink(model.getSelectedSheet(), row, column);
                  setRedrawId((id) => id + 1);
                }
              : undefined
          }
        />

        <SheetTabBar
          canEdit={canEdit}
          sheets={info}
          selectedIndex={model.getSelectedSheet()}
          workbookState={workbookState}
          onSheetSelected={(sheet: number): void => {
            if (info[sheet].state !== "visible") {
              model.unhideSheet(sheet);
            }
            model.setSelectedSheet(sheet);
            setRedrawId((value) => value + 1);
          }}
          onAddBlankSheet={(): void => {
            model.newSheet();
            setRedrawId((value) => value + 1);
          }}
          onSheetColorChanged={(color: Color): void => {
            try {
              model.setSheetColor(model.getSelectedSheet(), color);
              setRedrawId((value) => value + 1);
            } catch (e) {
              // TODO: Show a proper modal dialog
              alert(`${e}`);
            }
          }}
          onSheetRenamed={(name: string): void => {
            try {
              model.renameSheet(model.getSelectedSheet(), name);
              setRedrawId((value) => value + 1);
            } catch (e) {
              // TODO: Show a proper modal dialog
              alert(`${e}`);
            }
          }}
          onSheetDeleted={(): void => {
            const selectedSheet = model.getSelectedSheet();
            model.deleteSheet(selectedSheet);
            setRedrawId((value) => value + 1);
          }}
          onSheetDuplicated={(): void => {
            try {
              const selectedSheet = model.getSelectedSheet();
              model.duplicateSheet(selectedSheet);
              setRedrawId((value) => value + 1);
            } catch (e) {
              // TODO: Show a proper modal dialog
              alert(`${e}`);
            }
          }}
          onHideSheet={(): void => {
            const selectedSheet = model.getSelectedSheet();
            model.hideSheet(selectedSheet);
            setRedrawId((value) => value + 1);
          }}
          onMoveSheet={(fromIndex: number, toIndex: number): void => {
            model.moveSheet(fromIndex, toIndex);
            setRedrawId((value) => value + 1);
          }}
          onOpenRegionalSettings={() => {
            openDrawer("regionalSettings");
          }}
          model={model}
        />
      </div>
      <RightDrawer
        isOpen={isDrawerOpen}
        onClose={() => setDrawerOpen(false)}
        width={drawerWidth}
        onWidthChange={setDrawerWidth}
        model={model}
        onUpdate={() => {
          setRedrawId((id) => id + 1);
        }}
        getSelectedArea={() => {
          const worksheetNames = model
            .getWorksheetsProperties()
            .map((s) => s.name);
          const selectedView = model.getSelectedView();
          return getFullRangeToString(selectedView, worksheetNames);
        }}
        drawerType={drawerType}
        themes={themes}
        currentTheme={model.getTheme()}
        onThemePicked={handleThemePicked}
        customStyles={(() => {
          const builtinNames = new Set(
            model.getBuiltinNamedStyles().map((s) => s.name.toLowerCase()),
          );
          return model
            .getNamedStyleList()
            .filter((name) => !builtinNames.has(name.toLowerCase()))
            .map((name) => ({ name, style: model.getNamedStyle(name) }));
        })()}
        builtinStyles={model.getBuiltinNamedStyles()}
        formatOptions={fmtSettings}
        onApplyNamedStyle={(name) => {
          model.onApplyNamedStyle(name);
          setRedrawId((id) => id + 1);
        }}
        onAddNamedStyle={(payload): SaveError => {
          try {
            model.createNamedStyle(
              payload.name,
              payload.style,
              payload.includes,
            );
            setRedrawId((id) => id + 1);
            return { nameError: "" };
          } catch (e) {
            return { nameError: e instanceof Error ? e.message : String(e) };
          }
        }}
        onUpdateNamedStyle={(originalName, payload): SaveError => {
          try {
            model.updateNamedStyle(
              originalName,
              payload.name,
              payload.style,
              payload.includes,
            );
            setRedrawId((id) => id + 1);
            return { nameError: "" };
          } catch (e) {
            return { nameError: e instanceof Error ? e.message : String(e) };
          }
        }}
        onDeleteNamedStyle={(name) => {
          model.deleteNamedStyle(name);
          setRedrawId((id) => id + 1);
        }}
        initialLocale={model.getLocale()}
        initialTimezone={model.getTimezone()}
        initialLanguage={model.getLanguage()}
        onSettingsSave={(locale, timezone, language) => {
          model.setLocale(locale);
          model.setTimezone(timezone);
          model.setLanguage(language);
          setRedrawId((id) => id + 1);
        }}
      />
      <Alert
        open={alertDialog !== null}
        onClose={() => setAlertDialog(null)}
        title={alertDialog?.title ?? ""}
        message={alertDialog?.message ?? ""}
      />
      <FormatPicker
        open={isFormatPickerOpen}
        onClose={() => setFormatPickerOpen(false)}
        numFmt={style.num_fmt}
        onChange={onNumberFormatPicked}
      />
      {linkDialogCell && (
        <LinkDialog
          open
          onClose={() => setLinkDialogCell(null)}
          sheetNames={worksheets.map((sheet) => sheet.name)}
          selectedSheetName={
            worksheets[linkDialogCell.sheet]?.name ?? worksheets[0]?.name ?? ""
          }
          initialLink={
            model.getCellLink(
              linkDialogCell.sheet,
              linkDialogCell.row,
              linkDialogCell.column,
            ) ?? null
          }
          initialLabel={model.getFormattedCellValue(
            linkDialogCell.sheet,
            linkDialogCell.row,
            linkDialogCell.column,
          )}
          onSave={onSaveLink}
          onDelete={onDeleteLink}
        />
      )}
    </div>
  );
};

export default forwardRef(Workbook);
