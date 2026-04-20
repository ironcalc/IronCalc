import { type KeyboardEvent, type RefObject, useCallback } from "react";

interface Options {
  first: RefObject<HTMLElement | null>;
  last: RefObject<HTMLElement | null>;
  onClose: () => void;
  onConfirm?: () => void;
}

export function useModalKeyDown({ first, last, onClose, onConfirm }: Options) {
  const onKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.stopPropagation();
        onClose();
        return;
      }

      if (event.key === "Enter") {
        if (event.metaKey || event.ctrlKey) {
          event.preventDefault();
          onConfirm?.();
          return;
        }

        // Prevent Enter from submitting when focus is not on a button
        const target = event.target as HTMLElement;
        if (target.tagName !== "BUTTON") {
          event.preventDefault();
        }
        return;
      }

      if (event.key === "Tab") {
        const firstEl = first.current;
        const lastEl = last.current;
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
    [first, last, onClose, onConfirm],
  );

  return { onKeyDown };
}
