import {
  ArrowDown,
  ArrowUp,
  Eye,
  EyeOff,
  Plus,
  Snowflake,
  Trash2,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { DeleteButton, StyledMenu, StyledMenuItem } from "./Common";

interface RowHeaderContextMenuProps {
  open: boolean;
  onClose: () => void;
  anchorPosition: { top: number; left: number } | null;
  onInsertRowsAbove: () => void;
  onInsertRowsBelow: () => void;
  onFreezeRows: () => void;
  onUnfreezeRows: () => void;
  onDeleteRows: () => void;
  onMoveRowsUp: () => void;
  onMoveRowsDown: () => void;
  onHideRows: () => void;
  onShowHiddenRows: () => void;
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
    onHideRows,
    onShowHiddenRows,
    range,
    frozenRowsCount,
  } = properties;

  const { rowStart, rowEnd } = range;
  const rowCount = rowEnd - rowStart + 1;

  return (
    <StyledMenu
      open={open}
      onClose={onClose}
      anchorPosition={anchorPosition ?? undefined}
    >
      <StyledMenuItem onClick={onInsertRowsAbove}>
        <Plus />
        <div className="ic-context-menu-item-name">
          {t("context_menu.row_header.insert_rows_above", { count: rowCount })}
        </div>
      </StyledMenuItem>
      <StyledMenuItem onClick={onInsertRowsBelow}>
        <Plus />
        <div className="ic-context-menu-item-name">
          {t("context_menu.row_header.insert_rows_below", { count: rowCount })}
        </div>
      </StyledMenuItem>

      <hr aria-orientation="horizontal" />

      <StyledMenuItem onClick={onMoveRowsUp}>
        <ArrowUp />
        <div className="ic-context-menu-item-name">
          {t("context_menu.row_header.move_rows_up")}
        </div>
      </StyledMenuItem>
      <StyledMenuItem onClick={onMoveRowsDown}>
        <ArrowDown />
        <div className="ic-context-menu-item-name">
          {t("context_menu.row_header.move_rows_down")}
        </div>
      </StyledMenuItem>

      <hr aria-orientation="horizontal" />

      <StyledMenuItem onClick={onHideRows}>
        <EyeOff />
        <div className="ic-context-menu-item-name">
          {rowCount === 1
            ? t("context_menu.row_header.hide_row", { row: rowStart })
            : t("context_menu.row_header.hide_rows", { rowStart, rowEnd })}
        </div>
      </StyledMenuItem>
      <StyledMenuItem onClick={onShowHiddenRows}>
        <Eye />
        <div className="ic-context-menu-item-name">
          {t("context_menu.row_header.show_hidden_rows")}
        </div>
      </StyledMenuItem>

      <hr aria-orientation="horizontal" />

      <StyledMenuItem onClick={onFreezeRows}>
        <Snowflake />
        <div className="ic-context-menu-item-name">
          {t("context_menu.row_header.freeze_rows", { row: rowStart })}
        </div>
      </StyledMenuItem>
      {frozenRowsCount > 0 && (
        <StyledMenuItem onClick={onUnfreezeRows}>
          <Snowflake />
          <div className="ic-context-menu-item-name">
            {t("context_menu.row_header.unfreeze_rows")}
          </div>
        </StyledMenuItem>
      )}

      <hr aria-orientation="horizontal" />

      <DeleteButton onClick={onDeleteRows}>
        <Trash2 />
        <div className="ic-context-menu-item-name">
          {rowCount === 1
            ? t("context_menu.row_header.delete_row", { row: rowStart })
            : t("context_menu.row_header.delete_rows", { rowStart, rowEnd })}
        </div>
      </DeleteButton>
    </StyledMenu>
  );
};

export default RowHeaderContextMenu;
