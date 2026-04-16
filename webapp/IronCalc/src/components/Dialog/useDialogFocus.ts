import { useEffect, useRef } from "react";

export function useDialogFocus(open: boolean) {
  const dialogRef = useRef<HTMLDivElement>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (!open) {
      return;
    }

    previousFocusRef.current = document.activeElement as HTMLElement | null;

    requestAnimationFrame(() => {
      dialogRef.current?.focus();
    });
  }, [open]);

  function restoreFocus(): void {
    previousFocusRef.current?.focus();
  }

  return { dialogRef, restoreFocus };
}
