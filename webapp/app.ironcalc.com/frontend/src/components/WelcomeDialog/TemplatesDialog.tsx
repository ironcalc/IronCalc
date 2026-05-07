import { Button, IconButton } from "@ironcalc/workbook";
import { X } from "lucide-react";
import { useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import TemplatesList from "./TemplatesList";
import { useDialogFocus } from "./useDialogFocus";
import { useDialogKeyDown } from "./useDialogKeyDown";
import "./welcome-dialog.css";

interface TemplatesDialogProperties {
  open: boolean;
  onClose: () => void;
  onSelectTemplate: (templateId: string) => void;
}

function TemplatesDialog({
  open,
  onClose,
  onSelectTemplate,
}: TemplatesDialogProperties) {
  const { t } = useTranslation();
  const [selectedTemplate, setSelectedTemplate] = useState<string>("");
  const { dialogRef, restoreFocus } = useDialogFocus(open);
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

  if (!open) {
    return null;
  }

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
        <div className="app-ic-wd-template-header">
          <span className="app-ic-wd-template-header-title">
            {t("welcome_dialog.templates.choose_template")}
          </span>
          <IconButton
            ref={closeButtonRef}
            icon={<X />}
            aria-label={t("welcome_dialog.close_dialog")}
            onClick={handleClose}
          />
        </div>
        <div className="app-ic-wd-content">
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

export default TemplatesDialog;
