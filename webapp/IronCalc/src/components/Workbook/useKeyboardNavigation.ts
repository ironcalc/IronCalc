import { type KeyboardEvent, type RefObject, useCallback } from "react";
import { type NavigationKey, isEditingKey, isNavigationKey } from "../util";

export enum Border {
  Top = "top",
  Bottom = "bottom",
  Right = "right",
  Left = "left",
}

interface Options {
  onCellsDeleted: () => void;
  onExpandAreaSelectedKeyboard: (
    key: "ArrowRight" | "ArrowLeft" | "ArrowUp" | "ArrowDown",
  ) => void;
  onEditKeyPressStart: (initText: string) => void;
  onCellEditStart: () => void;
  onBold: () => void;
  onItalic: () => void;
  onUnderline: () => void;
  onNavigationToEdge: (direction: NavigationKey) => void;
  onPageDown: () => void;
  onPageUp: () => void;
  onArrowDown: () => void;
  onArrowUp: () => void;
  onArrowLeft: () => void;
  onArrowRight: () => void;
  onKeyHome: () => void;
  onKeyEnd: () => void;
  onUndo: () => void;
  onRedo: () => void;
  onNextSheet: () => void;
  onPreviousSheet: () => void;
  onEscape: () => void;
  onSelectColumn: () => void;
  onSelectRow: () => void;
  root: RefObject<HTMLDivElement | null>;
}

// # IronCalc Keyboard accessibility:
// * ArrowKeys: navigation
// * Enter: ArrowDown (Excel behaviour not g-sheets)
// * Tab: arrow right
// * Shift+Tab: arrow left
// * Home/End: First/last column
// * Shift+Arrows: selection
// * Ctrl+Arrows: navigating to edge
// * Ctrl+Home/End: navigation to end
// * PagDown/Up scroll Down/Up
// * Alt+ArrowDown/Up: next/previous sheet
//   (NB: Excel uses Ctrl+PageUp/Down for this but that highjacks a browser behaviour,
//    go to next/previous tab)
// * Ctrl+u/i/b: style
// * Ctrl+z/y: undo/redo
// * F2: start editing
// * Ctrl+Space: select column
// * Shift+Space: select row
//
// # Not implemented yet:
// * Ctrl+a: select all (continuous area around the selection, if it exists,
//   otherwise select whole sheet)
// * Ctrl+Shift+Arrows: select to edge
// * Ctrl+Shift+Home/End: select to end
// * Ctrl+Shift++: (after selecting) insert row/column (also Alt+I, R or C)
// * Ctrl+-: (after selecting) delete row/column

// References:
// In Google Sheets: Ctrl+/ shows the list of keyboard shortcuts
// https://support.google.com/docs/answer/181110
// https://support.microsoft.com/en-us/office/keyboard-shortcuts-in-excel-1798d9d5-842a-42b8-9c99-9b7213f0040f

const useKeyboardNavigation = (
  options: Options,
): { onKeyDown: (event: KeyboardEvent) => void } => {
  const onKeyDown = useCallback(
    (event: KeyboardEvent) => {
      const { key } = event;
      const lowerKey = key.toLowerCase();
      const { root } = options;
      // Silence the linter
      if (!root.current) {
        return;
      }
      if (event.target !== root.current) {
        return;
      }
      const isCtrl = event.metaKey || event.ctrlKey;
      const isShift = event.shiftKey;
      const isAlt = event.altKey;
      if (isCtrl && !isShift && !isAlt) {
        // Ctrl+...
        switch (lowerKey) {
          case "z": {
            options.onUndo();
            event.stopPropagation();
            event.preventDefault();
            break;
          }
          case "y": {
            options.onRedo();
            event.stopPropagation();
            event.preventDefault();
            break;
          }
          case "b": {
            options.onBold();
            event.stopPropagation();
            event.preventDefault();
            break;
          }
          case "i": {
            options.onItalic();
            event.stopPropagation();
            event.preventDefault();
            break;
          }
          case "u": {
            options.onUnderline();
            event.stopPropagation();
            event.preventDefault();
            break;
          }
          case "a": {
            // TODO: Area selection. CTRL+A should select "continuous" area around the selection,
            // if it does exist then whole sheet is selected.
            event.stopPropagation();
            event.preventDefault();
            break;
          }
          case " ": {
            options.onSelectColumn();
            event.stopPropagation();
            event.preventDefault();
            break;
          }
          // No default
        }
        if (isNavigationKey(key)) {
          // Ctrl+Arrows, Ctrl+Home/End
          options.onNavigationToEdge(key);
          event.stopPropagation();
          event.preventDefault();
        }
        return;
      }
      if (isCtrl && isShift && !isAlt) {
        // Ctrl+Shift+...
        switch (lowerKey) {
          case "z": {
            options.onRedo();
            event.stopPropagation();
            event.preventDefault();
            break;
          }
        }
        return;
      }
      if (isShift && !isAlt && !isCtrl) {
        // Shift+...
        switch (key) {
          case " ": {
            options.onSelectRow();
            event.stopPropagation();
            event.preventDefault();
            break;
          }
          case "ArrowRight":
          case "ArrowLeft":
          case "ArrowUp":
          case "ArrowDown": {
            options.onExpandAreaSelectedKeyboard(key);
            break;
          }
          case "Tab": {
            options.onArrowLeft();
            event.stopPropagation();
            event.preventDefault();
            break;
          }
        }
      }
      if (isAlt && !isCtrl && !isShift) {
        // Alt+...
        switch (key) {
          case "ArrowDown": {
            // select next sheet
            options.onNextSheet();
            event.stopPropagation();
            event.preventDefault();
            break;
          }
          case "ArrowUp": {
            // select previous sheet
            options.onPreviousSheet();
            event.stopPropagation();
            event.preventDefault();
            break;
          }
        }
        return;
      }
      if (isCtrl || isAlt) {
        // Other combinations with Ctrl or Alt are not handled
        return;
      }

      if (isEditingKey(key) || key === "Backspace") {
        const initText = key === "Backspace" ? "" : key;
        options.onEditKeyPressStart(initText);
        event.stopPropagation();
        event.preventDefault();
        return;
      }
      if (isShift) {
        // Other combinations with Shift are not handled
        return;
      }
      if (key === "F2") {
        options.onCellEditStart();
        event.stopPropagation();
        event.preventDefault();
        return;
      }
      // Worksheet Navigation
      switch (key) {
        case "ArrowRight":
        case "Tab": {
          options.onArrowRight();
          break;
        }
        case "ArrowLeft": {
          options.onArrowLeft();
          break;
        }
        case "ArrowDown":
        case "Enter": {
          options.onArrowDown();
          break;
        }
        case "ArrowUp": {
          options.onArrowUp();
          break;
        }
        case "End": {
          options.onKeyEnd();
          break;
        }
        case "Home": {
          options.onKeyHome();
          break;
        }
        case "Delete": {
          options.onCellsDeleted();
          break;
        }
        case "PageDown": {
          options.onPageDown();
          break;
        }
        case "PageUp": {
          options.onPageUp();
          break;
        }
        case "Escape": {
          options.onEscape();
        }
        // No default
      }
      event.stopPropagation();
      event.preventDefault();
    },
    [options],
  );
  return { onKeyDown };
};

export default useKeyboardNavigation;
