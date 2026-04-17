import { useEffect, useRef } from "react";

export function useModalFocus(open: boolean) {
  const modalRef = useRef<HTMLDivElement>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (!open) {
      return;
    }

    previousFocusRef.current = document.activeElement as HTMLElement | null;

    requestAnimationFrame(() => {
      modalRef.current?.focus();
    });
  }, [open]);

  function restoreFocus(): void {
    previousFocusRef.current?.focus();
  }

  return { modalRef, restoreFocus };
}
