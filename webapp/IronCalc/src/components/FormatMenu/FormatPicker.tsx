import { styled, TextField } from "@mui/material";
import { Check } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../Button/Button";
import { Dialog } from "../Dialog/Dialog";

type FormatPickerProps = {
  open: boolean;
  onClose: () => void;
  numFmt: string;
  onChange: (numberFmt: string) => void;
};

const FormatPicker = (properties: FormatPickerProps) => {
  const { t } = useTranslation();
  const [formatCode, setFormatCode] = useState(properties.numFmt);

  useEffect(() => {
    if (!properties.open) return;
    setFormatCode(properties.numFmt);
  }, [properties.open, properties.numFmt]);

  const onSubmit = (format_code: string): void => {
    properties.onChange(format_code);
    properties.onClose();
  };
  return (
    <Dialog
      open={properties.open}
      onClose={properties.onClose}
      title={t("num_fmt.title")}
      footer={
        <Button
          variant="primary"
          startIcon={<Check />}
          onClick={() => onSubmit(formatCode)}
        >
          {t("num_fmt.save")}
        </Button>
      }
    >
      <StyledTextField
        autoFocus
        value={formatCode}
        name="format_code"
        onChange={(event) => setFormatCode(event.target.value)}
        onKeyDown={(event) => {
          event.stopPropagation();
          if (event.key === "Enter") {
            onSubmit(formatCode);
          }
        }}
        spellCheck="false"
        onClick={(event) => event.stopPropagation()}
        onFocus={(event) => event.target.select()}
        onPaste={(event) => event.stopPropagation()}
        onCopy={(event) => event.stopPropagation()}
        onCut={(event) => event.stopPropagation()}
      />
    </Dialog>
  );
};

const StyledTextField = styled(TextField)(({ theme }) => ({
  width: "100%",
  borderRadius: 4,
  overflow: "hidden",

  "& .MuiInputBase-input": {
    fontSize: 14,
    padding: 10,
    border: `1px solid ${theme.palette.grey[300]}`,
    borderRadius: 4,
    color: theme.palette.common.black,
    backgroundColor: theme.palette.common.white,
  },

  "&:hover .MuiInputBase-input": {
    border: `1px solid ${theme.palette.grey[500]}`,
  },
}));

export default FormatPicker;
