import { type KeyboardEvent, type RefObject, useCallback } from "react";

interface Options {
  focusableElements: RefObject<HTMLElement | null>[];
  onClose: () => void;
  onConfirm?: () => void;
}

export function useModalKeyDown({
  focusableElements,
  onClose,
  onConfirm,
}: Options) {
  const onKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.stopPropagation();
        onClose();
        return;
      }

      if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
        event.preventDefault();
        onConfirm?.();
        return;
      }

      if (event.key === "Tab") {
        const firstEl = focusableElements[0]?.current;
        const lastEl = focusableElements[focusableElements.length - 1]?.current;
        if (!firstEl || !lastEl) {
          return;
        }

        if (event.shiftKey) {
          if (document.activeElement === firstEl) {
            event.preventDefault();
            lastEl.focus();
          }
        } else if (document.activeElement === lastEl) {
          event.preventDefault();
          firstEl.focus();
        }
      }
    },
    [focusableElements, onClose, onConfirm],
  );

  return { onKeyDown };
}
