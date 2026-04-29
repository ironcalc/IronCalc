import { Button } from "@ironcalc/workbook";
import { Share2 } from "lucide-react";
import { useTranslation } from "react-i18next";

export function ShareButton(properties: { onClick: () => void }) {
  const { t } = useTranslation();
  return (
    <Button startIcon={<Share2 />} onClick={properties.onClick}>
      {t("file_bar.share_popover.button")}
    </Button>
  );
}
