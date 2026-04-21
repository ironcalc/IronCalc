import { X } from "lucide-react";
import { type ReactNode, useEffect, useId, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Button } from "../Button/Button";
import { IconButton } from "../Button/IconButton";
import { Input, type InputProperties } from "../Input/Input";
import { useModalFocus } from "./useModalFocus";
import { useModalKeyDown } from "./useModalKeyDown";

export interface PromptProperties {
  open: boolean;
  onClose: () => void;
  onSubmit: (value: string) => void;
  title: string;
  message?: ReactNode;
  defaultValue?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  closeLabel?: string;
  className?: string;
  inputProps?: Omit<InputProperties, "value" | "onChange">;
}

export function Prompt({
  open,
  onClose,
  onSubmit,
  title,
  message,
  defaultValue = "",
  confirmLabel = "OK",
  cancelLabel = "Cancel",
  closeLabel = "Close",
  className,
  inputProps,
}: PromptProperties) {
  const modalId = useId();
  const titleId = `${modalId}-title`;
  const { modalRef, restoreFocus } = useModalFocus(open);
  const closeButtonRef = useRef<HTMLButtonElement>(null);
  const submitButtonRef = useRef<HTMLButtonElement>(null);
  const [value, setValue] = useState(defaultValue);

  useEffect(() => {
    if (open) {
      setValue(defaultValue);
    }
  }, [open, defaultValue]);

  const closeModal = (): void => {
    onClose();
    restoreFocus();
  };

  const handleSubmit = (): void => {
    onSubmit(value);
    closeModal();
  };

  const { onKeyDown } = useModalKeyDown({
    focusableElements: [closeButtonRef, submitButtonRef],
    onClose: closeModal,
    onConfirm: handleSubmit,
  });

  if (!open) {
    return null;
  }

  return createPortal(
    <div className="ic-modal-dialog-backdrop" onClick={closeModal} role="none">
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
          {message &&
            (typeof message === "string" ? <p>{message}</p> : message)}
          <Input
            {...inputProps}
            value={value}
            onChange={(event) => setValue(event.target.value)}
            onKeyDown={(event) => {
              event.stopPropagation();
              if (event.key === "Escape") {
                event.preventDefault();
                closeModal();
              } else if (
                event.key === "Enter" &&
                (event.metaKey || event.ctrlKey)
              ) {
                event.preventDefault();
                handleSubmit();
              }
              inputProps?.onKeyDown?.(event);
            }}
            onClick={(event) => {
              event.stopPropagation();
              inputProps?.onClick?.(event);
            }}
          />
        </div>
        <div className="ic-modal-dialog-footer">
          <Button size="md" variant="secondary" onClick={closeModal}>
            {cancelLabel}
          </Button>
          <Button ref={submitButtonRef} size="md" onClick={handleSubmit}>
            {confirmLabel}
          </Button>
        </div>
      </div>
    </div>,
    document.body,
  );
}

Prompt.displayName = "Prompt";
