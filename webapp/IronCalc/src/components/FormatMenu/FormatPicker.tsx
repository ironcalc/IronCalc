import { Check } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../Button/Button";
import { Dialog } from "../Dialog/Dialog";
import { Input } from "../Input/Input";

// FIXME: the stopPropagation everywhere is because of my bad implementation
// of keyboard handling in the spreadsheet

type FormatPickerProps = {
  className?: string;
  open: boolean;
  onClose: () => void;
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

  const onSubmit = (code: string): void => {
    properties.onChange(code);
    properties.onClose();
  };

  return (
    <Dialog
      open={properties.open}
      onClose={properties.onClose}
      onConfirm={() => onSubmit(formatCode)}
      title={t("num_fmt.title")}
      showHeader
      className={properties.className}
      footer={
        <Button
          size="md"
          startIcon={<Check />}
          onClick={() => onSubmit(formatCode)}
        >
          {t("num_fmt.save")}
        </Button>
      }
    >
      <Input
        autoFocus
        value={formatCode}
        name="format_code"
        onChange={(event) => setFormatCode(event.target.value)}
        onKeyDown={(event) => {
          event.stopPropagation();
          if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
            onSubmit(formatCode);
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
    </Dialog>
  );
};

export default FormatPicker;
