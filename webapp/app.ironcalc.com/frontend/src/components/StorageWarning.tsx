import { CloudOff } from "lucide-react";
import { type RefObject, useId, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { usePopoverPosition } from "./usePopoverPosition";

export function StorageWarning() {
  const { t } = useTranslation();
  const popoverId = useId();
  const [visible, setVisible] = useState(false);
  const { triggerRef, popoverRef, position } = usePopoverPosition(visible);

  return (
    <>
      <button
        ref={triggerRef as unknown as RefObject<HTMLButtonElement>}
        type="button"
        className="app-ic-file-bar-cloud-button"
        aria-describedby={visible ? popoverId : undefined}
        onMouseEnter={() => setVisible(true)}
        onMouseLeave={() => setVisible(false)}
        onFocus={() => setVisible(true)}
        onBlur={() => setVisible(false)}
        onPointerDown={(e) => {
          if (e.pointerType === "touch") {
            e.preventDefault();
            setVisible((v) => !v);
          }
        }}
      >
        <CloudOff />
      </button>
      {createPortal(
        <div
          ref={popoverRef}
          id={popoverId}
          role="tooltip"
          className="app-ic-file-bar-cloud-popover-content"
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
