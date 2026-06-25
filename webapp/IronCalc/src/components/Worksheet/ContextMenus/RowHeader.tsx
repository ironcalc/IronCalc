import {
  ArrowDown,
  ArrowUp,
  Eye,
  EyeOff,
  MoveVertical,
  Plus,
  Snowflake,
  Trash2,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { Menu } from "../../Menu/Menu";
import { MenuDivider } from "../../Menu/MenuDivider";
import { MenuItem } from "../../Menu/MenuItem";

interface RowHeaderContextMenuProps {
  open: boolean;
  onClose: () => void;
  anchorPosition: { x: number; y: number } | null;
  onInsertRowsAbove: () => void;
  onInsertRowsBelow: () => void;
  onFreezeRows: () => void;
  onUnfreezeRows: () => void;
  onDeleteRows: () => void;
  onMoveRowsUp: () => void;
  onMoveRowsDown: () => void;
  onSetRowHeight: () => void;
  onHideRows: () => void;
  onShowHiddenRows: () => void;
  hasHiddenRows: boolean;
  range: {
    rowStart: number;
    columnStart: string;
    rowEnd: number;
    columnEnd: string;
    columnCount: number;
  };
  frozenRowsCount: number;
}

const RowHeaderContextMenu = (properties: RowHeaderContextMenuProps) => {
  const { t } = useTranslation();
  const {
    open,
    onClose,
    anchorPosition,
    onInsertRowsAbove,
    onInsertRowsBelow,
    onFreezeRows,
    onUnfreezeRows,
    onDeleteRows,
    onMoveRowsUp,
    onMoveRowsDown,
    onSetRowHeight,
    onHideRows,
    onShowHiddenRows,
    hasHiddenRows,
    range,
    frozenRowsCount,
  } = properties;

  const { rowStart, rowEnd } = range;
  const rowCount = rowEnd - rowStart + 1;

  if (!anchorPosition) {
    return null;
  }

  return (
    <Menu open={open} onClose={onClose} anchorPosition={anchorPosition}>
      <MenuItem icon={<Plus />} onClick={onInsertRowsAbove}>
        {t("context_menu.row_header.insert_rows_above", { count: rowCount })}
      </MenuItem>
      <MenuItem icon={<Plus />} onClick={onInsertRowsBelow}>
        {t("context_menu.row_header.insert_rows_below", { count: rowCount })}
      </MenuItem>

      <MenuDivider />

      <MenuItem icon={<ArrowUp />} onClick={onMoveRowsUp}>
        {t("context_menu.row_header.move_rows_up")}
      </MenuItem>
      <MenuItem icon={<ArrowDown />} onClick={onMoveRowsDown}>
        {t("context_menu.row_header.move_rows_down")}
      </MenuItem>

      <MenuDivider />

      <MenuItem icon={<MoveVertical />} onClick={onSetRowHeight}>
        {t("context_menu.row_header.set_row_height")}
      </MenuItem>

      <MenuDivider />

      <MenuItem icon={<EyeOff />} onClick={onHideRows}>
        {rowCount === 1
          ? t("context_menu.row_header.hide_row", { row: rowStart })
          : t("context_menu.row_header.hide_rows", { rowStart, rowEnd })}
      </MenuItem>
      {hasHiddenRows && (
        <MenuItem icon={<Eye />} onClick={onShowHiddenRows}>
          {t("context_menu.row_header.show_hidden_rows")}
        </MenuItem>
      )}

      <MenuDivider />

      <MenuItem icon={<Snowflake />} onClick={onFreezeRows}>
        {t("context_menu.row_header.freeze_rows", { row: rowStart })}
      </MenuItem>
      {frozenRowsCount > 0 && (
        <MenuItem icon={<Snowflake />} onClick={onUnfreezeRows}>
          {t("context_menu.row_header.unfreeze_rows")}
        </MenuItem>
      )}

      <MenuDivider />

      <MenuItem icon={<Trash2 />} destructive onClick={onDeleteRows}>
        {rowCount === 1
          ? t("context_menu.row_header.delete_row", { row: rowStart })
          : t("context_menu.row_header.delete_rows", { rowStart, rowEnd })}
      </MenuItem>
    </Menu>
  );
};

export default RowHeaderContextMenu;
