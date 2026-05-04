import { Button } from "@ironcalc/workbook";
import { Share2 } from "lucide-react";
import { useTranslation } from "react-i18next";

import "./share-button.css";

export function ShareButton(properties: { onClick: () => void }) {
  const { onClick } = properties;
  const { t } = useTranslation();
  return (
    <Button onClick={onClick} startIcon={<Share2 />} className="share-button">
      <span className="share-button__text">
        {t("file_bar.share_popover.button")}
      </span>
    </Button>
  );
}
