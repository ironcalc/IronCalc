import { IconButton } from "@ironcalc/workbook";
import { X } from "lucide-react";
import { useId, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import TemplatesList from "./TemplatesList";
import { TEMPLATE_CATEGORIES } from "./templates";
import { useDialogFocus } from "./useDialogFocus";
import { useDialogKeyDown } from "./useDialogKeyDown";
import "./templates-dialog.css";
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
  const titleId = useId();
  const [selectedCategory, setSelectedCategory] = useState("all");
  const [gridScrolled, setGridScrolled] = useState(false);
  const dialogRef = useDialogFocus(open);
  const closeButtonRef = useRef<HTMLButtonElement>(null);
  const lastTemplateRef = useRef<HTMLButtonElement>(null);

  const handleClose = () => {
    onClose();
  };

  const { onKeyDown } = useDialogKeyDown({
    focusableElements: [closeButtonRef, lastTemplateRef],
    onClose: handleClose,
  });

  if (!open) {
    return null;
  }

  return createPortal(
    <div className="app-ic-wd-backdrop" onClick={handleClose} role="none">
      <div
        ref={dialogRef}
        className="app-ic-wd-paper app-ic-wd-paper--templates"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={onKeyDown}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        tabIndex={-1}
      >
        <div className="app-ic-wd-template-header">
          <span id={titleId} className="app-ic-wd-template-header-title">
            {t("welcome_dialog.templates.choose_template")}
          </span>
          <IconButton
            ref={closeButtonRef}
            icon={<X />}
            aria-label={t("welcome_dialog.close_dialog")}
            onClick={handleClose}
          />
        </div>
        <div
          className={`app-ic-wd-filters${gridScrolled ? " app-ic-wd-filters--scrolled" : ""}`}
        >
          <button
            type="button"
            className={`app-ic-wd-filter-pill${selectedCategory === "all" ? " app-ic-wd-filter-pill--active" : ""}`}
            onClick={() => setSelectedCategory("all")}
          >
            {t("welcome_dialog.templates.category_all")}
          </button>
          {TEMPLATE_CATEGORIES.map(({ id, labelKey }) => (
            <button
              key={id}
              type="button"
              className={`app-ic-wd-filter-pill${selectedCategory === id ? " app-ic-wd-filter-pill--active" : ""}`}
              onClick={() => setSelectedCategory(id)}
            >
              {t(labelKey)}
            </button>
          ))}
        </div>
        <div className="app-ic-wd-content">
          <TemplatesList
            selectedTemplate=""
            handleTemplateSelect={onSelectTemplate}
            categoryFilter={selectedCategory}
            onScroll={(e) => setGridScrolled(e.currentTarget.scrollTop > 0)}
            lastItemRef={lastTemplateRef}
          />
        </div>
      </div>
    </div>,
    document.body,
  );
}

export default TemplatesDialog;
