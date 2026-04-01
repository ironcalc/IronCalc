import { Trash2 } from "lucide-react";
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../Button/Button";
import "./sheet-delete-dialog.css";

interface SheetDeleteDialogProps {
  open: boolean;
  onClose: () => void;
  onDelete: () => void;
  sheetName: string;
}

function SheetDeleteDialog({
  open,
  onClose,
  onDelete,
  sheetName,
}: SheetDeleteDialogProps) {
  const { t } = useTranslation();
  const deleteButtonRef = useRef<HTMLButtonElement>(null);
  const cancelButtonRef = useRef<HTMLButtonElement>(null);
  const previousFocusedElement = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (!open) {
      return;
    }

    previousFocusedElement.current =
      document.activeElement as HTMLElement | null;

    requestAnimationFrame(() => {
      deleteButtonRef.current?.focus();
    });
  }, [open]);

  const closeDialog = (): void => {
    onClose();
    previousFocusedElement.current?.focus();
  };

  const handleDelete = (): void => {
    onDelete();
    previousFocusedElement.current?.focus();
  };

  if (!open) {
    return null;
  }

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: FIXME
    <div
      className="ic-sheet-delete-dialog-backdrop"
      onClick={closeDialog}
      role="presentation"
    >
      <div
        className="ic-sheet-delete-dialog"
        onClick={(event) => event.stopPropagation()}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.stopPropagation();
            closeDialog();
            return;
          }

          if (event.key === "Tab") {
            const focusable = [
              deleteButtonRef.current,
              cancelButtonRef.current,
            ].filter(
              (element): element is HTMLButtonElement => element !== null,
            );

            if (focusable.length === 0) {
              event.preventDefault();
              return;
            }

            const currentIndex = focusable.indexOf(
              document.activeElement as HTMLButtonElement,
            );

            if (event.shiftKey) {
              if (currentIndex <= 0) {
                event.preventDefault();
                focusable[focusable.length - 1]?.focus();
              }
            } else if (currentIndex === focusable.length - 1) {
              event.preventDefault();
              focusable[0]?.focus();
            }
          }
        }}
        role="dialog"
        aria-modal="true"
        aria-label={t("sheet_delete.title")}
      >
        <div className="ic-sheet-delete-dialog-icon">
          <Trash2 />
        </div>

        <h2 className="ic-sheet-delete-dialog-title">
          {t("sheet_delete.title")}
        </h2>

        <p className="ic-sheet-delete-dialog-body">
          {t("sheet_delete.message", {
            sheetName,
          })}
        </p>

        <div className="ic-sheet-delete-dialog-buttons">
          <Button
            ref={deleteButtonRef}
            size="md"
            variant="destructive"
            onClick={handleDelete}
            className="ic-sheet-delete-dialog-delete-button"
          >
            {t("sheet_delete.confirm")}
          </Button>

          <Button
            ref={cancelButtonRef}
            size="md"
            variant="secondary"
            onClick={closeDialog}
            className="ic-sheet-delete-dialog-cancel-button"
          >
            {t("sheet_delete.cancel")}
          </Button>
        </div>
      </div>
    </div>
  );
}

export default SheetDeleteDialog;
