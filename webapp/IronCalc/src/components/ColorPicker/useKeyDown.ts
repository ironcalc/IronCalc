import { type KeyboardEvent, useCallback } from "react";

interface Options {
  onEscape: () => void;
  getFocusableElements: () => HTMLElement[];
}

function getNavPosition(element: HTMLElement) {
  const row = Number(element.dataset.navRow);
  const col = Number(element.dataset.navCol);

  if (Number.isNaN(row) || Number.isNaN(col)) {
    return null;
  }

  return { row, col };
}

function getElementsInRow(
  focusable: HTMLElement[],
  row: number,
): HTMLElement[] {
  return focusable.filter((element) => getNavPosition(element)?.row === row);
}

function getLastColInRow(focusable: HTMLElement[], row: number): number | null {
  const elements = getElementsInRow(focusable, row);
  if (elements.length === 0) {
    return null;
  }

  return Math.max(
    ...elements
      .map((element) => getNavPosition(element)?.col)
      .filter((col): col is number => col !== undefined),
  );
}

// Navigation for the color picker:
// * Tab/Shift+Tab: cycle through focusable elements in DOM order
// * Escape: close the picker
// * Arrow keys: navigate between elements with data-nav-row and data-nav-col
export const useKeyDown = (
  options: Options,
): { onKeyDown: (event: KeyboardEvent) => void } => {
  const { onEscape, getFocusableElements } = options;

  const onKeyDown = useCallback(
    (event: KeyboardEvent) => {
      const { key, shiftKey } = event;
      const focusable = getFocusableElements();

      if (focusable.length === 0) {
        return;
      }

      const active = document.activeElement as HTMLElement | null;
      const currentIndex = active ? focusable.indexOf(active) : -1;

      if (key === "Escape") {
        event.preventDefault();
        onEscape();
        return;
      }

      if (key === "Tab") {
        event.preventDefault();

        if (currentIndex === -1) {
          focusable[0]?.focus();
          return;
        }

        const nextIndex = shiftKey
          ? (currentIndex - 1 + focusable.length) % focusable.length
          : (currentIndex + 1) % focusable.length;

        focusable[nextIndex]?.focus();
        return;
      }

      if (
        key !== "ArrowLeft" &&
        key !== "ArrowRight" &&
        key !== "ArrowUp" &&
        key !== "ArrowDown"
      ) {
        return;
      }

      if (!active) {
        return;
      }

      const currentPosition = getNavPosition(active);
      if (!currentPosition) {
        return;
      }

      let targetRow = currentPosition.row;
      let targetCol = currentPosition.col;

      if (key === "ArrowLeft") {
        targetCol -= 1;
      } else if (key === "ArrowRight") {
        targetCol += 1;
      } else if (key === "ArrowUp") {
        targetRow -= 1;
      } else if (key === "ArrowDown") {
        targetRow += 1;
      }

      let target = focusable.find((element) => {
        const position = getNavPosition(element);
        return position?.row === targetRow && position?.col === targetCol;
      });

      if (!target && key === "ArrowLeft") {
        const previousRow = currentPosition.row - 1;
        const lastCol = getLastColInRow(focusable, previousRow);

        if (lastCol !== null) {
          target = focusable.find((element) => {
            const position = getNavPosition(element);
            return position?.row === previousRow && position?.col === lastCol;
          });
        }
      }

      if (!target && key === "ArrowRight") {
        const nextRow = currentPosition.row + 1;

        target = focusable.find((element) => {
          const position = getNavPosition(element);
          return position?.row === nextRow && position?.col === 0;
        });
      }

      if (!target && (key === "ArrowUp" || key === "ArrowDown")) {
        target = focusable.find((element) => {
          const position = getNavPosition(element);
          return (
            position?.row === targetRow && position?.col === currentPosition.col
          );
        });
      }

      if (target) {
        event.preventDefault();
        target.focus();
      }
    },
    [getFocusableElements, onEscape],
  );

  return { onKeyDown };
};

export default useKeyDown;
