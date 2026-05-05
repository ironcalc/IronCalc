import { Button, Input, type Model } from "@ironcalc/workbook";
import { Check, Copy, GlobeLock } from "lucide-react";
import { QRCodeSVG } from "qrcode.react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { useModalFocus } from "./useModalFocus";
import { useShareDialog } from "./useShareDialog";

import "./share-dialog.css";

function ShareWorkbookDialog(properties: {
  onClose: () => void;
  model?: Model;
}) {
  const { t } = useTranslation();
  const { modalRef, restoreFocus } = useModalFocus(true);
  const { url, copied, handleCopy } = useShareDialog(properties.model);

  const handleClose = () => {
    restoreFocus();
    properties.onClose();
  };

  return createPortal(
    <div className="share-dialog-backdrop" onClick={handleClose} role="none">
      <div
        ref={modalRef}
        className="share-dialog"
        role="dialog"
        aria-modal="true"
        onClick={(event) => event.stopPropagation()}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            handleClose();
          }
        }}
        tabIndex={-1}
      >
        <div className="share-dialog-content">
          <div className="share-dialog-qr-wrapper">
            <QRCodeSVG value={url} />
          </div>
          <div className="share-dialog-url-wrapper">
            <Input disabled value={url} />
            <Button
              type="button"
              variant="secondary"
              startIcon={copied ? <Check /> : <Copy />}
              onClick={handleCopy}
            >
              {copied
                ? t("file_bar.share_popover.copied")
                : t("file_bar.share_popover.copy_url")}
            </Button>
          </div>
        </div>
        <div className="share-dialog-footer">
          <GlobeLock />
          {t("file_bar.share_popover.info_text")}
        </div>
      </div>
    </div>,
    document.body,
  );
}

export default ShareWorkbookDialog;
