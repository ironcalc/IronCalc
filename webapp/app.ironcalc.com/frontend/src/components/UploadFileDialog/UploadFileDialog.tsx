import { IconButton, Portal } from "@ironcalc/workbook";
import { BookOpen, FileUp, X } from "lucide-react";
import { type DragEvent, useId, useRef } from "react";
import { useTranslation } from "react-i18next";
import { useDialogFocus } from "./useDialogFocus";
import { useDialogKeyDown } from "./useDialogKeyDown";
import "./upload-file-dialog.css";
import { useState } from "react";

function UploadFileDialog({
  onClose,
  onModelUpload,
}: {
  onClose: () => void;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
}) {
  const [hover, setHover] = useState(false);
  const [message, setMessage] = useState("");
  const fileInputRef = useRef<HTMLInputElement>(null);
  const closeButtonRef = useRef<HTMLButtonElement>(null);
  const dropZoneRef = useRef<HTMLButtonElement>(null);
  const footerLinkRef = useRef<HTMLAnchorElement>(null);
  const titleId = useId();
  const { t } = useTranslation();
  const dialogRef = useDialogFocus(true);

  const { onKeyDown } = useDialogKeyDown({
    focusableElements: [closeButtonRef, dropZoneRef, footerLinkRef],
    onClose,
  });

  const handleDragEnter = (event: DragEvent<HTMLElement>) => {
    event.preventDefault();
    event.stopPropagation();
    setHover(true);
  };

  const handleDragOver = (event: DragEvent<HTMLElement>) => {
    event.preventDefault();
    event.stopPropagation();
    event.dataTransfer.dropEffect = "copy";
    setHover(true);
  };

  const handleDragLeave = (event: DragEvent<HTMLElement>) => {
    event.preventDefault();
    event.stopPropagation();
    setHover(false);
  };

  const handleFileUpload = (file: File) => {
    setMessage(
      t("file_bar.file_menu.import.uploading", { fileName: file.name }),
    );
    const reader = new FileReader();
    reader.onload = async () => {
      try {
        await onModelUpload(reader.result as ArrayBuffer, file.name);
        onClose();
      } catch (e) {
        setMessage(`${e}`);
      }
    };
    reader.readAsArrayBuffer(file);
  };

  const handleDrop = (event: DragEvent<HTMLElement>) => {
    event.preventDefault();
    event.stopPropagation();
    const { items, files } = event.dataTransfer;
    if (items) {
      for (let i = 0; i < items.length; i++) {
        if (items[i].kind === "file") {
          const file = items[i].getAsFile();
          if (file) {
            handleFileUpload(file);
            return;
          }
        }
      }
    } else if (files.length > 0) {
      handleFileUpload(files[0]);
    }
  };

  return (
    <Portal>
      <div className="app-ic-ufd-backdrop" onClick={onClose} role="none">
        <div
          ref={dialogRef}
          className="app-ic-ufd-paper"
          onClick={(e) => e.stopPropagation()}
          onKeyDown={onKeyDown}
          role="dialog"
          aria-modal="true"
          aria-labelledby={titleId}
          tabIndex={-1}
        >
          <input
            ref={fileInputRef}
            type="file"
            multiple
            style={{ display: "none" }}
            onChange={(event) => {
              const files = event.target.files;
              if (files) {
                for (const file of files) {
                  handleFileUpload(file);
                }
              }
            }}
          />
          <div className="app-ic-ufd-header">
            <span id={titleId} className="app-ic-ufd-header-title">
              {t("file_bar.file_menu.import.title")}
            </span>
            <IconButton
              ref={closeButtonRef}
              icon={<X />}
              aria-label={t("file_bar.file_menu.import.close_dialog")}
              onClick={onClose}
            />
          </div>

          {message === "" ? (
            <button
              ref={dropZoneRef}
              type="button"
              className={`app-ic-ufd-drop-zone${hover ? " app-ic-ufd-drop-zone--dragging" : ""}`}
              onDragEnter={handleDragEnter}
              onDragOver={handleDragOver}
              onDragLeave={handleDragLeave}
              onDragExit={handleDragLeave}
              onDrop={handleDrop}
              onClick={() => fileInputRef.current?.click()}
            >
              {hover ? (
                <div>{t("file_bar.file_menu.import.drop_file_here")}</div>
              ) : (
                <>
                  <div className="app-ic-ufd-drop-zone-icon">
                    <FileUp />
                  </div>
                  <div className="app-ic-ufd-drop-zone-text">
                    <span>{t("file_bar.file_menu.import.subtitle")} </span>
                    <span className="app-ic-ufd-browse-link">
                      {t("file_bar.file_menu.import.subtitle_link")}
                    </span>
                  </div>
                </>
              )}
            </button>
          ) : (
            <div className="app-ic-ufd-drop-zone">
              <div>{message}</div>
            </div>
          )}

          <div className="app-ic-ufd-footer">
            <BookOpen />
            <a
              ref={footerLinkRef}
              className="app-ic-ufd-footer-link"
              href="https://docs.ironcalc.com/web-application/importing-files.html"
              target="_blank"
              rel="noopener noreferrer"
            >
              {t("file_bar.file_menu.import.learn_more")}
            </a>
          </div>
        </div>
      </div>
    </Portal>
  );
}

export default UploadFileDialog;
