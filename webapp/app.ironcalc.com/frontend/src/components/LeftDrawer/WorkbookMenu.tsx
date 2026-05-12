import { Menu, MenuDivider, MenuItem } from "@ironcalc/workbook";
import {
  Copy,
  EllipsisVertical,
  FileDown,
  Pin,
  PinOff,
  Trash2,
} from "lucide-react";
import { useTranslation } from "react-i18next";

interface WorkbookMenuProps {
  isPinned: boolean;
  onOpen: () => void;
  onDownload: () => void;
  onPinToggle: () => void;
  onDuplicate: () => void;
  onDelete: () => void;
}

function WorkbookMenu({
  isPinned,
  onOpen,
  onDownload,
  onPinToggle,
  onDuplicate,
  onDelete,
}: WorkbookMenuProps) {
  const { t } = useTranslation();

  return (
    <Menu
      trigger={
        <button
          type="button"
          className="app-ic-drawer-workbook-ellipsis"
          onClick={onOpen}
          onMouseDown={(e) => e.stopPropagation()}
        >
          <EllipsisVertical />
        </button>
      }
    >
      <MenuItem icon={<FileDown />} onClick={onDownload}>
        {t("left_drawer.workbook_menu.download")}
      </MenuItem>
      <MenuItem icon={isPinned ? <PinOff /> : <Pin />} onClick={onPinToggle}>
        {isPinned
          ? t("left_drawer.workbook_menu.unpin")
          : t("left_drawer.workbook_menu.pin")}
      </MenuItem>
      <MenuItem icon={<Copy />} onClick={onDuplicate}>
        {t("left_drawer.workbook_menu.duplicate")}
      </MenuItem>
      <MenuDivider />
      <MenuItem destructive icon={<Trash2 />} onClick={onDelete}>
        {t("left_drawer.workbook_menu.delete")}
      </MenuItem>
    </Menu>
  );
}

export default WorkbookMenu;
