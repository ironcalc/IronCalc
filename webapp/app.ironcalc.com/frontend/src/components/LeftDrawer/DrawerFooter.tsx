import { Confirm } from "@ironcalc/workbook";
import { BookOpen, Trash2 } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../../../../IronCalc/src/components/Button/Button";

interface DrawerFooterProps {
  checkedCount: number;
  onDeleteChecked: () => void;
  onCancelChecked: () => void;
}

function DrawerFooter({
  checkedCount,
  onDeleteChecked,
  onCancelChecked,
}: DrawerFooterProps) {
  const { t } = useTranslation();
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);

  if (checkedCount > 0) {
    return (
      <>
        <div className="app-ic-drawer-footer app-ic-drawer-footer--selection">
          <Button
            variant="destructive"
            startIcon={<Trash2 />}
            onClick={() => setIsDeleteDialogOpen(true)}
          >
            {t("left_drawer.delete_workbooks", { count: checkedCount })}
          </Button>
          <Button variant="secondary" onClick={onCancelChecked}>
            {t("left_drawer.cancel")}
          </Button>
        </div>
        <Confirm
          open={isDeleteDialogOpen}
          onClose={() => setIsDeleteDialogOpen(false)}
          onConfirm={() => {
            setIsDeleteDialogOpen(false);
            onDeleteChecked();
          }}
          title={t("file_bar.file_menu.delete_workbook.title")}
          message={t("left_drawer.bulk_delete_message", {
            count: checkedCount,
          })}
          confirmLabel={t("file_bar.file_menu.delete_workbook.confirm_button")}
          cancelLabel={t("file_bar.file_menu.delete_workbook.cancel_button")}
          variant="destructive"
        />
      </>
    );
  }

  return (
    <div className="app-ic-drawer-footer">
      <a
        className="app-ic-drawer-footer-link"
        href="https://docs.ironcalc.com/"
        target="_blank"
        rel="noopener noreferrer"
      >
        <BookOpen />
        {t("left_drawer.documentation")}
      </a>
    </div>
  );
}

export default DrawerFooter;
