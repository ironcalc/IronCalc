import { type KeyboardEvent, useCallback } from "react";

interface Options {
  onEscape: () => void;
  getFocusableElements: () => HTMLElement[];
}

// Simple navigation:
// * Tab/Shift+Tab: cycle through focusable elements
// * Escape: close the picker
export const useKeyDown = (
  options: Options,
): { onKeyDown: (event: KeyboardEvent) => void } => {
  const { onEscape, getFocusableElements } = options;

  const onKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onEscape();
        return;
      }

      if (event.key !== "Tab") {
        return;
      }

      const focusable = getFocusableElements();

      if (focusable.length === 0) {
        return;
      }

      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      const active = document.activeElement;

      if (!event.shiftKey && active === last) {
        event.preventDefault();
        first.focus();
      }

      if (event.shiftKey && active === first) {
        event.preventDefault();
        last.focus();
      }
    },
    [getFocusableElements, onEscape],
  );

  return { onKeyDown };
};

export default useKeyDown;
