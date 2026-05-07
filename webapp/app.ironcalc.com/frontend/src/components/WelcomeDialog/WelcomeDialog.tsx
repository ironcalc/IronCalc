import {
  Button,
  IconButton,
  IronCalcIconWhite as IronCalcIcon,
} from "@ironcalc/workbook";
import { Table, X } from "lucide-react";
import type { ReactNode } from "react";
import { useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import TemplatesList from "./TemplatesList";
import TemplatesListItem from "./TemplatesListItem";
import { useDialogFocus } from "./useDialogFocus";
import { useDialogKeyDown } from "./useDialogKeyDown";
import "./welcome-dialog.css";

export function DialogHeaderLogoWrapper({
  className,
  children,
}: {
  className?: string;
  children: ReactNode;
}) {
  return (
    <div className={`app-ic-wd-header-logo${className ? ` ${className}` : ""}`}>
      {children}
    </div>
  );
}

function WelcomeDialog({
  onClose,
  onSelectTemplate,
}: {
  onClose: () => void;
  onSelectTemplate: (templateId: string) => void;
}) {
  const { t } = useTranslation();
  const [selectedTemplate, setSelectedTemplate] = useState<string>("blank");
  const { dialogRef, restoreFocus } = useDialogFocus(true);
  const closeButtonRef = useRef<HTMLButtonElement>(null);
  const confirmButtonRef = useRef<HTMLButtonElement>(null);

  const handleClose = () => {
    onClose();
    restoreFocus();
  };

  const { onKeyDown } = useDialogKeyDown({
    focusableElements: [closeButtonRef, confirmButtonRef],
    onClose: handleClose,
  });

  return createPortal(
    <div className="app-ic-wd-backdrop" onClick={handleClose} role="none">
      <div
        ref={dialogRef}
        className="app-ic-wd-paper"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={onKeyDown}
        role="dialog"
        aria-modal="true"
        tabIndex={-1}
      >
        <div className="app-ic-wd-welcome-header">
          <div className="app-ic-wd-header-body">
            <DialogHeaderLogoWrapper>
              <IronCalcIcon />
            </DialogHeaderLogoWrapper>
            <span className="app-ic-wd-header-title">
              {t("welcome_dialog.title")}
            </span>
            <span className="app-ic-wd-header-subtitle">
              {t("welcome_dialog.subtitle")}
            </span>
          </div>
          <IconButton
            ref={closeButtonRef}
            icon={<X />}
            aria-label={t("welcome_dialog.close_dialog")}
            onClick={handleClose}
          />
        </div>
        <div className="app-ic-wd-content">
          <div className="app-ic-wd-list-title">{t("welcome_dialog.new")}</div>
          <div className="app-ic-wd-templates-list">
            <TemplatesListItem
              title={t("welcome_dialog.blank_workbook")}
              description={t("welcome_dialog.blank_workbook_description")}
              icon={<Table />}
              iconColor="#F2994A"
              active={selectedTemplate === "blank"}
              onClick={() => setSelectedTemplate("blank")}
            />
          </div>
          <div className="app-ic-wd-list-title">
            {t("welcome_dialog.templates.templates")}
          </div>
          <TemplatesList
            selectedTemplate={selectedTemplate}
            handleTemplateSelect={setSelectedTemplate}
          />
        </div>
        <div className="app-ic-wd-footer">
          <Button
            ref={confirmButtonRef}
            onClick={() => onSelectTemplate(selectedTemplate)}
          >
            {t("welcome_dialog.create_workbook")}
          </Button>
        </div>
      </div>
    </div>,
    document.body,
  );
}

export default WelcomeDialog;
