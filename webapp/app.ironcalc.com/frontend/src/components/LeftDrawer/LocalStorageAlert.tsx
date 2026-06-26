import { IconButton } from "@ironcalc/workbook";
import { X } from "lucide-react";
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
    <div className="app-ic-drawer-alert">
      <p>
        <Trans
          i18nKey="welcome_dialog.storage_warning"
          components={{
            warn: <strong className="app-ic-wd-storage-warning-title" />,
            docsLink: (
              // biome-ignore lint/a11y/useAnchorContent: content is provided by the translation
              <a
                href="https://docs.ironcalc.com/web-application/about.html"
                target="_blank"
                rel="noopener noreferrer"
              />
            ),
          }}
        />
      </p>
      <IconButton
        icon={<X />}
        aria-label={t("left_drawer.alert.close")}
        onClick={handleClose}
      />
    </div>
  );
}

export default LocalStorageAlert;
