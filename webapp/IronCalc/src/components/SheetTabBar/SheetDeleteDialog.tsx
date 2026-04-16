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

  const handleDelete = (): void => {
    onDelete();
    onClose();
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      onConfirm={handleDelete}
      title={t("sheet_delete.title")}
      showHeader
      footer={
        <>
          <Button size="md" variant="secondary" onClick={onClose}>
            {t("sheet_delete.cancel")}
          </Button>
          <Button size="md" variant="destructive" onClick={handleDelete}>
            {t("sheet_delete.confirm")}
          </Button>
        </>
      }
    >
      <p>{t("sheet_delete.message", { sheetName })}</p>
    </Dialog>
  );
}

export default SheetDeleteDialog;
