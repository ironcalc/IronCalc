import { Button, Dialog, styled } from "@mui/material";
import { Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";

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

const DialogWrapper = styled("div")(({ theme }) => ({
  position: "fixed",
  top: "50%",
  left: "50%",
  transform: "translate(-50%, -50%)",
  background: theme.palette.common.white,
  display: "flex",
  flexDirection: "column",
  gap: 8,
  padding: 12,
  borderRadius: 8,
  boxShadow: `0px 1px 3px 0px ${theme.palette.common.black}1A`,
  width: 280,
  maxWidth: "calc(100% - 40px)",
  zIndex: 50,
  fontFamily: '"Inter", sans-serif',
}));

const IconWrapper = styled("div")(({ theme }) => ({
  display: "flex",
  justifyContent: "center",
  alignItems: "center",
  width: 36,
  height: 36,
  borderRadius: 4,
  backgroundColor: `${theme.palette.error.main}1A`,
  margin: "12px auto 8px auto",
  color: theme.palette.error.main,
  "& svg": {
    width: 16,
    height: 16,
  },
}));

const Title = styled("h2")(({ theme }) => ({
  margin: 0,
  fontSize: 14,
  fontWeight: 600,
  color: theme.palette.grey[900],
  textAlign: "center",
}));

const Body = styled("p")(({ theme }) => ({
  margin: 0,
  textAlign: "center",
  color: theme.palette.grey[900],
  fontSize: 12,
}));

const ButtonGroup = styled("div")({
  display: "flex",
  flexDirection: "column",
  gap: 8,
  marginTop: 8,
  width: "100%",
});

const DeleteButton = styled(Button)(({ theme }) => ({
  backgroundColor: theme.palette.error.main,
  color: theme.palette.common.white,
  textTransform: "none",
  "&:hover": {
    backgroundColor: theme.palette.error.dark,
  },
}));

const CancelButton = styled(Button)(({ theme }) => ({
  backgroundColor: theme.palette.grey[200],
  color: theme.palette.grey[700],
  textTransform: "none",
  "&:hover": {
    backgroundColor: theme.palette.grey[300],
  },
}));

export default SheetDeleteDialog;
