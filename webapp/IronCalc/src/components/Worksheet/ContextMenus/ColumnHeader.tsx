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
import {
  DeleteButton,
  ItemNameStyled,
  MenuDivider,
  StyledMenu,
  StyledMenuItem,
} from "./Common";

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
      transitionDuration={0}
      autoFocus={false}
      anchorReference="anchorPosition"
      anchorPosition={anchorPosition ?? undefined}
    >
      <StyledMenuItem onClick={onInsertColumnsLeft}>
        <Plus />
        <ItemNameStyled>
          {t("context_menu.column_header.insert_columns_before", {
            count: columnCount,
          })}
        </ItemNameStyled>
      </StyledMenuItem>
      <StyledMenuItem onClick={onInsertColumnsRight}>
        <Plus />
        <ItemNameStyled>
          {t("context_menu.column_header.insert_columns_after", {
            count: columnCount,
          })}
        </ItemNameStyled>
      </StyledMenuItem>

      <MenuDivider />

      <StyledMenuItem onClick={onMoveColumnsLeft}>
        <ArrowLeft />
        <ItemNameStyled>
          {t("context_menu.column_header.move_columns_left")}
        </ItemNameStyled>
      </StyledMenuItem>
      <StyledMenuItem onClick={onMoveColumnsRight}>
        <ArrowRight />
        <ItemNameStyled>
          {t("context_menu.column_header.move_columns_right")}
        </ItemNameStyled>
      </StyledMenuItem>

      <MenuDivider />
      <StyledMenuItem onClick={onHideColumns}>
        <EyeOff />
        <ItemNameStyled>
          {columnCount === 1
            ? t("context_menu.column_header.hide_column", {
                column: columnStart,
              })
            : t("context_menu.column_header.hide_columns", {
                columnStart,
                columnEnd,
              })}
        </ItemNameStyled>
      </StyledMenuItem>
      {hiddenColumnsCount > 0 && (
        <StyledMenuItem onClick={onShowHiddenColumns}>
          <Eye />
          <ItemNameStyled>
            {t("context_menu.column_header.show_hidden_columns")}
          </ItemNameStyled>
        </StyledMenuItem>
      )}
      <MenuDivider />

      <StyledMenuItem onClick={onFreezeColumns}>
        <Snowflake />
        <ItemNameStyled>
          {t("context_menu.column_header.freeze_columns", {
            column: columnStart,
          })}
        </ItemNameStyled>
      </StyledMenuItem>
      {frozenColumnsCount > 0 && (
        <StyledMenuItem onClick={onUnfreezeColumns}>
          <Snowflake />
          <ItemNameStyled>
            {t("context_menu.column_header.unfreeze_columns")}
          </ItemNameStyled>
        </StyledMenuItem>
      )}

      <MenuDivider />

      <DeleteButton onClick={onDeleteColumns}>
        <Trash2 />
        <ItemNameStyled>
          {columnCount === 1
            ? t("context_menu.column_header.delete_column", {
                column: columnStart,
              })
            : t("context_menu.column_header.delete_columns", {
                columnStart,
                columnEnd,
              })}
        </ItemNameStyled>
      </DeleteButton>
    </StyledMenu>
  );
};

export default ColumnHeaderContextMenu;
