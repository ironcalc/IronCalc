import { useTooltipPosition } from "@ironcalc/workbook";
import { CloudOff } from "lucide-react";
import { useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";

export function StorageWarning() {
  const { t } = useTranslation();
  const [visible, setVisible] = useState(false);
  const { triggerRef, tooltipRef, position } = useTooltipPosition(visible);

  return (
    <>
      <span
        ref={triggerRef}
        role="none"
        onMouseEnter={() => setVisible(true)}
        onMouseLeave={() => setVisible(false)}
      >
        <div className="file-bar-cloud-button">
          <CloudOff />
        </div>
      </span>
      {createPortal(
        <div
          ref={tooltipRef}
          className="file-bar-cloud-popover-content"
          data-visible={visible}
          style={position}
        >
          {t("file_bar.title_input.warning_text1")}
          <strong>{t("file_bar.title_input.warning_text2")}</strong>
        </div>,
        document.body,
      )}
    </>
  );
}
