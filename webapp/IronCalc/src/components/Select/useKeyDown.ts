import { type KeyboardEvent, useCallback } from "react";

interface Options {
  open: boolean;
  onEscape: () => void;
  getFocusableElements: () => HTMLElement[];
}

// Navigation for the Select menu:
// * Tab/Shift+Tab: cycle through focusable options
// * Escape: close the menu
export const useKeyDown = (
  options: Options,
): { onKeyDown: (event: KeyboardEvent) => void } => {
  const { open, onEscape, getFocusableElements } = options;

  const onKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (!open) {
        return;
      }

      const { key, shiftKey } = event;

      if (key === "Escape") {
        event.preventDefault();
        onEscape();
        return;
      }

      if (key !== "Tab") {
        return;
      }

      const focusable = getFocusableElements();
      if (focusable.length === 0) {
        return;
      }

      event.preventDefault();

      const active = document.activeElement as HTMLElement | null;
      const currentIndex = active ? focusable.indexOf(active) : -1;

      if (currentIndex === -1) {
        focusable[0]?.focus();
        return;
      }

      const nextIndex = shiftKey
        ? (currentIndex - 1 + focusable.length) % focusable.length
        : (currentIndex + 1) % focusable.length;

      focusable[nextIndex]?.focus();
    },
    [getFocusableElements, onEscape, open],
  );

  return { onKeyDown };
};

export default useKeyDown;
