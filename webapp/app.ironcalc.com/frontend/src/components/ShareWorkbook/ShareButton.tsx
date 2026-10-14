import { Button } from "@ironcalc/workbook";
import { Share2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useSelectedUuid } from "../useStorage";

import "./share-workbook.css";

export function ShareButton(properties: { onClick: () => void }) {
  const { onClick } = properties;
  const { t } = useTranslation();
  // A workbook opened from a link is not stored locally until it is copied.
  const selectedUuid = useSelectedUuid();
  return (
    <Button
      type="button"
      disabled={!selectedUuid}
      onClick={onClick}
      startIcon={<Share2 />}
      className="app-ic-share-button"
      aria-label={t("file_bar.share_popover.button")}
    >
      <span className="app-ic-share-button-text">
        {t("file_bar.share_popover.button")}
      </span>
    </Button>
  );
}
