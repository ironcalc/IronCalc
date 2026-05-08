import { IconButton } from "@ironcalc/workbook";
import { CircleAlert, X } from "lucide-react";
import { useState } from "react";
import { Trans, useTranslation } from "react-i18next";
import "./left-drawer.css";

const ALERT_DISMISSED_KEY = "localStorageAlertDismissed";

function LocalStorageAlert() {
  const { t } = useTranslation();
  const [isAlertVisible, setIsAlertVisible] = useState(
    () => localStorage.getItem(ALERT_DISMISSED_KEY) !== "true",
  );

  const handleClose = () => {
    setIsAlertVisible(false);
    localStorage.setItem(ALERT_DISMISSED_KEY, "true");
  };

  if (!isAlertVisible) {
    return null;
  }

  return (
    <div className="app-ic-drawer-alert">
      <div className="app-ic-drawer-alert-icon">
        <CircleAlert />
      </div>
      <div>
        <h2>{t("left_drawer.alert.title")}</h2>
        <p>{t("left_drawer.alert.subtitle")}</p>
        <p>
          <Trans
            i18nKey="left_drawer.alert.subtitle2"
            components={{ bold: <strong /> }}
          />
        </p>
      </div>
      <IconButton
        icon={<X />}
        aria-label={t("left_drawer.alert.close")}
        onClick={handleClose}
      />
    </div>
  );
}

export default LocalStorageAlert;
