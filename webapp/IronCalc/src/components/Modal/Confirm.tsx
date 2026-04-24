import "./modal-dialog.css";
import { X } from "lucide-react";
import { type ReactNode, useId, useRef } from "react";
import { createPortal } from "react-dom";
import { Button } from "../Button/Button";
import { IconButton } from "../Button/IconButton";
import { useModalFocus } from "./useModalFocus";
import { useModalKeyDown } from "./useModalKeyDown";

export interface ConfirmProperties {
  open: boolean;
  onClose: () => void;
  onConfirm: () => void;
  title: string;
  message: ReactNode;
  confirmLabel?: string;
  cancelLabel?: string;
  closeLabel?: string;
  variant?: "default" | "destructive";
}

export function Confirm({
  open,
  onClose,
  onConfirm,
  title,
  message,
  confirmLabel = "OK",
  cancelLabel = "Cancel",
  closeLabel = "Close",
  variant = "default",
}: ConfirmProperties) {
  const modalId = useId();
  const titleId = `${modalId}-title`;
  const { modalRef, restoreFocus } = useModalFocus(open);
  const closeButtonRef = useRef<HTMLButtonElement>(null);
  const confirmButtonRef = useRef<HTMLButtonElement>(null);

  const closeModal = (): void => {
    onClose();
    restoreFocus();
  };

  const handleConfirm = (): void => {
    onConfirm();
    closeModal();
  };

  const { onKeyDown } = useModalKeyDown({
    focusableElements: [closeButtonRef, confirmButtonRef],
    onClose: closeModal,
    onConfirm: handleConfirm,
  });

  if (!open) {
    return null;
  }

  return createPortal(
    <div className="ic-modal-dialog-backdrop" onClick={closeModal} role="none">
      <div
        ref={modalRef}
        className="ic-modal-dialog"
        onClick={(event) => event.stopPropagation()}
        onKeyDown={onKeyDown}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        tabIndex={-1}
      >
        <div className="ic-modal-dialog-header">
          <h2 id={titleId}>{title}</h2>
          <IconButton
            ref={closeButtonRef}
            icon={<X />}
            aria-label={closeLabel}
            onClick={closeModal}
          />
        </div>
        <div className="ic-modal-dialog-body">
          {typeof message === "string" ? <p>{message}</p> : message}
        </div>
        <div className="ic-modal-dialog-footer">
          <Button size="md" variant="secondary" onClick={closeModal}>
            {cancelLabel}
          </Button>
          <Button
            ref={confirmButtonRef}
            size="md"
            autoFocus
            variant={variant === "destructive" ? "destructive" : undefined}
            onClick={handleConfirm}
          >
            {confirmLabel}
          </Button>
        </div>
      </div>
    </div>,
    document.body,
  );
}

Confirm.displayName = "Confirm";
