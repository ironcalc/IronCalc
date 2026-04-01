import { Check, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../Button/Button";
import { IconButton } from "../Button/IconButton";
import { Input } from "../Input/Input";
import "./format-picker.css";

// FIXME: This control should be a modal prompt
// FIXME: the stopPropagation everywhere is because of my bad implementation
// of keyboard handling in the spreadsheet

type FormatPickerProps = {
  className?: string;
  open: boolean;
  onClose: () => void;
  onExited: () => void;
  numFmt: string;
  onChange: (numberFmt: string) => void;
};

// Returns a list of focusable elements inside the dialog.
function getDialogElements(dialog: HTMLDivElement | null): HTMLElement[] {
  if (!dialog) {
    return [];
  }

  return Array.from(dialog.querySelectorAll<HTMLElement>("button, input"));
}

const FormatPicker = (properties: FormatPickerProps) => {
  const { t } = useTranslation();
  const [formatCode, setFormatCode] = useState(properties.numFmt);
  const dialogElement = useRef<HTMLDivElement>(null);
  const previousFocusedElement = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (properties.open) {
      setFormatCode(properties.numFmt);
    }
  }, [properties.numFmt, properties.open]);

  const closeDialog = (): void => {
    properties.onClose();
    previousFocusedElement.current?.focus();
  };

  useEffect(() => {
    if (!properties.open) {
      return;
    }

    previousFocusedElement.current =
      document.activeElement as HTMLElement | null;

    requestAnimationFrame(() => {
      const focusable = getDialogElements(dialogElement.current);
      focusable[0]?.focus();
    });
  }, [properties.open]);

  const handleClose = () => {
    closeDialog();
  };

  const onSubmit = (format_code: string): void => {
    properties.onChange(format_code);
    closeDialog();
  };

  if (!properties.open) {
    return null;
  }

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: FIXME
    <div
      className="ic-format-picker-backdrop"
      onClick={closeDialog}
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          event.stopPropagation();
          closeDialog();
        }
      }}
      role="presentation"
    >
      <div
        className={`ic-format-picker-dialog${properties.className ? ` ${properties.className}` : ""}`}
        onClick={(event) => event.stopPropagation()}
        onKeyDown={(event) => {
          if (event.key === "Tab") {
            const focusable = getDialogElements(dialogElement.current);

            if (focusable.length === 0) {
              event.preventDefault();
              return;
            }

            const first = focusable[0];
            const last = focusable[focusable.length - 1];
            const activeElement = document.activeElement;

            if (event.shiftKey) {
              if (activeElement === first) {
                event.preventDefault();
                last?.focus();
              }
            } else if (activeElement === last) {
              event.preventDefault();
              first?.focus();
            }
          } else if (event.key === "Escape") {
            event.stopPropagation();
            closeDialog();
          }
        }}
        ref={dialogElement}
        role="dialog"
        aria-modal="true"
        aria-label={t("num_fmt.title")}
      >
        <div className="ic-format-picker-title">
          {t("num_fmt.title")}
          <IconButton
            icon={<X />}
            onClick={handleClose}
            title={t("num_fmt.close")}
            aria-label={t("num_fmt.close")}
          />
        </div>

        <div className="ic-format-picker-content">
          <Input
            autoFocus
            value={formatCode}
            name="format_code"
            onChange={(event) => setFormatCode(event.target.value)}
            onKeyDown={(event) => {
              event.stopPropagation();
              if (event.key === "Enter") {
                onSubmit(formatCode);
              } else if (event.key === "Escape") {
                event.stopPropagation();
                closeDialog();
              }
            }}
            spellCheck={false}
            onClick={(event) => event.stopPropagation()}
            onFocus={(event) => event.target.select()}
            onPaste={(event) => event.stopPropagation()}
            onCopy={(event) => event.stopPropagation()}
            onCut={(event) => event.stopPropagation()}
          />
        </div>

        <div className="ic-format-picker-footer">
          <Button
            size="md"
            startIcon={<Check />}
            onClick={() => onSubmit(formatCode)}
          >
            {t("num_fmt.save")}
          </Button>
        </div>
      </div>
    </div>
  );
};

export default FormatPicker;
