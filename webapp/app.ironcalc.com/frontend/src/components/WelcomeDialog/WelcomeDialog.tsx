import "./welcome-dialog.css";
import {
  Button,
  IconButton,
  IronCalcIconWhite as IronCalcIcon,
} from "@ironcalc/workbook";
import { Table, X } from "lucide-react";
import { useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import TemplatesList from "./TemplatesList";
import TemplatesListItem from "./TemplatesListItem";

function WelcomeDialog(properties: {
  onClose: () => void;
  onSelectTemplate: (templateId: string) => void;
}) {
  const { t } = useTranslation();
  const [selectedTemplate, setSelectedTemplate] = useState<string>("blank");

  return createPortal(
    <div className="ic-modal-dialog-backdrop welcome-backdrop" role="none">
      <div
        className="ic-modal-dialog welcome-paper"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={(e) => e.code === "Escape" && properties.onClose()}
        role="dialog"
        aria-modal="true"
        tabIndex={-1}
      >
        <div className="welcome-header">
          <div className="welcome-header-brand">
            <div className="welcome-header-logo">
              <IronCalcIcon />
            </div>
            <span className="welcome-header-title">
              {t("welcome_dialog.title")}
            </span>
            <span className="welcome-header-subtitle">
              {t("welcome_dialog.subtitle")}
            </span>
          </div>
          <IconButton
            icon={<X />}
            aria-label={t("welcome_dialog.close_dialog")}
            size="xs"
            variant="ghost"
            onClick={properties.onClose}
          />
        </div>
        <div className="welcome-content">
          <div className="welcome-list-title">{t("welcome_dialog.new")}</div>
          <div className="welcome-templates-list">
            <TemplatesListItem
              title={t("welcome_dialog.blank_workbook")}
              description={t("welcome_dialog.blank_workbook_description")}
              icon={<Table />}
              iconColor="#F2994A"
              active={selectedTemplate === "blank"}
              onClick={() => setSelectedTemplate("blank")}
            />
          </div>
          <div className="welcome-list-title">
            {t("welcome_dialog.templates.templates")}
          </div>
          <TemplatesList
            selectedTemplate={selectedTemplate}
            handleTemplateSelect={setSelectedTemplate}
          />
        </div>
        <div className="welcome-footer">
          <Button onClick={() => properties.onSelectTemplate(selectedTemplate)}>
            {t("welcome_dialog.create_workbook")}
          </Button>
        </div>
      </div>
    </div>,
    document.body,
  );
}

export default WelcomeDialog;
