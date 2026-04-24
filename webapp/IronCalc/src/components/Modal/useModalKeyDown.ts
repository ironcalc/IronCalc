import { type KeyboardEvent, type RefObject, useCallback } from "react";

interface Options {
  focusableElements: RefObject<HTMLElement | null>[];
  onClose: () => void;
  onConfirm?: () => void;
  enterConfirm?: boolean;
}

export function useModalKeyDown({
  focusableElements,
  onClose,
  onConfirm,
  enterConfirm,
}: Options) {
  const onKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.stopPropagation();
        onClose();
        return;
      }

      if (
        event.key === "Enter" &&
        (event.metaKey || event.ctrlKey || enterConfirm)
      ) {
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
    [focusableElements, onClose, onConfirm, enterConfirm],
  );

  return { onKeyDown };
}
