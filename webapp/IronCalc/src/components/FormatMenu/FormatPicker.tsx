import { Check, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../Button/Button";
import { IconButton } from "../Button/IconButton";
import { Input } from "../Input/Input";
import "./format-picker.css";

// FIXME: This control should be a modal prompt
// FIXME: the stopPropagation everywhere is becaus eof my bad implementation
// of keyboard handling in the spreadsheet

type FormatPickerProps = {
  className?: string;
  open: boolean;
  onClose: () => void;
  onExited: () => void;
  numFmt: string;
  onChange: (numberFmt: string) => void;
};

const FormatPicker = (properties: FormatPickerProps) => {
  const { t } = useTranslation();
  const [formatCode, setFormatCode] = useState(properties.numFmt);

  useEffect(() => {
    if (properties.open) {
      setFormatCode(properties.numFmt);
    }
  }, [properties.numFmt, properties.open]);

  const handleClose = () => {
    properties.onClose();
  };

  const onSubmit = (format_code: string): void => {
    properties.onChange(format_code);
    properties.onClose();
  };

  if (!properties.open) {
    return null;
  }

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: FIXME
    <div
      className="ic-format-picker-backdrop"
      onClick={properties.onClose}
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          event.stopPropagation();
          properties.onClose();
        }
      }}
      role="presentation"
    >
      {/** biome-ignore lint/a11y/useKeyWithClickEvents: FIXME */}
      <div
        className={`ic-format-picker-dialog${properties.className ? ` ${properties.className}` : ""}`}
        onClick={(event) => event.stopPropagation()}
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
            defaultValue={properties.numFmt}
            name="format_code"
            onChange={(event) => setFormatCode(event.target.value)}
            onKeyDown={(event) => {
              event.stopPropagation();
              if (event.key === "Enter") {
                onSubmit(formatCode);
                properties.onClose();
              } else if (event.key === "Escape") {
                event.stopPropagation();
                properties.onClose();
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
