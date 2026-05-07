import { Confirm } from "@ironcalc/workbook";
import { useTranslation } from "react-i18next";

interface DeleteWorkbookDialogProperties {
  open: boolean;
  onClose: () => void;
  onConfirm: () => void;
  workbookName: string;
}

function DeleteWorkbookDialog({
  open,
  onClose,
  onConfirm,
  workbookName,
}: DeleteWorkbookDialogProperties) {
  const { t } = useTranslation();

  return (
    <Confirm
      open={open}
      onClose={onClose}
      onConfirm={onConfirm}
      title={t("file_bar.file_menu.delete_workbook.title")}
      message={
        <>
          <p>
            {t("file_bar.file_menu.delete_workbook.subtitle", { workbookName })}
          </p>
          <p>
            <strong>{t("file_bar.file_menu.delete_workbook.warning")}</strong>
          </p>
        </>
      }
      confirmLabel={t("file_bar.file_menu.delete_workbook.confirm_button")}
      cancelLabel={t("file_bar.file_menu.delete_workbook.cancel_button")}
      variant="destructive"
    />
  );
}

DeleteWorkbookDialog.displayName = "DeleteWorkbookDialog";

export default DeleteWorkbookDialog;
