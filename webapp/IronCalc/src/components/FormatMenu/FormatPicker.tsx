import { Dialog, styled } from "@mui/material";
import { Check, X } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../Button/Button";
import { IconButton } from "../Button/IconButton";
import { Input } from "../Input/Input";

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

  const handleClose = () => {
    properties.onClose();
  };

  const onSubmit = (format_code: string): void => {
    properties.onChange(format_code);
    properties.onClose();
  };
  return (
    <Dialog
      open={properties.open}
      onClose={properties.onClose}
      PaperProps={{
        style: { minWidth: "280px" },
      }}
    >
      <StyledDialogTitle>
        {t("num_fmt.title")}
        <IconButton
          icon={<X />}
          onClick={handleClose}
          title={t("num_fmt.close")}
          aria-label={t("num_fmt.close")}
        />
      </StyledDialogTitle>

      <StyledDialogContent>
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
            }
          }}
          spellCheck={false}
          onClick={(event) => event.stopPropagation()}
          onFocus={(event) => event.target.select()}
          onPaste={(event) => event.stopPropagation()}
          onCopy={(event) => event.stopPropagation()}
          onCut={(event) => event.stopPropagation()}
        />
      </StyledDialogContent>
      <DialogFooter>
        <Button
          size="md"
          startIcon={<Check />}
          onClick={() => onSubmit(formatCode)}
        >
          {t("num_fmt.save")}
        </Button>
      </DialogFooter>
    </Dialog>
  );
};

const StyledDialogTitle = styled("div")(({ theme }) => ({
  display: "flex",
  alignItems: "center",
  height: 44,
  fontSize: 14,
  fontWeight: 500,
  fontFamily: "Inter",
  padding: "0px 12px",
  justifyContent: "space-between",
  borderBottom: `1px solid ${theme.palette.grey[300]}`,
}));

const StyledDialogContent = styled("div")({
  fontSize: 12,
  margin: 12,
});

const DialogFooter = styled("div")(({ theme }) => ({
  color: "#757575",
  display: "flex",
  alignItems: "center",
  borderTop: `1px solid ${theme.palette.grey[300]}`,
  fontFamily: "Inter",
  justifyContent: "flex-end",
  padding: 12,
}));

export default FormatPicker;
