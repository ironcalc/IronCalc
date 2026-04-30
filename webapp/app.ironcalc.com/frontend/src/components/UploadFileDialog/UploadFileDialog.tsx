import "./upload-file-dialog.css";
import { IconButton } from "@ironcalc/workbook";
import { BookOpen, FileUp, X } from "lucide-react";
import { type DragEvent, useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";

function UploadFileDialog(properties: {
  onClose: () => void;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
}) {
  const [hover, setHover] = useState(false);
  const [message, setMessage] = useState("");
  const fileInputRef = useRef<HTMLInputElement>(null);
  const closeButtonRef = useRef<HTMLButtonElement>(null);
  const { t } = useTranslation();
  const { onModelUpload } = properties;

  useEffect(() => {
    closeButtonRef.current?.focus();
    return () => {
      const root = document.getElementById("root");
      if (root) root.style.filter = "none";
    };
  }, []);

  const handleClose = () => properties.onClose();

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

  const handleFileUpload = (file: File) => {
    setMessage(
      t("file_bar.file_menu.import.uploading", { fileName: file.name }),
    );
    const reader = new FileReader();
    reader.onload = async () => {
      try {
        await onModelUpload(reader.result as ArrayBuffer, file.name);
        handleClose();
      } catch (e) {
        setMessage(`${e}`);
      }
    };
    reader.readAsArrayBuffer(file);
  };

  return createPortal(
    <div
      className="ic-modal-dialog-backdrop upload-dialog-backdrop"
      onClick={handleClose}
      role="none"
    >
      <div
        className="ic-modal-dialog upload-dialog-paper"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={(e) => e.code === "Escape" && handleClose()}
        role="dialog"
        aria-modal="true"
        tabIndex={-1}
      >
        <div className="upload-dialog-title">
          <span className="upload-dialog-title-text">
            {t("file_bar.file_menu.import.title")}
          </span>
          <IconButton
            ref={closeButtonRef}
            icon={<X />}
            aria-label={t("file_bar.file_menu.import.close_dialog")}
            size="xs"
            variant="ghost"
            onClick={handleClose}
          />
        </div>

        {message === "" ? (
          <label
            className={`upload-dropzone${hover ? " upload-dropzone--hover" : ""}`}
            onDragEnter={handleDragEnter}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDragExit={handleDragLeave}
            onDrop={handleDrop}
          >
            <input
              ref={fileInputRef}
              type="file"
              multiple
              accept="*"
              className="upload-dropzone-hidden-input"
              onChange={(event) => {
                const files = event.target.files;
                if (files) {
                  for (const file of files) handleFileUpload(file);
                }
              }}
            />
            {!hover ? (
              <>
                <div className="upload-dropzone-icon">
                  <FileUp />
                </div>
                <div className="upload-dropzone-subtitle">
                  {t("file_bar.file_menu.import.subtitle")}{" "}
                  <span className="upload-dropzone-link">
                    {t("file_bar.file_menu.import.subtitle_link")}
                  </span>
                </div>
              </>
            ) : (
              <div>{t("file_bar.file_menu.import.drop_file_here")}</div>
            )}
          </label>
        ) : (
          <div className="upload-dropzone">
            <div>{message}</div>
          </div>
        )}

        <div className="ic-modal-dialog-footer upload-dialog-footer">
          <BookOpen />
          <a
            className="upload-dialog-footer-link"
            href="https://docs.ironcalc.com/web-application/importing-files.html"
            target="_blank"
            rel="noopener noreferrer"
          >
            {t("file_bar.file_menu.import.learn_more")}
          </a>
        </div>
      </div>
    </div>,
    document.body,
  );
}

export default UploadFileDialog;
