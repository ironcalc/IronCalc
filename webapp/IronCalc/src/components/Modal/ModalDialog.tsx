import type { KeyboardEventHandler, ReactNode, RefObject } from "react";
import { createPortal } from "react-dom";

interface ModalDialogProperties {
  modalRef: RefObject<HTMLDivElement | null>;
  titleId: string;
  onClose: () => void;
  onKeyDown: KeyboardEventHandler<HTMLDivElement>;
  className?: string;
  children: ReactNode;
}

export function ModalDialog({
  modalRef,
  titleId,
  onClose,
  onKeyDown,
  className,
  children,
}: ModalDialogProperties) {
  return createPortal(
    <div
      className="ic-modal-dialog-backdrop"
      onClick={onClose}
      onPointerDown={(event) => event.stopPropagation()}
      role="none"
    >
      <div
        ref={modalRef}
        className={["ic-modal-dialog", className].filter(Boolean).join(" ")}
        onClick={(event) => event.stopPropagation()}
        onKeyDown={onKeyDown}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        tabIndex={-1}
      >
        {children}
      </div>
    </div>,
    document.body,
  );
}
