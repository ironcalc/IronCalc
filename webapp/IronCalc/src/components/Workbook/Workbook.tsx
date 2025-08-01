import type {
  BorderOptions,
  ClipboardCell,
  Model,
  WorksheetProperties,
} from "@ironcalc/wasm";
import { styled } from "@mui/material/styles";
import { useCallback, useEffect, useRef, useState } from "react";
import FormulaBar from "../FormulaBar/FormulaBar";
import SheetTabBar from "../SheetTabBar";
import Toolbar from "../Toolbar/Toolbar";
import Worksheet from "../Worksheet/Worksheet";
import {
  COLUMN_WIDTH_SCALE,
  LAST_COLUMN,
  LAST_ROW,
  ROW_HEIGH_SCALE,
} from "../WorksheetCanvas/constants";
import type WorksheetCanvas from "../WorksheetCanvas/worksheetCanvas";
import { devicePixelRatio } from "../WorksheetCanvas/worksheetCanvas";
import {
  CLIPBOARD_ID_SESSION_STORAGE_KEY,
  getNewClipboardId,
} from "../clipboard";
import {
  type NavigationKey,
  getCellAddress,
  getFullRangeToString,
} from "../util";
import type { WorkbookState } from "../workbookState";
import useKeyboardNavigation from "./useKeyboardNavigation";

const Workbook = (props: { model: Model; workbookState: WorkbookState }) => {
  const { model, workbookState } = props;
  const rootRef = useRef<HTMLDivElement | null>(null);
  const worksheetRef = useRef<{
    getCanvas: () => WorksheetCanvas | null;
  }>(null);

  // Calling `setRedrawId((id) => id + 1);` forces a redraw
  // This is needed because `model` or `workbookState` can change without React being aware of it
  const setRedrawId = useState(0)[1];

  const worksheets = model.getWorksheetsProperties();
  const info = worksheets.map(
    ({ name, color, sheet_id, state }: WorksheetProperties) => {
      return { name, color: color ? color : "#FFF", sheetId: sheet_id, state };
    },
  );
  const focusWorkbook = useCallback(() => {
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

  const onTextColorPicked = (hex: string) => {
    updateRangeStyle("font.color", hex);
  };

  const onFillColorPicked = (hex: string) => {
    updateRangeStyle("fill.fg_color", hex);
  };

  const onNumberFormatPicked = (numberFmt: string) => {
    updateRangeStyle("num_fmt", numberFmt);
  };

  const onIncreaseFontSize = (delta: number) => {
    updateRangeStyle("font.size_delta", `${delta}`);
  };

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
        styleRow.push(model.getCellStyle(sheet, row, column));
      }
      styles.push(styleRow);
    }
    workbookState.setCopyStyles(styles);
    // FIXME: This is so that the cursor indicates there are styles to be pasted
    const el = rootRef.current?.getElementsByClassName("sheet-container")[0];
    if (el) {
      // Taken from lucide icons: <PaintRoller /> and rotated.
      const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-paint-roller" style="transform:rotate(-8deg)"><rect width="16" height="6" x="2" y="2" rx="2"></rect><path d="M10 16v-2a2 2 0 0 1 2-2h8a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2"></path><rect width="4" height="6" x="8" y="16" rx="1"></rect></svg>`;
      (el as HTMLElement).style.cursor =
        `url('data:image/svg+xml;utf8,${encodeURIComponent(svg)}'), auto`;
    }
  };

  // FIXME: I *think* we should have only one on onKeyPressed function that goes to
  // the Rust end
  const { onKeyDown } = useKeyboardNavigation({
    onCellsDeleted: (): void => {
      const {
        sheet,
        range: [rowStart, columnStart, rowEnd, columnEnd],
      } = model.getSelectedView();
      const row = Math.min(rowStart, rowEnd);
      const column = Math.min(columnStart, columnEnd);

      const width = Math.abs(columnEnd - columnStart);
      const height = Math.abs(rowEnd - rowStart);
      model.rangeClearContents(
        sheet,
        row,
        column,
        row + height,
        column + width,
      );
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
      const editorWidth =
        model.getColumnWidth(sheet, column) * COLUMN_WIDTH_SCALE;
      const editorHeight = model.getRowHeight(sheet, row) * ROW_HEIGH_SCALE;
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
      const editorWidth =
        model.getColumnWidth(sheet, column) * COLUMN_WIDTH_SCALE;
      const editorHeight = model.getRowHeight(sheet, row) * ROW_HEIGH_SCALE;
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
    onBold: () => {
      const { sheet, row, column } = model.getSelectedView();
      const value = model.getCellStyle(sheet, row, column).font.b;
      onToggleBold(!value);
    },
    onItalic: () => {
      const { sheet, row, column } = model.getSelectedView();
      const value = model.getCellStyle(sheet, row, column).font.i;
      onToggleItalic(!value);
    },
    onUnderline: () => {
      const { sheet, row, column } = model.getSelectedView();
      const value = model.getCellStyle(sheet, row, column).font.u;
      onToggleUnderline(!value);
    },
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
      const view = model.getSelectedView();
      const cell = model.getSelectedCell();
      model.setSelectedCell(cell[1], 1);
      model.setTopLeftVisibleCell(view.top_row, 1);
      setRedrawId((id) => id + 1);
    },
    onKeyEnd: (): void => {
      const view = model.getSelectedView();
      const cell = model.getSelectedCell();
      model.setSelectedCell(cell[1], LAST_COLUMN);
      model.setTopLeftVisibleCell(view.top_row, LAST_COLUMN - 5);
      setRedrawId((id) => id + 1);
    },
    onUndo: (): void => {
      model.undo();
      setRedrawId((id) => id + 1);
    },
    onRedo: (): void => {
      model.redo();
      setRedrawId((id) => id + 1);
    },
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

  useEffect(() => {
    if (!rootRef.current) {
      return;
    }
    if (!workbookState.getEditingCell()) {
      focusWorkbook();
    }
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

  const formulaValue = () => {
    const cell = workbookState.getEditingCell();
    if (cell) {
      return workbookState.getEditingText();
    }
    const { sheet, row, column } = model.getSelectedView();
    return model.getCellContent(sheet, row, column);
  };

  const getCellStyle = useCallback(() => {
    const { sheet, row, column } = model.getSelectedView();
    return model.getCellStyle(sheet, row, column);
  }, [model]);

  const style = getCellStyle();

  return (
    <Container
      ref={rootRef}
      onKeyDown={onKeyDown}
      tabIndex={0}
      onClick={(event: React.MouseEvent) => {
        if (!workbookState.getEditingCell()) {
          focusWorkbook();
        } else {
          event.stopPropagation();
        }
      }}
      onPaste={(event: React.ClipboardEvent) => {
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
          model.pasteFromClipboard(
            source.sheet,
            source.area,
            data,
            source.type === "cut",
          );
          setRedrawId((id) => id + 1);
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
          model.pasteCsvText(range, value);
          setRedrawId((id) => id + 1);
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
        event.clipboardData.setData("text/plain", data.csv.trim());
        event.clipboardData.setData("application/json", clipboardJsonStr);
        event.preventDefault();
        event.stopPropagation();
      }}
      onCut={(event: React.ClipboardEvent) => {
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
        onClearFormatting={() => {
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
        }}
        onIncreaseFontSize={(delta: number) => {
          onIncreaseFontSize(delta);
        }}
        onDownloadPNG={() => {
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
          let [x, y] = worksheetCanvas.getCoordinatesByCell(
            rowStart,
            columnStart,
          );
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
        }}
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
        fillColor={style.fill.fg_color || "#FFFFFF"}
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
        canEdit={true}
        numFmt={style.num_fmt}
        showGridLines={model.getShowGridLines(model.getSelectedSheet())}
        onToggleShowGridLines={(show) => {
          const sheet = model.getSelectedSheet();
          model.setShowGridLines(sheet, show);
          setRedrawId((id) => id + 1);
        }}
        nameManagerProperties={{
          newDefinedName: (
            name: string,
            scope: number | undefined,
            formula: string,
          ) => {
            model.newDefinedName(name, scope, formula);
            setRedrawId((id) => id + 1);
          },
          updateDefinedName: (
            name: string,
            scope: number | undefined,
            newName: string,
            newScope: number | undefined,
            newFormula: string,
          ) => {
            model.updateDefinedName(name, scope, newName, newScope, newFormula);
            setRedrawId((id) => id + 1);
          },
          deleteDefinedName: (name: string, scope: number | undefined) => {
            model.deleteDefinedName(name, scope);
            setRedrawId((id) => id + 1);
          },
          selectedArea: () => {
            const worksheetNames = worksheets.map((s) => s.name);
            const selectedView = model.getSelectedView();

            return getFullRangeToString(selectedView, worksheetNames);
          },
          worksheets,
          definedNameList: model.getDefinedNameList(),
        }}
      />
      <FormulaBar
        cellAddress={cellAddress()}
        formulaValue={formulaValue()}
        onChange={() => {
          setRedrawId((id) => id + 1);
          focusWorkbook();
        }}
        onTextUpdated={() => {
          setRedrawId((id) => id + 1);
        }}
        model={model}
        workbookState={workbookState}
      />
      <Worksheet
        model={model}
        workbookState={workbookState}
        refresh={(): void => {
          setRedrawId((id) => id + 1);
        }}
        ref={worksheetRef}
      />

      <SheetTabBar
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
        onSheetColorChanged={(hex: string): void => {
          try {
            model.setSheetColor(model.getSelectedSheet(), hex);
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
        onHideSheet={(): void => {
          const selectedSheet = model.getSelectedSheet();
          model.hideSheet(selectedSheet);
          setRedrawId((value) => value + 1);
        }}
      />
    </Container>
  );
};

const Container = styled("div")`
  display: flex;
  flex-direction: column;
  height: 100%;
  position: relative;
  font-family: ${({ theme }) => theme.typography.fontFamily};

  &:focus {
    outline: none;
  }
`;

export default Workbook;
