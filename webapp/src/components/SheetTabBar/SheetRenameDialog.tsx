import { Dialog, TextField, styled } from "@mui/material";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { theme } from "../../theme";

interface SheetRenameDialogProps {
  open: boolean;
  onClose: () => void;
  onNameChanged: (name: string) => void;
  defaultName: string;
}

const SheetRenameDialog = (properties: SheetRenameDialogProps) => {
  const { t } = useTranslation();
  const [name, setName] = useState(properties.defaultName);
  const handleClose = () => {
    properties.onClose();
  };
  return (
    <Dialog open={properties.open} onClose={properties.onClose}>
      <StyledDialogTitle>
        {t("sheet_rename.title")}
        <Cross onClick={handleClose} onKeyDown={() => {}}>
          <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <title>Close</title>
            <path
              d="M12 4.5L4 12.5"
              stroke="#333333"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
            <path
              d="M4 4.5L12 12.5"
              stroke="#333333"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </Cross>
      </StyledDialogTitle>
      <StyledDialogContent>
        <StyledTextField
          autoFocus
          defaultValue={properties.defaultName}
          onClick={(event) => event.stopPropagation()}
          onKeyDown={(event) => {
            event.stopPropagation();
            if (event.key === "Enter") {
              properties.onNameChanged(name);
              properties.onClose();
            }
          }}
          onChange={(event) => {
            setName(event.target.value);
          }}
          spellCheck="false"
          onPaste={(event) => event.stopPropagation()}
        />
      </StyledDialogContent>
      <DialogFooter>
        <StyledButton
          onClick={() => {
            properties.onNameChanged(name);
          }}
        >
          {t("sheet_rename.rename")}
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
    background-color: ${theme.palette.grey["100"]};
  }
  display: flex;
  border-radius: 4px;
  height: 24px;
  width: 24px;
  cursor: pointer;
  align-items: center;
  justify-content: center;
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

export default SheetRenameDialog;
