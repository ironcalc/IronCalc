import { Clipboard, Copy, Plus, Scissors, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Menu } from "../../Menu/Menu";
import { MenuDivider } from "../../Menu/MenuDivider";
import { MenuItem, MenuItemWithSubmenu } from "../../Menu/MenuItem";

interface CellContextMenuProps {
  open: boolean;
  onClose: () => void;
  anchorPosition: { x: number; y: number } | null;
  column: string;
  row: number;
  onInsertColumnLeft: () => void;
  onInsertColumnRight: () => void;
  onInsertRowAbove: () => void;
  onInsertRowBelow: () => void;
  onDeleteColumn: () => void;
  onDeleteRow: () => void;
}

const CellContextMenu = (properties: CellContextMenuProps) => {
  const { t } = useTranslation();
  const {
    open,
    onClose,
    anchorPosition,
    column,
    row,
    onInsertColumnLeft,
    onInsertColumnRight,
    onInsertRowAbove,
    onInsertRowBelow,
    onDeleteColumn,
    onDeleteRow,
  } = properties;

  if (!anchorPosition) {
    return null;
  }

  return (
    <Menu open={open} onClose={onClose} anchorPosition={anchorPosition}>
      <MenuItem icon={<Scissors />} onClick={() => {}}>
        {t("context_menu.cell.cut")}
      </MenuItem>
      <MenuItem icon={<Copy />} onClick={() => {}}>
        {t("context_menu.cell.copy")}
      </MenuItem>
      <MenuItem icon={<Clipboard />} onClick={() => {}}>
        {t("context_menu.cell.paste")}
      </MenuItem>

      <MenuDivider />

      <MenuItemWithSubmenu
        icon={<Plus />}
        submenu={
          <>
            <MenuItem onClick={onInsertColumnLeft}>
              {t("context_menu.cell.insert_column_left")}
            </MenuItem>
            <MenuItem onClick={onInsertColumnRight}>
              {t("context_menu.cell.insert_column_right")}
            </MenuItem>
            <MenuDivider />
            <MenuItem onClick={onInsertRowAbove}>
              {t("context_menu.cell.insert_row_above")}
            </MenuItem>
            <MenuItem onClick={onInsertRowBelow}>
              {t("context_menu.cell.insert_row_below")}
            </MenuItem>
          </>
        }
      >
        {t("context_menu.cell.insert")}
      </MenuItemWithSubmenu>

      <MenuItemWithSubmenu
        icon={<Trash2 />}
        submenu={
          <>
            <MenuItem onClick={onDeleteColumn}>
              {t("context_menu.column_header.delete_column", { column })}
            </MenuItem>
            <MenuItem onClick={onDeleteRow}>
              {t("context_menu.row_header.delete_row", { row })}
            </MenuItem>
          </>
        }
      >
        {t("context_menu.cell.delete")}
      </MenuItemWithSubmenu>
    </Menu>
  );
};

export default CellContextMenu;
