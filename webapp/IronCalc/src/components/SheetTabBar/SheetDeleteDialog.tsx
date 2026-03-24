import { Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "../Button/Button";
import { Dialog } from "../Dialog/Dialog";

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
    <Dialog
      open={open}
      onClose={onClose}
      title={t("sheet_delete.title")}
      showCloseButton={false}
      footer={
        <>
          <Button variant="secondary" onClick={onClose}>
            {t("sheet_delete.cancel")}
          </Button>
          <Button
            variant="destructive"
            startIcon={<Trash2 />}
            onClick={onDelete}
            autoFocus
          >
            {t("sheet_delete.confirm")}
          </Button>
        </>
      }
    >
      {t("sheet_delete.message", {
        sheetName,
      })}
    </Dialog>
  );
}

export default SheetDeleteDialog;
