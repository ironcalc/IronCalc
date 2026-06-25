import { useTranslation } from "react-i18next";
import { Confirm } from "../Modal";

interface SheetDeleteModalProps {
  open: boolean;
  onClose: () => void;
  onDelete: () => void;
  sheetName: string;
}

function SheetDeleteModal({
  open,
  onClose,
  onDelete,
  sheetName,
}: SheetDeleteModalProps) {
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

export default SheetDeleteModal;
