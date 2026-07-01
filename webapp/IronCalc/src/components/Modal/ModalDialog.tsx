import type { KeyboardEventHandler, ReactNode, RefObject } from "react";
import { useState } from "react";
import { createAnchoredPortal } from "../createAnchoredPortal";

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
  const [portalAnchor, setPortalAnchor] = useState<HTMLSpanElement | null>(
    null,
  );

  return (
    <>
      <span ref={setPortalAnchor} hidden />
      {createAnchoredPortal(
        <div
          className="ic-modal-dialog-backdrop"
          onClick={onClose}
          // HACK: prevents the workbook from stealing focus while the modal is open
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
        portalAnchor,
      )}
    </>
  );
}
