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
import {
  DeleteButton,
  ItemNameStyled,
  MenuDivider,
  StyledMenu,
  StyledMenuItem,
} from "./Common";

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
      transitionDuration={0}
      autoFocus={false}
      anchorReference="anchorPosition"
      anchorPosition={anchorPosition ?? undefined}
    >
      <StyledMenuItem onClick={onInsertRowsAbove}>
        <Plus />
        <ItemNameStyled>
          {t("context_menu.row_header.insert_rows_above", { count: rowCount })}
        </ItemNameStyled>
      </StyledMenuItem>
      <StyledMenuItem onClick={onInsertRowsBelow}>
        <Plus />
        <ItemNameStyled>
          {t("context_menu.row_header.insert_rows_below", { count: rowCount })}
        </ItemNameStyled>
      </StyledMenuItem>

      <MenuDivider />

      <StyledMenuItem onClick={onMoveRowsUp}>
        <ArrowUp />
        <ItemNameStyled>
          {t("context_menu.row_header.move_rows_up")}
        </ItemNameStyled>
      </StyledMenuItem>
      <StyledMenuItem onClick={onMoveRowsDown}>
        <ArrowDown />
        <ItemNameStyled>
          {t("context_menu.row_header.move_rows_down")}
        </ItemNameStyled>
      </StyledMenuItem>

      <MenuDivider />

      <StyledMenuItem onClick={onHideRows}>
        <EyeOff />
        <ItemNameStyled>
          {rowCount === 1
            ? t("context_menu.row_header.hide_row", { row: rowStart })
            : t("context_menu.row_header.hide_rows", { rowStart, rowEnd })}
        </ItemNameStyled>
      </StyledMenuItem>
      <StyledMenuItem onClick={onShowHiddenRows}>
        <Eye />
        <ItemNameStyled>
          {t("context_menu.row_header.show_hidden_rows")}
        </ItemNameStyled>
      </StyledMenuItem>

      <MenuDivider />

      <StyledMenuItem onClick={onFreezeRows}>
        <Snowflake />
        <ItemNameStyled>
          {t("context_menu.row_header.freeze_rows", { row: rowStart })}
        </ItemNameStyled>
      </StyledMenuItem>
      {frozenRowsCount > 0 && (
        <StyledMenuItem onClick={onUnfreezeRows}>
          <Snowflake />
          <ItemNameStyled>
            {t("context_menu.row_header.unfreeze_rows")}
          </ItemNameStyled>
        </StyledMenuItem>
      )}

      <MenuDivider />

      <DeleteButton onClick={onDeleteRows}>
        <Trash2 />
        <ItemNameStyled>
          {rowCount === 1
            ? t("context_menu.row_header.delete_row", { row: rowStart })
            : t("context_menu.row_header.delete_rows", { rowStart, rowEnd })}
        </ItemNameStyled>
      </DeleteButton>
    </StyledMenu>
  );
};

export default RowHeaderContextMenu;
