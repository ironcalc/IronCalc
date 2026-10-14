import { Button } from "@ironcalc/workbook";
import { Info } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useSelectedUuid } from "./useStorage";

// Shown when the open workbook is not stored in the browser (opened from a
// shared link or an example). Edits there are not saved until it is copied.
export function UnsavedWorkbookBar(properties: { onSaveCopy: () => void }) {
  const { t } = useTranslation();
  const selectedUuid = useSelectedUuid();

  if (selectedUuid) {
    return null;
  }

  return (
    <div className="app-ic-unsaved-bar" role="status">
      <Info />
      <span>{t("unsaved_bar.message")}</span>
      <Button size="sm" onClick={properties.onSaveCopy}>
        {t("unsaved_bar.save_copy")}
      </Button>
    </div>
  );
}
