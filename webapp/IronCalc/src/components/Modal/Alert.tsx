import { X } from "lucide-react";
import { type ReactNode, useId } from "react";
import { createPortal } from "react-dom";
import { Button } from "../Button/Button";
import { IconButton } from "../Button/IconButton";
import "./modal-dialog.css";
import { useModalFocus } from "./useModalFocus";
import { useModalKeyDown } from "./useModalKeyDown";

export interface AlertProperties {
  open: boolean;
  onClose: () => void;
  title: string;
  message: ReactNode;
  confirmLabel?: string;
}

export function Alert({
  open,
  onClose,
  title,
  message,
  confirmLabel = "OK",
}: AlertProperties) {
  const modalId = useId();
  const titleId = `${modalId}-title`;
  const { modalRef, restoreFocus } = useModalFocus(open);

  const closeModal = (): void => {
    onClose();
    restoreFocus();
  };

  const { onKeyDown } = useModalKeyDown({
    modalRef,
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
          <IconButton icon={<X />} aria-label="Close" onClick={closeModal} />
        </div>
        <div className="ic-modal-dialog-body">
          {typeof message === "string" ? <p>{message}</p> : message}
        </div>
        <div className="ic-modal-dialog-footer">
          <Button size="md" onClick={closeModal}>
            {confirmLabel}
          </Button>
        </div>
      </div>
    </div>,
    document.body,
  );
}

Alert.displayName = "Alert";
