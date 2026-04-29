import "./share-workbook-modal.css";
import type { Model } from "@ironcalc/workbook";
import { Button, Input } from "@ironcalc/workbook";
import { Check, Copy, GlobeLock } from "lucide-react";
import { QRCodeSVG } from "qrcode.react";
import { useEffect, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { shareModel } from "../rpc";

function ShareWorkbookModal(properties: {
  onClose: () => void;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
  model?: Model;
}) {
  const [url, setUrl] = useState<string>("");
  const [copied, setCopied] = useState(false);
  const { t } = useTranslation();

  useEffect(() => {
    const generateUrl = async () => {
      if (properties.model) {
        const bytes = properties.model.toBytes();
        const hash = await shareModel(bytes);
        setUrl(`${location.origin}/?model=${hash}`);
      }
    };
    generateUrl();
  }, [properties.model]);

  useEffect(() => {
    let timeoutId: ReturnType<typeof setTimeout>;
    if (copied) {
      timeoutId = setTimeout(() => {
        setCopied(false);
      }, 2000);
    }
    return () => {
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
    };
  }, [copied]);

  const handleClose = () => {
    properties.onClose();
  };

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(url);
      setCopied(true);
    } catch (err) {
      console.error("Failed to copy text: ", err);
    }
  };

  return createPortal(
    <div
      className="ic-modal-dialog-backdrop share-modal-backdrop"
      onClick={handleClose}
      role="none"
    >
      <div
        className="ic-modal-dialog share-modal-paper"
        onClick={(event) => event.stopPropagation()}
        onKeyDown={(event) => {
          if (event.code === "Escape") {
            handleClose();
          }
        }}
        role="dialog"
        aria-modal="true"
        tabIndex={-1}
      >
        <div className="ic-modal-dialog-body share-modal-content">
          <div className="share-modal-qr-wrapper">
            <QRCodeSVG value={url} />
          </div>
          <div className="share-modal-url-wrapper">
            <Input readOnly disabled value={url} />
            <Button
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
        <div className="ic-modal-dialog-footer share-modal-footer">
          <GlobeLock />
          {t("file_bar.share_popover.info_text")}
        </div>
      </div>
    </div>,
    document.body,
  );
}

export default ShareWorkbookModal;
