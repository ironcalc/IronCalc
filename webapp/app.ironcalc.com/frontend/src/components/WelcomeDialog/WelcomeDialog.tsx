import {
  IconButton,
  IronCalcIconWhite as IronCalcIcon,
} from "@ironcalc/workbook";
import { LayoutTemplate, Plus, Upload, X } from "lucide-react";
import { useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Trans, useTranslation } from "react-i18next";
import TemplatesList from "./TemplatesList";
import { useDialogFocus } from "./useDialogFocus";
import { useDialogKeyDown } from "./useDialogKeyDown";
import "./welcome-dialog.css";

function WelcomeDialog({
  onClose,
  onSelectTemplate,
  onModelUpload,
  onOpenTemplates,
}: {
  onClose: () => void;
  onSelectTemplate: (templateId: string) => void;
  onModelUpload: (arrayBuffer: ArrayBuffer, fileName: string) => Promise<void>;
  onOpenTemplates: () => void;
}) {
  const { t } = useTranslation();
  const [gridScrolled, setGridScrolled] = useState(false);
  const dialogRef = useDialogFocus(true);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const blankButtonRef = useRef<HTMLButtonElement>(null);
  const lastTemplateRef = useRef<HTMLButtonElement>(null);

  const { onKeyDown } = useDialogKeyDown({
    focusableElements: [blankButtonRef, lastTemplateRef],
    onClose,
  });

  const handleFileChange = async (
    event: React.ChangeEvent<HTMLInputElement>,
  ) => {
    const file = event.target.files?.[0];
    event.target.value = "";
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
          accept=".xlsx,application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
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
                <IconButton
                  className="app-ic-wd-mobile-close"
                  icon={<X />}
                  aria-label={t("welcome_dialog.close_dialog")}
                  onClick={onClose}
                />
              </div>
              <div className="app-ic-wd-action-group">
                <button
                  ref={blankButtonRef}
                  type="button"
                  className="app-ic-wd-action-button"
                  onClick={() => onSelectTemplate("blank")}
                >
                  <span className="app-ic-wd-action-button-icon">
                    <Plus />
                  </span>
                  {t("welcome_dialog.blank_workbook")}
                </button>
                <button
                  type="button"
                  className="app-ic-wd-action-button"
                  onClick={() => fileInputRef.current?.click()}
                >
                  <span className="app-ic-wd-action-button-icon">
                    <Upload />
                  </span>
                  {t("welcome_dialog.import_workbook")}
                </button>
                <button
                  type="button"
                  className="app-ic-wd-action-button app-ic-wd-action-button--mobile-only"
                  onClick={onOpenTemplates}
                >
                  <span className="app-ic-wd-action-button-icon">
                    <LayoutTemplate />
                  </span>
                  {t("welcome_dialog.templates.templates")}
                </button>
              </div>
            </div>
            <div className="app-ic-wd-storage-warning">
              <div>
                <Trans
                  i18nKey="welcome_dialog.storage_warning"
                  components={{
                    warn: (
                      <strong className="app-ic-wd-storage-warning-title" />
                    ),
                    docsLink: (
                      // biome-ignore lint/a11y/useAnchorContent: content is provided by the translation
                      <a
                        href="https://docs.ironcalc.com/web-application/about.html"
                        target="_blank"
                        rel="noopener noreferrer"
                      />
                    ),
                  }}
                />
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
              lastItemRef={lastTemplateRef}
            />
          </div>
        </div>
      </div>
    </div>,
    document.body,
  );
}

export default WelcomeDialog;
