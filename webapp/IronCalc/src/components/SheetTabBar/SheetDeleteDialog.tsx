import styled from "@emotion/styled";
import { Button, Dialog } from "@mui/material";
import { Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { theme } from "../../theme";

interface SheetDeleteDialogProps {
  open: boolean;
  onClose: () => void;
  onDelete: () => void;
  sheetName: string;
}

function SheetDeleteDialog({
  open,
  onClose,
  onDelete,
  sheetName,
}: SheetDeleteDialogProps) {
  const { t } = useTranslation();

  return (
    <Dialog open={open} onClose={onClose}>
      <DialogWrapper>
        <IconWrapper>
          <Trash2 />
        </IconWrapper>
        <Title>{t("sheet_delete.title")}</Title>
        <Body>
          {t("sheet_delete.message", {
            sheetName,
          })}
        </Body>
        <ButtonGroup>
          <DeleteButton onClick={onDelete} autoFocus>
            {t("sheet_delete.confirm")}
          </DeleteButton>
          <CancelButton onClick={onClose}>
            {t("sheet_delete.cancel")}
          </CancelButton>
        </ButtonGroup>
      </DialogWrapper>
    </Dialog>
  );
}

const DialogWrapper = styled.div`
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: white;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border-radius: 8px;
  box-shadow: 0px 1px 3px 0px ${theme.palette.common.black}1A;
  width: 280px;
  max-width: calc(100% - 40px);
  z-index: 50;
  font-family: "Inter", sans-serif;
`;

const IconWrapper = styled.div`
  display: flex;
  justify-content: center;
  align-items: center;
  width: 36px;
  height: 36px;
  border-radius: 4px;
  background-color: ${theme.palette.error.main}1A;
  margin: 12px auto 8px auto;
  color: ${theme.palette.error.main};
  svg {
    width: 16px;
    height: 16px;
  }
`;

const Title = styled.h2`
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: ${theme.palette.grey["900"]};
  text-align: center;
`;

const Body = styled.p`
  margin: 0;
  text-align: center;
  color: ${theme.palette.grey["900"]};
  font-size: 12px;
`;

const ButtonGroup = styled.div`
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 8px;
  width: 100%;
`;

const StyledButton = styled.button`
  cursor: pointer;
  color: ${theme.palette.common.white};
  background-color: ${theme.palette.primary.main};
  padding: 0px 10px;
  height: 36px;
  border-radius: 4px;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  text-overflow: ellipsis;
  transition: background-color 150ms;
  text-transform: none;
  &:hover {
    background-color: ${theme.palette.primary.dark};
  }
`;

const DeleteButton = styled(Button)`
  background-color: ${theme.palette.error.main};
  color: ${theme.palette.common.white};
  text-transform: none;
  &:hover {
    background-color: ${theme.palette.error.dark};
  }
`;

const CancelButton = styled(Button)`
  background-color: ${theme.palette.grey["200"]};
  color: ${theme.palette.grey["700"]};
  text-transform: none;
  &:hover {
    background-color: ${theme.palette.grey["300"]};
  }
`;

export default SheetDeleteDialog;
