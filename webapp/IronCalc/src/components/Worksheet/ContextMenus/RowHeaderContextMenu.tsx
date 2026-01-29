import { ArrowDown, ArrowUp, Plus, Snowflake, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { theme } from "../../../theme";
import {
  DeleteButton,
  ItemNameStyled,
  MenuDivider,
  StyledMenu,
  StyledMenuItem,
} from "./CellContextMenu";

const red_color = theme.palette.error.main;

interface RowHeaderContextMenuProps {
  open: boolean;
  onClose: () => void;
  anchorPosition: { top: number; left: number } | null;
  onInsertRowAbove: () => void;
  onInsertRowBelow: () => void;
  onFreezeRows: () => void;
  onUnfreezeRows: () => void;
  onDeleteRow: () => void;
  onMoveRowUp: () => void;
  onMoveRowDown: () => void;
  row: number;
  frozenRowsCount: number;
}

const RowHeaderContextMenu = (properties: RowHeaderContextMenuProps) => {
  const { t } = useTranslation();
  const {
    open,
    onClose,
    anchorPosition,
    onInsertRowAbove,
    onInsertRowBelow,
    onFreezeRows,
    onUnfreezeRows,
    onDeleteRow,
    onMoveRowUp,
    onMoveRowDown,
    row,
    frozenRowsCount,
  } = properties;

  return (
    <StyledMenu
      open={open}
      onClose={onClose}
      transitionDuration={0}
      autoFocus={false}
      anchorReference="anchorPosition"
      anchorPosition={anchorPosition ?? undefined}
    >
      <StyledMenuItem
        onClick={() => {
          onInsertRowAbove();
        }}
        disableRipple
      >
        <Plus />
        <ItemNameStyled>{t("cell_context.insert_row_above")}</ItemNameStyled>
      </StyledMenuItem>
      <StyledMenuItem
        onClick={() => {
          onInsertRowBelow();
        }}
        disableRipple
      >
        <Plus />
        <ItemNameStyled>{t("cell_context.insert_row_below")}</ItemNameStyled>
      </StyledMenuItem>

      <MenuDivider />

      <StyledMenuItem
        onClick={() => {
          onMoveRowUp();
        }}
        disableRipple
      >
        <ArrowUp />
        <ItemNameStyled>{t("cell_context.move_row_up")}</ItemNameStyled>
      </StyledMenuItem>
      <StyledMenuItem
        onClick={() => {
          onMoveRowDown();
        }}
        disableRipple
      >
        <ArrowDown />
        <ItemNameStyled>{t("cell_context.move_row_down")}</ItemNameStyled>
      </StyledMenuItem>

      <MenuDivider />

      <StyledMenuItem
        onClick={() => {
          onFreezeRows();
        }}
        disableRipple
      >
        <Snowflake />
        <ItemNameStyled>
          {t("cell_context.freeze_rows", { row })}
        </ItemNameStyled>
      </StyledMenuItem>
      {frozenRowsCount > 0 && (
        <StyledMenuItem
          onClick={() => {
            onUnfreezeRows();
          }}
          disableRipple
        >
          <Snowflake />
          <ItemNameStyled>{t("cell_context.unfreeze_rows")}</ItemNameStyled>
        </StyledMenuItem>
      )}

      <MenuDivider />

      <DeleteButton onClick={onDeleteRow} disableRipple>
        <Trash2 />
        <ItemNameStyled style={{ color: red_color }}>
          {t("cell_context.delete_row", { row })}
        </ItemNameStyled>
      </DeleteButton>
    </StyledMenu>
  );
};

export default RowHeaderContextMenu;
