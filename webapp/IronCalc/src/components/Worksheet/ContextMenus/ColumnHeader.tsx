import {
  ArrowLeft,
  ArrowRight,
  Eye,
  EyeOff,
  MoveHorizontal,
  Plus,
  Snowflake,
  Trash2,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { Menu } from "../../Menu/Menu";
import { MenuDivider } from "../../Menu/MenuDivider";
import { MenuItem } from "../../Menu/MenuItem";

interface Range {
  rowStart: number;
  columnStart: string;
  rowEnd: number;
  columnEnd: string;
  columnCount: number;
}

interface ColumnHeaderContextMenuProps {
  open: boolean;
  onClose: () => void;
  anchorPosition: { x: number; y: number } | null;
  onInsertColumnsLeft: () => void;
  onInsertColumnsRight: () => void;
  onFreezeColumns: () => void;
  onUnfreezeColumns: () => void;
  onDeleteColumns: () => void;
  onHideColumns: () => void;
  onShowHiddenColumns: () => void;
  onMoveColumnsLeft: () => void;
  onMoveColumnsRight: () => void;
  onSetColumnWidth: () => void;
  range: Range;
  frozenColumnsCount: number;
  hiddenColumnsCount: number;
}

const ColumnHeaderContextMenu = (properties: ColumnHeaderContextMenuProps) => {
  const { t } = useTranslation();
  const {
    open,
    onClose,
    anchorPosition,
    onInsertColumnsLeft,
    onInsertColumnsRight,
    onFreezeColumns,
    onUnfreezeColumns,
    onDeleteColumns,
    onMoveColumnsLeft,
    onMoveColumnsRight,
    onSetColumnWidth,
    onHideColumns,
    onShowHiddenColumns,
    range,
    frozenColumnsCount,
    hiddenColumnsCount,
  } = properties;

  const { columnStart, columnEnd, columnCount } = range;

  if (!anchorPosition) {
    return null;
  }

  return (
    <Menu open={open} onClose={onClose} anchorPosition={anchorPosition}>
      <MenuItem icon={<Plus />} onClick={onInsertColumnsLeft}>
        {t("context_menu.column_header.insert_columns_before", {
          count: columnCount,
        })}
      </MenuItem>
      <MenuItem icon={<Plus />} onClick={onInsertColumnsRight}>
        {t("context_menu.column_header.insert_columns_after", {
          count: columnCount,
        })}
      </MenuItem>

      <MenuDivider />

      <MenuItem icon={<ArrowLeft />} onClick={onMoveColumnsLeft}>
        {t("context_menu.column_header.move_columns_left")}
      </MenuItem>
      <MenuItem icon={<ArrowRight />} onClick={onMoveColumnsRight}>
        {t("context_menu.column_header.move_columns_right")}
      </MenuItem>

      <MenuDivider />

      <MenuItem icon={<MoveHorizontal />} onClick={onSetColumnWidth}>
        {t("context_menu.column_header.set_column_width")}
      </MenuItem>

      <MenuDivider />

      <MenuItem icon={<EyeOff />} onClick={onHideColumns}>
        {columnCount === 1
          ? t("context_menu.column_header.hide_column", {
              column: columnStart,
            })
          : t("context_menu.column_header.hide_columns", {
              columnStart,
              columnEnd,
            })}
      </MenuItem>
      {hiddenColumnsCount > 0 && (
        <MenuItem icon={<Eye />} onClick={onShowHiddenColumns}>
          {t("context_menu.column_header.show_hidden_columns")}
        </MenuItem>
      )}

      <MenuDivider />

      <MenuItem icon={<Snowflake />} onClick={onFreezeColumns}>
        {t("context_menu.column_header.freeze_columns", {
          column: columnStart,
        })}
      </MenuItem>
      {frozenColumnsCount > 0 && (
        <MenuItem icon={<Snowflake />} onClick={onUnfreezeColumns}>
          {t("context_menu.column_header.unfreeze_columns")}
        </MenuItem>
      )}

      <MenuDivider />

      <MenuItem icon={<Trash2 />} destructive onClick={onDeleteColumns}>
        {columnCount === 1
          ? t("context_menu.column_header.delete_column", {
              column: columnStart,
            })
          : t("context_menu.column_header.delete_columns", {
              columnStart,
              columnEnd,
            })}
      </MenuItem>
    </Menu>
  );
};

export default ColumnHeaderContextMenu;
