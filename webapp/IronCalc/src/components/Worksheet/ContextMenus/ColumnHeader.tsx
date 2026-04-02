import {
  ArrowLeft,
  ArrowRight,
  Eye,
  EyeOff,
  Plus,
  Snowflake,
  Trash2,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { DeleteButton, StyledMenu, StyledMenuItem } from "./Common";

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
  anchorPosition: { top: number; left: number } | null;
  onInsertColumnsLeft: () => void;
  onInsertColumnsRight: () => void;
  onFreezeColumns: () => void;
  onUnfreezeColumns: () => void;
  onDeleteColumns: () => void;
  onHideColumns: () => void;
  onShowHiddenColumns: () => void;
  onMoveColumnsLeft: () => void;
  onMoveColumnsRight: () => void;
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
    onHideColumns,
    onShowHiddenColumns,
    range,
    frozenColumnsCount,
    hiddenColumnsCount,
  } = properties;

  const { columnStart, columnEnd, columnCount } = range;

  return (
    <StyledMenu
      open={open}
      onClose={onClose}
      anchorPosition={anchorPosition ?? undefined}
    >
      <StyledMenuItem onClick={onInsertColumnsLeft}>
        <Plus />
        <div className="ic-context-menu-item-name">
          {t("context_menu.column_header.insert_columns_before", {
            count: columnCount,
          })}
        </div>
      </StyledMenuItem>
      <StyledMenuItem onClick={onInsertColumnsRight}>
        <Plus />
        <div className="ic-context-menu-item-name">
          {t("context_menu.column_header.insert_columns_after", {
            count: columnCount,
          })}
        </div>
      </StyledMenuItem>

      <div className="ic-context-menu-divider" />

      <StyledMenuItem onClick={onMoveColumnsLeft}>
        <ArrowLeft />
        <div className="ic-context-menu-item-name">
          {t("context_menu.column_header.move_columns_left")}
        </div>
      </StyledMenuItem>
      <StyledMenuItem onClick={onMoveColumnsRight}>
        <ArrowRight />
        <div className="ic-context-menu-item-name">
          {t("context_menu.column_header.move_columns_right")}
        </div>
      </StyledMenuItem>

      <div className="ic-context-menu-divider" />
      <StyledMenuItem onClick={onHideColumns}>
        <EyeOff />
        <div className="ic-context-menu-item-name">
          {columnCount === 1
            ? t("context_menu.column_header.hide_column", {
                column: columnStart,
              })
            : t("context_menu.column_header.hide_columns", {
                columnStart,
                columnEnd,
              })}
        </div>
      </StyledMenuItem>
      {hiddenColumnsCount > 0 && (
        <StyledMenuItem onClick={onShowHiddenColumns}>
          <Eye />
          <div className="ic-context-menu-item-name">
            {t("context_menu.column_header.show_hidden_columns")}
          </div>
        </StyledMenuItem>
      )}
      <div className="ic-context-menu-divider" />

      <StyledMenuItem onClick={onFreezeColumns}>
        <Snowflake />
        <div className="ic-context-menu-item-name">
          {t("context_menu.column_header.freeze_columns", {
            column: columnStart,
          })}
        </div>
      </StyledMenuItem>
      {frozenColumnsCount > 0 && (
        <StyledMenuItem onClick={onUnfreezeColumns}>
          <Snowflake />
          <div className="ic-context-menu-item-name">
            {t("context_menu.column_header.unfreeze_columns")}
          </div>
        </StyledMenuItem>
      )}

      <div className="ic-context-menu-divider" />

      <DeleteButton onClick={onDeleteColumns}>
        <Trash2 />
        <div className="ic-context-menu-item-name">
          {columnCount === 1
            ? t("context_menu.column_header.delete_column", {
                column: columnStart,
              })
            : t("context_menu.column_header.delete_columns", {
                columnStart,
                columnEnd,
              })}
        </div>
      </DeleteButton>
    </StyledMenu>
  );
};

export default ColumnHeaderContextMenu;
