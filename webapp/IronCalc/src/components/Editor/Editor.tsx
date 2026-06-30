// This is the cell editor for IronCalc
// It is also the single most difficult part of the UX. It is based on an idea of the
// celebrated Polish developer Mateusz Kopec.
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

// You can either be editing a formula or content.
// When editing content (behaviour is common to Excel and Google Sheets):
// * If you start editing by typing you are in *accept* mode
// * If you start editing by F2 you are in *cruise* mode
// * If you start editing by double click you are in *cruise* mode
// In Google Sheets "Enter" starts editing and puts you in *cruise* mode. We do not do that
// Once you are in cruise mode it is not possible to switch to accept mode
// The only way to go from accept mode to cruise mode is clicking in the content somewhere

// When editing a formula.
// In Google Sheets you are either in insert mode or cruise mode.
// You can get back to accept mode if you delete the whole formula
// In Excel you can be either in insert or accept but if you click in the formula body
// you switch to cruise mode. Once in cruise mode you can go to insert mode by selecting a range.
// Then you are back in accept/insert modes

import type { Model } from "@ironcalc/wasm";
import {
  type CSSProperties,
  type KeyboardEvent as ReactKeyboardEvent,
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { FormulaHelper } from "../FormulaHelper/FormulaHelper";
import {
  applyListCompletion,
  getCompletion,
} from "../FormulaHelper/formulaCompletion";
import { Alert } from "../Modal";
import type { WorkbookState } from "../workbookState";
import useKeyDown from "./useKeyDown";
import getFormulaHTML from "./util";
import "./editor.css";
import { createAnchoredPortal } from "../createAnchoredPortal";

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

const caretColor = "rgb(242, 153, 74)";

interface EditorOptions {
  originalText: string;
  onEditEnd: () => void;
  onTextUpdated: () => void;
  model: Model;
  workbookState: WorkbookState;
  type: "cell" | "formula-bar";
  canEdit: boolean;
}

const Editor = (options: EditorOptions) => {
  const {
    canEdit,
    model,
    onEditEnd,
    onTextUpdated,
    originalText,
    workbookState,
    type,
  } = options;

  const { t } = useTranslation();
  const [text, setText] = useState(originalText);
  const [cursor, setCursor] = useState(originalText.length);
  const [formulaError, setFormulaError] = useState<string | null>(null);
  // Formula helper popup state: the highlighted row in list mode, whether the
  // user dismissed it with Escape, and the viewport position (from the textarea
  // rect) where it is rendered via a portal, to escape `overflow: hidden`.
  const [helperSelected, setHelperSelected] = useState(0);
  const [helperDismissed, setHelperDismissed] = useState(false);
  const [helperPosition, setHelperPosition] = useState<{
    left: number;
    top: number;
  } | null>(null);

  const formulaRef = useRef<HTMLDivElement>(null);
  const maskRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    setText(originalText);
    const textarea = textareaRef.current;
    // Only rewrite the textarea (and snap the caret to the end) when the value
    // changed from the OUTSIDE — a different cell selected, or a new edit
    // session. While the user types, `originalText` is derived from the same
    // text being edited and flows back in unchanged, so the DOM value already
    // matches; rewriting here would yank the caret to the end on every
    // keystroke (and break clicking in the middle).
    if (textarea && textarea.value !== originalText) {
      textarea.value = originalText;
      // If the value changed because a reference is being inserted at the cursor
      // (cruise mode: clicking/dragging cells in the grid), keep the caret right
      // after the inserted reference. Snapping to the end is only correct when
      // the reference sits at the end of the formula; in the middle it would
      // strand the caret past the rest of the text.
      const cell = workbookState.getEditingCell();
      const referencedStr = cell?.referencedRange?.str;
      if (cell && referencedStr) {
        const caret = cell.cursorStart + referencedStr.length;
        textarea.setSelectionRange(caret, caret);
        setCursor(caret);
      } else {
        setCursor(originalText.length);
      }
    }
  }, [originalText, workbookState]);

  const { onKeyDown } = useKeyDown({
    model,
    onEditEnd,
    onTextUpdated,
    workbookState,
    textareaRef,
  });

  useEffect(() => {
    const cell = workbookState.getEditingCell();
    if (!cell) {
      return;
    }
    const { editorWidth, editorHeight } = cell;
    if (formulaRef.current) {
      const scrollWidth = formulaRef.current.scrollWidth;
      if (scrollWidth > editorWidth - 5) {
        cell.editorWidth = scrollWidth + 10;
      }
      const scrollHeight = formulaRef.current.scrollHeight;
      if (scrollHeight > editorHeight) {
        cell.editorHeight = scrollHeight;
      }
    }
    if (type === cell.focus) {
      textareaRef.current?.focus({ preventScroll: true });
    }
  });

  const onChange = useCallback(() => {
    const textarea = textareaRef.current;
    const cell = workbookState.getEditingCell();
    if (!textarea || !cell) {
      return;
    }
    const value = textarea.value;
    cell.text = value;
    cell.referencedRange = null;
    cell.cursorStart = textarea.selectionStart;
    cell.cursorEnd = textarea.selectionEnd;
    const styledFormula = getFormulaHTML(model, value);
    if (value === "" && type === "cell") {
      // When we delete the content of a cell we jump to accept mode
      cell.mode = "accept";
    }
    workbookState.setEditingCell(cell);

    workbookState.setActiveRanges(styledFormula.activeRanges);
    setText(cell.text);
    setCursor(textarea.selectionStart);

    onTextUpdated();

    // Should we stop propagations?
    // event.stopPropagation();
    // event.preventDefault();
  }, [workbookState, model, onTextUpdated, type]);

  const onBlur = useCallback(() => {
    const cell = workbookState.getEditingCell();
    if (type !== cell?.focus) {
      // If the onBlur event is called because we switch from the cell editor to the formula editor
      // or vice versa, do nothing
      return;
    }
    if (textareaRef.current) {
      textareaRef.current.value = "";
    }

    // This happens if the blur hasn't been taken care before by
    // onclick or onpointerdown events
    // If we are editing a cell finish that
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
  }, [model, workbookState, onEditEnd, type]);

  const cell = workbookState.getEditingCell();

  const showEditor = cell !== null || type === "formula-bar" ? "block" : "none";
  const mtext = cell ? workbookState.getEditingText() : originalText;
  const styledFormula = getFormulaHTML(model, mtext, cursor).html;

  // The formula helper is shown while editing a formula in whichever editor
  // currently has focus — the cell editor or the formula bar (both render this
  // same component). Gating on `cell.focus === type` keeps a single popup. We
  // compute the completion here so the keyboard handler and the popup share one
  // source.
  const helperActive =
    cell !== null &&
    cell.focus === type &&
    text.startsWith("=") &&
    !helperDismissed;
  const completion = helperActive ? getCompletion(model, text, cursor) : null;
  const showHelper = completion !== null;

  // Reset the highlighted row whenever the formula or caret changes (typing,
  // clicking); arrow-key navigation is preventDefaulted so it does not land
  // here. Editing the text also clears a previous Escape dismissal.
  // biome-ignore lint/correctness/useExhaustiveDependencies: reset on edit/caret
  useEffect(() => {
    setHelperSelected(0);
  }, [text, cursor]);
  // biome-ignore lint/correctness/useExhaustiveDependencies: reset on text edit
  useEffect(() => {
    setHelperDismissed(false);
  }, [text]);

  // Keep the popup anchored to the bottom-left of the textarea. `text`/`cursor`
  // are listed so we re-measure as the editor grows, even though they are not
  // read directly here.
  // biome-ignore lint/correctness/useExhaustiveDependencies: re-measure on edit
  useLayoutEffect(() => {
    if (showHelper && textareaRef.current) {
      const rect = textareaRef.current.getBoundingClientRect();
      setHelperPosition({ left: rect.left, top: rect.bottom + 4 });
    } else {
      setHelperPosition(null);
    }
  }, [showHelper, text, cursor]);

  // Replace the partial function name with `NAME(` and place the caret inside.
  const acceptHelperFunction = () => {
    const textarea = textareaRef.current;
    if (!textarea || completion?.kind !== "list") {
      return;
    }
    const result = applyListCompletion(
      textarea.value,
      textarea.selectionStart,
      completion,
      helperSelected,
    );
    textarea.value = result.text;
    textarea.setSelectionRange(result.cursor, result.cursor);
    onChange();
  };

  // Let the helper consume navigation keys before the editor's own handler.
  // Returns true when the event was handled.
  const handleHelperKeyDown = (
    event: ReactKeyboardEvent<HTMLTextAreaElement>,
  ): boolean => {
    if (!completion) {
      return false;
    }
    if (event.key === "Escape") {
      setHelperDismissed(true);
      event.preventDefault();
      event.stopPropagation();
      return true;
    }
    if (completion.kind !== "list") {
      return false;
    }
    if (event.key === "ArrowDown") {
      setHelperSelected((value) =>
        Math.min(value + 1, completion.matches.length - 1),
      );
    } else if (event.key === "ArrowUp") {
      setHelperSelected((value) => Math.max(value - 1, 0));
    } else if (event.key === "Enter" || event.key === "Tab") {
      acceptHelperFunction();
    } else {
      return false;
    }
    event.preventDefault();
    event.stopPropagation();
    return true;
  };

  return (
    <>
      <Alert
        open={formulaError !== null}
        onClose={() => setFormulaError(null)}
        title={t("error_dialog.error_editing_formula")}
        message={formulaError ?? undefined}
      />
      <div
        style={{
          position: "relative",
          width: "100%",
          height: "100%",
          overflow: "hidden",
          display: showEditor,
          background: "#FFF",
          fontFamily: "var(--palette-sheet-default-cell-font-family)",
          fontSize: "12px",
        }}
      >
        <div
          ref={maskRef}
          style={{
            ...commonCSS,
            textAlign: "left",
            pointerEvents: "none",
            height: "100%",
          }}
        >
          <div
            style={{
              display: "inline-block",
            }}
            ref={formulaRef}
          >
            {styledFormula}
          </div>
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
            height: "100%",
            overflow: "hidden",
            alignContent: "baseline",
          }}
          defaultValue={text}
          spellCheck="false"
          onKeyDown={(event) => {
            if (handleHelperKeyDown(event)) {
              return;
            }
            try {
              onKeyDown(event);
            } catch (error) {
              // quit editing without modifying the cell
              const cell = workbookState.getEditingCell();
              if (cell) {
                model.setSelectedSheet(cell.sheet);
              }
              workbookState.clearEditingCell();
              onEditEnd();
              setFormulaError(String(error));
            }
          }}
          disabled={!canEdit}
          onChange={onChange}
          onBlur={onBlur}
          onPointerDown={(event) => {
            if (!canEdit) {
              return;
            }
            // We are either clicking in the same cell we are editing,
            // in which case we just change the mode to edit, or we click
            // in a different editor, in which case we switch the focus
            const cell = workbookState.getEditingCell();
            if (cell) {
              // We make sure the mode is edit
              cell.mode = "edit";
              cell.focus = type;
              workbookState.setEditingCell(cell);
              event.stopPropagation();
            }
          }}
          onScroll={() => {
            if (maskRef.current && textareaRef.current) {
              maskRef.current.style.left = `-${textareaRef.current.scrollLeft}px`;
              maskRef.current.style.top = `-${textareaRef.current.scrollTop}px`;
            }
          }}
          onSelect={(event) => {
            // Track caret moves (arrows, clicks) so the helper follows along.
            setCursor(event.currentTarget.selectionStart);
          }}
          onPaste={(event) => event.stopPropagation()}
          onCopy={(event) => event.stopPropagation()}
          onDoubleClick={(event) => event.stopPropagation()}
          onCut={(event) => event.stopPropagation()}
        />
      </div>
      {showHelper && helperPosition
        ? createAnchoredPortal(
            <div
              style={{
                position: "fixed",
                left: helperPosition.left,
                top: helperPosition.top,
                zIndex: 1000,
              }}
            >
              <FormulaHelper
                completion={completion}
                selected={helperSelected}
                onSelect={setHelperSelected}
              />
            </div>,
            textareaRef.current,
          )
        : null}
    </>
  );
};
export default Editor;
