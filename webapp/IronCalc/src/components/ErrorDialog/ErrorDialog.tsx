import { AlertCircle } from "lucide-react";
import { useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { Button } from "../Button/Button";
import "./error-dialog.css";

interface ErrorDialogProps {
  open: boolean;
  onClose: () => void;
  title: string;
  message?: string;
}

function ErrorDialog({ open, onClose, title, message }: ErrorDialogProps) {
  const { t } = useTranslation();
  const okButtonRef = useRef<HTMLButtonElement>(null);
  const previousFocusedElement = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (!open) {
      return;
    }

    previousFocusedElement.current =
      document.activeElement as HTMLElement | null;

    requestAnimationFrame(() => {
      okButtonRef.current?.focus();
    });
  }, [open]);

  const closeDialog = (): void => {
    onClose();
    previousFocusedElement.current?.focus();
  };

  if (!open) {
    return null;
  }

  return createPortal(
    // biome-ignore lint/a11y/noStaticElementInteractions: FIXME
    <div
      className="ic-error-dialog-backdrop"
      onClick={closeDialog}
      role="presentation"
    >
      <div
        className="ic-error-dialog"
        onClick={(event) => event.stopPropagation()}
        onKeyDown={(event) => {
          if (event.key === "Escape" || event.key === "Enter") {
            event.stopPropagation();
            closeDialog();
          }
        }}
        role="alertdialog"
        aria-modal="true"
        aria-label={title}
      >
        <div className="ic-error-dialog-icon">
          <AlertCircle />
        </div>

        <h2 className="ic-error-dialog-title">{title}</h2>

        {message && <p className="ic-error-dialog-body">{message}</p>}

        <div className="ic-error-dialog-buttons">
          <Button
            ref={okButtonRef}
            size="md"
            variant="primary"
            onClick={closeDialog}
            className="ic-error-dialog-ok-button"
          >
            {t("error_dialog.ok")}
          </Button>
        </div>
      </div>
    </div>,
    document.body,
  );
}

export default ErrorDialog;
