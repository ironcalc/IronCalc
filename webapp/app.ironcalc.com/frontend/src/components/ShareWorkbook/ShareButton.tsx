import { Button } from "@ironcalc/workbook";
import { Share2 } from "lucide-react";
import { useTranslation } from "react-i18next";

import "./share-workbook.css";

export function ShareButton(properties: { onClick: () => void }) {
  const { onClick } = properties;
  const { t } = useTranslation();
  return (
    <Button
      type="button"
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
