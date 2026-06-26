import { CloudOff } from "lucide-react";
import { type RefObject, useId, useState } from "react";
import { createPortal } from "react-dom";
import { Trans } from "react-i18next";
import { usePopoverPosition } from "./usePopoverPosition";

export function StorageWarning() {
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
          <p>
            <Trans
              i18nKey="file_bar.cloud_warning"
              components={{
                warn: <strong className="app-ic-wd-storage-warning-title" />,
              }}
            />
          </p>
        </div>,
        document.body,
      )}
    </>
  );
}
