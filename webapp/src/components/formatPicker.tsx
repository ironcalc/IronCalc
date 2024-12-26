import styled from "@emotion/styled";
import { Dialog, TextField } from "@mui/material";
import { Check, X } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { theme } from "../theme";

type FormatPickerProps = {
  className?: string;
  open: boolean;
  onClose: () => void;
  onExited?: () => void;
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
        <Cross
          onClick={handleClose}
          title={t("num_fmt.close")}
          tabIndex={0}
          onKeyDown={(event) => event.key === "Enter" && properties.onClose()}
        >
          <X />
        </Cross>
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
        />
      </StyledDialogContent>
      <DialogFooter>
        <StyledButton
          onClick={() => onSubmit(formatCode)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              onSubmit(formatCode);
              properties.onClose();
            }
          }}
          tabIndex={0}
        >
          <Check
            style={{ width: "16px", height: "16px", marginRight: "8px" }}
          />
          {t("num_fmt.save")}
        </StyledButton>
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

const Cross = styled("div")`
  &:hover {
    background-color: ${theme.palette.grey["50"]};
  }
  display: flex;
  border-radius: 4px;
  height: 24px;
  width: 24px;
  cursor: pointer;
  align-items: center;
  justify-content: center;
  svg {
    width: 16px;
    height: 16px;
    stroke-width: 1.5;
  }
`;

const StyledDialogContent = styled("div")`
  font-size: 12px;
  margin: 12px;
`;

const StyledTextField = styled(TextField)`
  width: 100%;
  min-width: 320px;
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

const StyledButton = styled("div")`
  cursor: pointer;
  color: #ffffff;
  background: #f2994a;
  padding: 0px 10px;
  height: 36px;
  line-height: 36px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  font-family: "Inter";
  font-size: 14px;
  &:hover {
    background: #d68742;
  }
`;

export default FormatPicker;
