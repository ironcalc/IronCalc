import { Button, type CollabProvider, Input } from "@ironcalc/workbook";
import { Check, Copy, Radio } from "lucide-react";
import { QRCodeSVG } from "qrcode.react";
import { useEffect, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { useModalFocus } from "../ShareWorkbook/useModalFocus";
import { useCollaborators } from "./CollabControls";

import "../ShareWorkbook/share-workbook.css";
import "./collab.css";

// The live-session dialog: the invite link (with QR code) and who is in
// the room right now. Styled after the share dialog.

function CollabDialog(properties: {
  provider: CollabProvider;
  onClose: () => void;
}) {
  const { provider, onClose } = properties;
  const { t } = useTranslation();
  const { modalRef, restoreFocus } = useModalFocus(true);
  const collaborators = useCollaborators(provider);
  const [copied, setCopied] = useState(false);

  const params = new URLSearchParams(window.location.search);
  const room = params.get("room") ?? "";
  const url = `${window.location.origin}/?room=${encodeURIComponent(room)}`;

  useEffect(() => {
    if (!copied) {
      return;
    }
    const timeoutId = setTimeout(() => setCopied(false), 2000);
    return () => clearTimeout(timeoutId);
  }, [copied]);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(url);
      setCopied(true);
    } catch (err) {
      console.error("Failed to copy text: ", err);
    }
  };

  const handleClose = () => {
    restoreFocus();
    onClose();
  };

  return createPortal(
    <div
      className="app-ic-share-dialog-backdrop"
      onClick={handleClose}
      role="none"
    >
      <div
        ref={modalRef}
        className="app-ic-share-dialog"
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
        <div className="app-ic-share-dialog-content">
          <div className="app-ic-share-dialog-qr-wrapper">
            <QRCodeSVG value={url} />
          </div>
          <div className="app-ic-share-dialog-url-wrapper">
            <Input disabled value={url} />
            <Button
              type="button"
              variant="secondary"
              startIcon={copied ? <Check /> : <Copy />}
              onClick={handleCopy}
            >
              {copied
                ? t("file_bar.collab.copied")
                : t("file_bar.collab.copy_url")}
            </Button>
          </div>
          <div className="app-ic-collab-dialog-people">
            {collaborators.map((collaborator) => (
              <div
                key={collaborator.clientId}
                className="app-ic-collab-dialog-person"
              >
                <span
                  className="app-ic-collab-avatar"
                  style={{ backgroundColor: collaborator.color }}
                >
                  {(collaborator.name[0] || "?").toUpperCase()}
                </span>
                <span className="app-ic-collab-dialog-person-name">
                  {collaborator.name}
                  {collaborator.isSelf && ` (${t("file_bar.collab.you")})`}
                </span>
              </div>
            ))}
          </div>
        </div>
        <div className="app-ic-share-dialog-footer">
          <Radio />
          {t("file_bar.collab.info_text")}
        </div>
      </div>
    </div>,
    document.body,
  );
}

export default CollabDialog;
