import { useTranslation } from "react-i18next";
import { Confirm } from "../Modal/Confirm";

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
    <Confirm
      open={open}
      onClose={onClose}
      onConfirm={onDelete}
      title={t("sheet_delete.title")}
      message={t("sheet_delete.message", { sheetName })}
      confirmLabel={t("sheet_delete.confirm")}
      cancelLabel={t("sheet_delete.cancel")}
      variant="destructive"
    />
  );
}

export default SheetDeleteDialog;
