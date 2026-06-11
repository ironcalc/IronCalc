import {
  IconButton,
  IronCalcIconWhite as IronCalcIcon,
} from "@ironcalc/workbook";
import { CloudOff, Plus, Upload, X } from "lucide-react";
import { useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import TemplatesList from "./TemplatesList";
import { useDialogFocus } from "./useDialogFocus";
import { useDialogKeyDown } from "./useDialogKeyDown";
import "./welcome-dialog.css";

function WelcomeDialog({
  onClose,
  onSelectTemplate,
  onModelUpload,
}: {
  onClose: () => void;
  onSelectTemplate: (templateId: string) => void;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
}) {
  const { t } = useTranslation();
  const [gridScrolled, setGridScrolled] = useState(false);
  const dialogRef = useDialogFocus(true);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const { onKeyDown } = useDialogKeyDown({
    focusableElements: [],
    onClose,
  });

  const handleFileChange = async (
    event: React.ChangeEvent<HTMLInputElement>,
  ) => {
    const file = event.target.files?.[0];
    if (!file) {
      return;
    }
    const arrayBuffer = await file.arrayBuffer();
    await onModelUpload(arrayBuffer, file.name);
    onClose();
  };

  return createPortal(
    <div className="app-ic-wd-backdrop" onClick={onClose} role="none">
      <div
        ref={dialogRef}
        className="app-ic-wd-paper"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={onKeyDown}
        role="dialog"
        aria-modal="true"
        aria-label={t("welcome_dialog.title")}
        tabIndex={-1}
      >
        <input
          ref={fileInputRef}
          type="file"
          style={{ display: "none" }}
          onChange={handleFileChange}
        />
        <div className="app-ic-wd-content app-ic-wd-content--two-col">
          <div className="app-ic-wd-col-actions">
            <div className="app-ic-wd-col-actions-top">
              <div className="app-ic-wd-logo-wrapper">
                <div className="app-ic-wd-logo-icon">
                  <IronCalcIcon />
                </div>
                <span className="app-ic-wd-logo-title">IronCalc</span>
              </div>
              <div className="app-ic-wd-action-group">
                <button
                  type="button"
                  className="app-ic-wd-action-button"
                  onClick={() => onSelectTemplate("blank")}
                >
                  <Plus />
                  {t("welcome_dialog.blank_workbook")}
                </button>
                <button
                  type="button"
                  className="app-ic-wd-action-button"
                  onClick={() => fileInputRef.current?.click()}
                >
                  <Upload />
                  {t("welcome_dialog.import_workbook")}
                </button>
              </div>
            </div>
            <div className="app-ic-wd-storage-warning">
              <CloudOff />
              <div>
                {t("file_bar.title_input.warning_text1")}
                <strong>{t("file_bar.title_input.warning_text2")}</strong>
              </div>
            </div>
          </div>
          <div className="app-ic-wd-col-templates">
            <div
              className={`app-ic-wd-list-title${gridScrolled ? " app-ic-wd-list-title--scrolled" : ""}`}
            >
              {t("welcome_dialog.templates.examples_and_templates")}
              <IconButton
                icon={<X />}
                aria-label={t("welcome_dialog.close_dialog")}
                onClick={onClose}
              />
            </div>
            <TemplatesList
              selectedTemplate=""
              handleTemplateSelect={onSelectTemplate}
              columns={3}
              onScroll={(e) => setGridScrolled(e.currentTarget.scrollTop > 0)}
            />
          </div>
        </div>
      </div>
    </div>,
    document.body,
  );
}

export default WelcomeDialog;
