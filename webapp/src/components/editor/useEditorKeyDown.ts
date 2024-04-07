import { useCallback, KeyboardEvent } from "react";
import { WorkbookState } from "../workbookState";
import { Model } from "@ironcalc/wasm";

interface Options {
  // onMoveCaretToStart: () => void;
  // onMoveCaretToEnd: () => void;
  // onEditEnd: (delta: { deltaRow: number; deltaColumn: number }) => void;
  // onEditEscape: () => void;
  // onReferenceCycle: () => void;
  // text: string;
  // setText: (text: string) => void;
  model: Model;
  state: WorkbookState;
  refresh: () => void;
}

const useEditorKeydown = (
  options: Options
): {
  onKeyDown: (event: KeyboardEvent) => void;
} => {
  const { state, model } = options;
  const onKeyDown = useCallback((event: KeyboardEvent) => {
    const { key, shiftKey } = event;
    // const { mode, text } = state.getEditor() ?? { mode: "init", text: "" };
    switch (key) {
      // case "Enter":
      //   // options.onEditEnd({ deltaRow: 1, deltaColumn: 0 });
      //   const { row, column } = state.getSelectedCell();
      //   const sheet = state.getSelectedSheet();
      //   model.setUserInput(sheet, row, column, text);
      //   state.selectCell({ row: row + 1, column });
      //   event.preventDefault();
      //   event.stopPropagation();
      //   options.refresh();
      //   break;
      // case 'ArrowUp': {
      //   if (mode === 'init') {
      //     options.onEditEnd({ deltaRow: -1, deltaColumn: 0 });
      //   } else {
      //     options.onMoveCaretToStart();
      //   }
      //   event.preventDefault();
      //   event.stopPropagation();
      //   break;
      // }
      // case 'ArrowDown': {
      //   if (mode === 'init') {
      //     options.onEditEnd({ deltaRow: 1, deltaColumn: 0 });
      //   } else {
      //     options.onMoveCaretToEnd();
      //   }
      //   event.preventDefault();
      //   event.stopPropagation();
      //   break;
      // }
      // case 'Tab': {
      //   if (event.shiftKey) {
      //     options.onEditEnd({ deltaRow: 0, deltaColumn: -1 });
      //   } else {
      //     options.onEditEnd({ deltaRow: 0, deltaColumn: 1 });
      //   }
      //   event.preventDefault();
      //   event.stopPropagation();

      //   break;
      // }
      // case 'Escape': {
      //   options.onEditEscape();
      //   event.preventDefault();
      //   event.stopPropagation();

      //   break;
      // }
      // case 'ArrowLeft': {
      //   if (mode === 'init') {
      //     options.onEditEnd({ deltaRow: 0, deltaColumn: -1 });
      //     event.preventDefault();
      //     event.stopPropagation();
      //   }

      //   break;
      // }
      // case 'ArrowRight': {
      //   if (mode === 'init') {
      //     options.onEditEnd({ deltaRow: 0, deltaColumn: 1 });
      //     event.preventDefault();
      //     event.stopPropagation();
      //   }

      //   break;
      // }
      // case 'F4': {
      //   options.onReferenceCycle();
      //   event.preventDefault();
      //   event.stopPropagation();

      //   break;
      // }
      default:
        break;
    }
  }, [model, state]);
  return { onKeyDown };
};

export default useEditorKeydown;
