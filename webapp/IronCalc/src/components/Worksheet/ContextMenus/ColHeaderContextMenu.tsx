import { ArrowLeft, ArrowRight, Plus, Snowflake, Trash2 } from "lucide-react";
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

interface ColHeaderContextMenuProps {
  open: boolean;
  onClose: () => void;
  anchorPosition: { top: number; left: number } | null;
  onInsertColumnLeft: () => void;
  onInsertColumnRight: () => void;
  onFreezeColumns: () => void;
  onUnfreezeColumns: () => void;
  onDeleteColumn: () => void;
  onMoveColumnLeft: () => void;
  onMoveColumnRight: () => void;
  column: string;
  frozenColumnsCount: number;
}

const ColHeaderContextMenu = (properties: ColHeaderContextMenuProps) => {
  const { t } = useTranslation();
  const {
    open,
    onClose,
    anchorPosition,
    onInsertColumnLeft,
    onInsertColumnRight,
    onFreezeColumns,
    onUnfreezeColumns,
    onDeleteColumn,
    onMoveColumnLeft,
    onMoveColumnRight,
    column,
    frozenColumnsCount,
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
          onInsertColumnLeft();
        }}
        disableRipple
      >
        <Plus />
        <ItemNameStyled>
          {t("cell_context.insert_column_before")}
        </ItemNameStyled>
      </StyledMenuItem>
      <StyledMenuItem
        onClick={() => {
          onInsertColumnRight();
        }}
        disableRipple
      >
        <Plus />
        <ItemNameStyled>{t("cell_context.insert_column_after")}</ItemNameStyled>
      </StyledMenuItem>

      <MenuDivider />

      <StyledMenuItem
        onClick={() => {
          onMoveColumnLeft();
        }}
        disableRipple
      >
        <ArrowLeft />
        <ItemNameStyled>{t("cell_context.move_column_left")}</ItemNameStyled>
      </StyledMenuItem>
      <StyledMenuItem
        onClick={() => {
          onMoveColumnRight();
        }}
        disableRipple
      >
        <ArrowRight />
        <ItemNameStyled>{t("cell_context.move_column_right")}</ItemNameStyled>
      </StyledMenuItem>

      <MenuDivider />

      <StyledMenuItem
        onClick={() => {
          onFreezeColumns();
        }}
        disableRipple
      >
        <Snowflake />
        <ItemNameStyled>
          {t("cell_context.freeze_columns", { column })}
        </ItemNameStyled>
      </StyledMenuItem>
      {frozenColumnsCount > 0 && (
        <StyledMenuItem
          onClick={() => {
            onUnfreezeColumns();
          }}
          disableRipple
        >
          <Snowflake />
          <ItemNameStyled>{t("cell_context.unfreeze_columns")}</ItemNameStyled>
        </StyledMenuItem>
      )}

      <MenuDivider />

      <DeleteButton onClick={onDeleteColumn} disableRipple>
        <Trash2 />
        <ItemNameStyled style={{ color: red_color }}>
          {t("cell_context.delete_column", { column })}
        </ItemNameStyled>
      </DeleteButton>
    </StyledMenu>
  );
};

export default ColHeaderContextMenu;
