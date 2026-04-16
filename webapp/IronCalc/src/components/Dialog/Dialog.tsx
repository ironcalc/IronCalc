import { X } from "lucide-react";
import { type ReactNode, useId } from "react";
import { createPortal } from "react-dom";

import { IconButton } from "../Button/IconButton";
import "./dialog.css";
import { useDialogFocus } from "./useDialogFocus";
import { useDialogKeyDown } from "./useDialogKeyDown";

/**
 * Reusable Dialog component with backdrop and portal rendering.
 * Closes on backdrop click or Esc key, and restores focus on close.
 * Use `showHeader` to render a header with title and an X close button.
 * Use `onConfirm` to handle Cmd/Ctrl+Enter.
 */

export interface DialogProperties {
  open: boolean;
  onClose: () => void;
  onConfirm?: () => void;
  title?: ReactNode;
  showHeader?: boolean;
  footer?: ReactNode;
  children: ReactNode;
  className?: string;
}

export function Dialog({
  open,
  onClose,
  onConfirm,
  title,
  showHeader = false,
  footer,
  children,
  className,
}: DialogProperties) {
  const dialogId = useId();
  const titleId = `${dialogId}-title`;
  const { dialogRef, restoreFocus } = useDialogFocus(open);

  const closeDialog = (): void => {
    onClose();
    restoreFocus();
  };

  const { onKeyDown } = useDialogKeyDown({
    dialogRef,
    onClose: closeDialog,
    onConfirm,
  });

  if (!open) {
    return null;
  }

  return createPortal(
    // biome-ignore lint/a11y/noStaticElementInteractions: FIXME
    <div
      className="ic-dialog-backdrop"
      onClick={closeDialog}
      role="presentation"
    >
      <div
        ref={dialogRef}
        className={["ic-dialog", className].filter(Boolean).join(" ")}
        onClick={(event) => event.stopPropagation()}
        onKeyDown={onKeyDown}
        role="dialog"
        aria-modal="true"
        aria-labelledby={showHeader && title ? titleId : undefined}
        tabIndex={-1}
      >
        {showHeader && title && (
          <div className="ic-dialog-header">
            <h2 id={titleId}>{title}</h2>

            <IconButton icon={<X />} aria-label="Close" onClick={closeDialog} />
          </div>
        )}

        <div className="ic-dialog-body">{children}</div>

        {footer && <div className="ic-dialog-footer">{footer}</div>}
      </div>
    </div>,
    document.body,
  );
}

Dialog.displayName = "Dialog";
