import { CircleAlert, X } from "lucide-react";
import { useState } from "react";
import { Trans, useTranslation } from "react-i18next";

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
    <div className="storage-alert">
      <div className="storage-alert-icon">
        <CircleAlert />
      </div>
      <div className="storage-alert-message">
        <h2 className="storage-alert-title">{t("left_drawer.alert.title")}</h2>
        <p className="storage-alert-body">{t("left_drawer.alert.subtitle")}</p>
        <p className="storage-alert-body">
          <Trans
            i18nKey="left_drawer.alert.subtitle2"
            components={{ bold: <strong /> }}
          />
        </p>
      </div>
      <button
        type="button"
        className="storage-alert-close"
        onClick={handleClose}
        aria-label={t("left_drawer.alert.dismiss")}
      >
        <X />
      </button>
    </div>
  );
}

export default LocalStorageAlert;
