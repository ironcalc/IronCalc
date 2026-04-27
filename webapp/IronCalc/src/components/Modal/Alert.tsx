import "./modal-dialog.css";
import { X } from "lucide-react";
import { type ReactNode, useId, useRef } from "react";
import { createPortal } from "react-dom";
import { Button } from "../Button/Button";
import { IconButton } from "../Button/IconButton";
import { useModalFocus } from "./useModalFocus";
import { useModalKeyDown } from "./useModalKeyDown";

export interface AlertProperties {
  open: boolean;
  onClose: () => void;
  title: string;
  message: ReactNode;
  confirmLabel?: string;
  closeLabel?: string;
}

export function Alert({
  open,
  onClose,
  title,
  message,
  confirmLabel = "OK",
  closeLabel = "Close",
}: AlertProperties) {
  const modalId = useId();
  const titleId = `${modalId}-title`;
  const { modalRef, restoreFocus } = useModalFocus(open);
  const closeButtonRef = useRef<HTMLButtonElement>(null);
  const confirmButtonRef = useRef<HTMLButtonElement>(null);

  const closeModal = (): void => {
    onClose();
    restoreFocus();
  };

  const { onKeyDown } = useModalKeyDown({
    focusableElements: [closeButtonRef, confirmButtonRef],
    onClose: closeModal,
    onConfirm: closeModal,
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
          <Button
            ref={confirmButtonRef}
            size="md"
            onClick={closeModal}
            autoFocus
          >
            {confirmLabel}
          </Button>
        </div>
      </div>
    </div>,
    document.body,
  );
}

Alert.displayName = "Alert";
