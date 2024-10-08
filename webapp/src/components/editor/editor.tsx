// This is the cell editor for IronCalc
// It is also the most difficult part of the UX. It is based on an idea of Mateusz Kopec.
// There is a hidden texarea and we only show the caret. What we see is a div with the same text content
// but in HTML so we can have different colors.
// Some keystrokes have different behaviour than a raw HTML text area.
// For those cases we capture the keydown event and stop its propagation.
// As the editor changes content we need to propagate those changes so the spreadsheet can
// mark with colors the active ranges or update the formula in the formula bar
//
// Events outside the editor might influence the editor
// 1. Clicking on a different cell:
//    * might either terminate the editing
//    * or add the external cell to the formula
// 2. Clicking on a sheet tab would open the new sheet or terminate editing
// 3. Clicking somewhere else will finish editing
//
// Keyboard navigation is also fairly complex. For instance RightArrow might:
// 1. End editing and navigate to the cell on the right
// 2. Move the cursor to the right
// 3. Insert in the formula the cell name on the right

import type { Model } from "@ironcalc/wasm";
import {
  type CSSProperties,
  type KeyboardEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import type { WorkbookState } from "../workbookState";
import getFormulaHTML from "./util";

const commonCSS: CSSProperties = {
  fontWeight: "inherit",
  fontFamily: "inherit",
  fontSize: "inherit",
  position: "absolute",
  left: 0,
  top: 0,
  whiteSpace: "pre",
  width: "100%",
  padding: 0,
  lineHeight: "22px",
};

const caretColor = "#FF8899";

interface EditorOptions {
  minimalWidth: number | string;
  minimalHeight: number | string;
  display: boolean;
  expand: boolean;
  originalText: string;
  onEditEnd: () => void;
  onTextUpdated: () => void;
  model: Model;
  workbookState: WorkbookState;
  type: "cell" | "formula-bar";
}

const Editor = (options: EditorOptions) => {
  const {
    display,
    expand,
    minimalHeight,
    minimalWidth,
    model,
    onEditEnd,
    onTextUpdated,
    originalText,
    workbookState,
    type,
  } = options;

  const [width, setWidth] = useState(minimalWidth);
  const [height, setHeight] = useState(minimalHeight);
  const [text, setText] = useState(originalText);
  const [styledFormula, setStyledFormula] = useState(
    getFormulaHTML(model, text, "").html,
  );

  const formulaRef = useRef<HTMLDivElement>(null);
  const maskRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    setText(originalText);
    setStyledFormula(getFormulaHTML(model, originalText, "").html);
    if (textareaRef.current) {
      textareaRef.current.value = originalText;
    }
  }, [originalText, model]);

  const onKeyDown = useCallback(
    (event: KeyboardEvent) => {
      const { key, shiftKey, altKey } = event;
      const textarea = textareaRef.current;
      if (!textarea) {
        return;
      }
      switch (key) {
        case "Enter": {
          if (altKey) {
            // new line
            const start = textarea.selectionStart;
            const end = textarea.selectionEnd;
            const newText = `${text.slice(0, start)}\n${text.slice(end)}`;
            setText(newText);
            setTimeout(() => {
              textarea.setSelectionRange(start + 1, start + 1);
            }, 0);
            event.stopPropagation();
            event.preventDefault();
            return;
          }
          // end edit and select cell bellow
          setTimeout(() => {
            const cell = workbookState.getEditingCell();
            if (cell) {
              model.setUserInput(
                cell.sheet,
                cell.row,
                cell.column,
                cell.text + (cell.referencedRange?.str || ""),
              );
              const sign = shiftKey ? -1 : 1;
              model.setSelectedSheet(cell.sheet);
              model.setSelectedCell(cell.row + sign, cell.column);
              workbookState.clearEditingCell();
            }
            onEditEnd();
          }, 0);
          // event bubbles up
          return;
        }
        case "Tab": {
          // end edit and select cell to the right
          const cell = workbookState.getEditingCell();
          if (cell) {
            workbookState.clearEditingCell();
            model.setUserInput(
              cell.sheet,
              cell.row,
              cell.column,
              cell.text + (cell.referencedRange?.str || ""),
            );
            const sign = shiftKey ? -1 : 1;
            model.setSelectedSheet(cell.sheet);
            model.setSelectedCell(cell.row, cell.column + sign);
            if (textareaRef.current) {
              textareaRef.current.value = "";
              setStyledFormula(getFormulaHTML(model, "", "").html);
            }
            event.stopPropagation();
            event.preventDefault();
          }
          onEditEnd();
          return;
        }
        case "Escape": {
          // quit editing without modifying the cell
          const cell = workbookState.getEditingCell();
          if (cell) {
            model.setSelectedSheet(cell.sheet);
          }
          workbookState.clearEditingCell();
          onEditEnd();
          return;
        }
        // TODO: Arrow keys navigate in Excel
        case "ArrowRight": {
          return;
        }
        default: {
          // We run this in a timeout because the value is not yet in the textarea
          // since we are capturing the keydown event
          setTimeout(() => {
            const cell = workbookState.getEditingCell();
            if (cell) {
              // accept whatever is in the referenced range
              const value = textarea.value;
              const styledFormula = getFormulaHTML(model, value, "");

              cell.text = value;
              cell.referencedRange = null;
              cell.cursorStart = textarea.selectionStart;
              cell.cursorEnd = textarea.selectionEnd;
              workbookState.setEditingCell(cell);

              workbookState.setActiveRanges(styledFormula.activeRanges);
              setStyledFormula(styledFormula.html);

              onTextUpdated();
            }
          }, 0);
        }
      }
    },
    [model, text, onEditEnd, onTextUpdated, workbookState],
  );

  useEffect(() => {
    if (display) {
      textareaRef.current?.focus();
    }
  }, [display]);

  const onChange = useCallback(() => {
    if (textareaRef.current) {
      textareaRef.current.value = "";
      setStyledFormula(getFormulaHTML(model, "", "").html);
    }

    // This happens if the blur hasn't been taken care before by
    // onclick or onpointerdown events
    // If we are editing a cell finish that
    const cell = workbookState.getEditingCell();
    if (cell) {
      model.setUserInput(
        cell.sheet,
        cell.row,
        cell.column,
        workbookState.getEditingText(),
      );
      workbookState.clearEditingCell();
    }
    onEditEnd();
  }, [model, workbookState, onEditEnd]);

  const isCellEditing = workbookState.getEditingCell() !== null;

  const showEditor =
    (isCellEditing && display) || type === "formula-bar" ? "block" : "none";

  return (
    <div
      style={{
        position: "relative",
        width,
        height,
        overflow: "hidden",
        display: showEditor,
        background: "#FFF",
      }}
    >
      <div
        ref={maskRef}
        style={{
          ...commonCSS,
          textAlign: "left",
          pointerEvents: "none",
          height,
        }}
      >
        <div ref={formulaRef}>{styledFormula}</div>
      </div>
      <textarea
        ref={textareaRef}
        rows={1}
        style={{
          ...commonCSS,
          color: "transparent",
          backgroundColor: "transparent",
          caretColor,
          outline: "none",
          resize: "none",
          border: "none",
          height,
          overflow: "hidden",
        }}
        defaultValue={text}
        spellCheck="false"
        onKeyDown={onKeyDown}
        onBlur={onChange}
        onClick={(event) => {
          // Prevents this from bubbling up and focusing on the spreadsheet
          if (isCellEditing && type === "cell") {
            event.stopPropagation();
          }
        }}
      />
    </div>
  );
};
export default Editor;
