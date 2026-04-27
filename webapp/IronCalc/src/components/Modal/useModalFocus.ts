import { useEffect, useRef } from "react";

/**
 * Moves focus into the modal when it's open, and restores it to the previously
 * focused when closing it. Without this, closing the modal would leave
 * the user with no focused element which is unconvenient for keyboard-only users.
 */
export function useModalFocus(open: boolean) {
  const modalRef = useRef<HTMLDivElement>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (!open) {
      return;
    }

    previousFocusRef.current = document.activeElement as HTMLElement | null;

    requestAnimationFrame(() => {
      if (!modalRef.current?.contains(document.activeElement)) {
        modalRef.current?.focus();
      }
    });
  }, [open]);

  function restoreFocus(): void {
    previousFocusRef.current?.focus();
  }

  return { modalRef, restoreFocus };
}
