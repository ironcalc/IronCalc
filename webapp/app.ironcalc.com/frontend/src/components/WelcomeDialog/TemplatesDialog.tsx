import "./welcome-dialog.css";
import { Button, IconButton } from "@ironcalc/workbook";
import { X } from "lucide-react";
import { useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import TemplatesList from "./TemplatesList";

function TemplatesDialog(properties: {
  onClose: () => void;
  onSelectTemplate: (templateId: string) => void;
}) {
  const { t } = useTranslation();
  const [selectedTemplate, setSelectedTemplate] = useState<string>("");

  return createPortal(
    <div
      className="ic-modal-dialog-backdrop welcome-backdrop"
      role="none"
      onClick={properties.onClose}
    >
      <div
        className="ic-modal-dialog welcome-paper"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={(e) => e.code === "Escape" && properties.onClose()}
        role="dialog"
        aria-modal="true"
        tabIndex={-1}
      >
        <div className="templates-header">
          <span>{t("welcome_dialog.templates.choose_template")}</span>
          <IconButton
            icon={<X />}
            aria-label={t("welcome_dialog.close_dialog")}
            size="xs"
            variant="ghost"
            onClick={properties.onClose}
          />
        </div>
        <div className="welcome-content">
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

export default TemplatesDialog;
