import styled from "@emotion/styled";
import { Dialog, TextField } from "@mui/material";
import { Check, X } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { theme } from "../../theme";
import { Button } from "../Button/Button";
import { IconButton } from "../Button/IconButton";

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
        <StyledTextField
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
          spellCheck="false"
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

const StyledDialogTitle = styled("div")`
  display: flex;
  align-items: center;
  height: 44px;
  font-size: 14px;
  font-weight: 500;
  font-family: Inter;
  padding: 0px 12px;
  justify-content: space-between;
  border-bottom: 1px solid ${theme.palette.grey["300"]};
`;

const StyledDialogContent = styled("div")`
  font-size: 12px;
  margin: 12px;
`;

const StyledTextField = styled(TextField)`
  width: 100%;
  border-radius: 4px;
  overflow: hidden;
  & .MuiInputBase-input {
    font-size: 14px;
    padding: 10px;
    border: 1px solid ${theme.palette.grey["300"]};
    border-radius: 4px;
    color: ${theme.palette.common.black};
    background-color: ${theme.palette.common.white};
  }
  &:hover .MuiInputBase-input {
    border: 1px solid ${theme.palette.grey["500"]};
  }
`;

const DialogFooter = styled("div")`
  color: #757575;
  display: flex;
  align-items: center;
  border-top: 1px solid ${theme.palette.grey["300"]};
  font-family: Inter;
  justify-content: flex-end;
  padding: 12px;
`;

export default FormatPicker;
