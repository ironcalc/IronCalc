import { type KeyboardEvent, type RefObject, useCallback } from "react";

const FOCUSABLE_SELECTORS =
  ':is(a[href], button, input, select, textarea, [tabindex]):not([disabled]):not([tabindex="-1"])';

interface Options {
  modalRef: RefObject<HTMLDivElement | null>;
  onClose: () => void;
  onConfirm?: () => void;
}

export function useModalKeyDown({ modalRef, onClose, onConfirm }: Options) {
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
        const modal = modalRef.current;
        if (!modal) {
          return;
        }

        const focusable = Array.from(
          modal.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTORS),
        );

        if (focusable.length === 0) {
          event.preventDefault();
          return;
        }

        const first = focusable[0];
        const last = focusable[focusable.length - 1];

        if (event.shiftKey) {
          if (document.activeElement === first) {
            event.preventDefault();
            last?.focus();
          }
        } else if (document.activeElement === last) {
          event.preventDefault();
          first?.focus();
        }
      }
    },
    [modalRef, onClose, onConfirm],
  );

  return { onKeyDown };
}
