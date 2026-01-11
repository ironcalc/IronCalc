import { Menu, MenuItem, styled } from "@mui/material";
import {
  ArrowLeftRight,
  ArrowUpDown,
  BetweenHorizontalStart,
  BetweenVerticalStart,
  ChevronRight,
  Snowflake,
  Trash2,
} from "lucide-react";
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { theme } from "../../../theme";

const red_color = theme.palette.error.main;

interface CellContextMenuProps {
  open: boolean;
  onClose: () => void;
  anchorPosition: { top: number; left: number } | null;
  onInsertRowAbove: () => void;
  onInsertRowBelow: () => void;
  onInsertColumnLeft: () => void;
  onInsertColumnRight: () => void;
  onFreezeColumns: () => void;
  onFreezeRows: () => void;
  onUnfreezeColumns: () => void;
  onUnfreezeRows: () => void;
  onDeleteRow: () => void;
  onDeleteColumn: () => void;
  onMoveColumnLeft: () => void;
  onMoveColumnRight: () => void;
  onMoveRowUp: () => void;
  onMoveRowDown: () => void;
  row: number;
  column: string;
}

const CellContextMenu = (properties: CellContextMenuProps) => {
  const { t } = useTranslation();
  const {
    open,
    onClose,
    anchorPosition,
    onInsertRowAbove,
    onInsertRowBelow,
    onInsertColumnLeft,
    onInsertColumnRight,
    onFreezeColumns,
    onFreezeRows,
    onUnfreezeColumns,
    onUnfreezeRows,
    onDeleteRow,
    onDeleteColumn,
    onMoveColumnLeft,
    onMoveColumnRight,
    onMoveRowUp,
    onMoveRowDown,
    row,
    column,
  } = properties;
  const [freezeMenuOpen, setFreezeMenuOpen] = useState(false);
  const freezeRef = useRef(null);

  const [insertRowMenuOpen, setInsertRowMenuOpen] = useState(false);
  const insertRowRef = useRef(null);

  const [insertColumnMenuOpen, setInsertColumnMenuOpen] = useState(false);
  const insertColumnRef = useRef(null);

  const [moveRowMenuOpen, setMoveRowMenuOpen] = useState(false);
  const moveRowRef = useRef(null);

  const [moveColumnMenuOpen, setMoveColumnMenuOpen] = useState(false);
  const moveColumnRef = useRef(null);

  return (
    <>
      <StyledMenu
        open={open}
        onClose={onClose}
        transitionDuration={0}
        anchorReference="anchorPosition"
        anchorPosition={anchorPosition ?? undefined}
      >
        <StyledMenuItem
          ref={insertColumnRef}
          disableRipple
          onClick={() => setInsertColumnMenuOpen(true)}
        >
          <BetweenVerticalStart />
          <ItemNameStyled>{t("cell_context.insert_column")}</ItemNameStyled>
          <ChevronRightStyled />
        </StyledMenuItem>
        <StyledMenuItem
          ref={insertRowRef}
          disableRipple
          onClick={() => setInsertRowMenuOpen(true)}
        >
          <BetweenHorizontalStart />
          <ItemNameStyled>{t("cell_context.insert_row")}</ItemNameStyled>
          <ChevronRightStyled />
        </StyledMenuItem>
        <MenuDivider />
        <StyledMenuItem
          ref={moveRowRef}
          disableRipple
          onClick={() => setMoveRowMenuOpen(true)}
        >
          <ArrowUpDown />
          <ItemNameStyled>{t("cell_context.move_row")}</ItemNameStyled>
          <ChevronRightStyled />
        </StyledMenuItem>
        <StyledMenuItem
          ref={moveColumnRef}
          disableRipple
          onClick={() => setMoveColumnMenuOpen(true)}
        >
          <ArrowLeftRight />
          <ItemNameStyled>{t("cell_context.move_column")}</ItemNameStyled>
          <ChevronRightStyled />
        </StyledMenuItem>
        <MenuDivider />
        <StyledMenuItem
          ref={freezeRef}
          disableRipple
          onClick={() => setFreezeMenuOpen(true)}
        >
          <Snowflake />
          <ItemNameStyled>{t("cell_context.freeze")}</ItemNameStyled>
          <ChevronRightStyled />
        </StyledMenuItem>
        <MenuDivider />
        <DeleteButton disableRipple onClick={onDeleteRow}>
          <Trash2 />
          <ItemNameStyled style={{ color: red_color }}>
            {t("cell_context.delete_row", { row })}
          </ItemNameStyled>
        </DeleteButton>
        <DeleteButton disableRipple onClick={onDeleteColumn}>
          <Trash2 />
          <ItemNameStyled style={{ color: red_color }}>
            {t("cell_context.delete_column", { column })}
          </ItemNameStyled>
        </DeleteButton>
      </StyledMenu>
      <StyledMenu
        open={insertRowMenuOpen}
        onClose={() => setInsertRowMenuOpen(false)}
        transitionDuration={0}
        anchorEl={insertRowRef.current}
        anchorOrigin={{
          vertical: "top",
          horizontal: "right",
        }}
      >
        <StyledMenuItem
          disableRipple
          onClick={() => {
            setInsertRowMenuOpen(false);
            onInsertRowAbove();
          }}
        >
          <ItemNameStyled>{t("cell_context.insert_row_above")}</ItemNameStyled>
        </StyledMenuItem>
        <StyledMenuItem
          disableRipple
          onClick={() => {
            setInsertRowMenuOpen(false);
            onInsertRowBelow();
          }}
        >
          <ItemNameStyled>{t("cell_context.insert_row_below")}</ItemNameStyled>
        </StyledMenuItem>
      </StyledMenu>
      <StyledMenu
        open={insertColumnMenuOpen}
        onClose={() => setInsertColumnMenuOpen(false)}
        transitionDuration={0}
        anchorEl={insertColumnRef.current}
        anchorOrigin={{
          vertical: "top",
          horizontal: "right",
        }}
      >
        <StyledMenuItem
          disableRipple
          onClick={() => {
            setInsertColumnMenuOpen(false);
            onInsertColumnLeft();
          }}
        >
          <ItemNameStyled>
            {t("cell_context.insert_column_before")}
          </ItemNameStyled>
        </StyledMenuItem>
        <StyledMenuItem
          disableRipple
          onClick={() => {
            setInsertColumnMenuOpen(false);
            onInsertColumnRight();
          }}
        >
          <ItemNameStyled>
            {t("cell_context.insert_column_after")}
          </ItemNameStyled>
        </StyledMenuItem>
      </StyledMenu>
      <StyledMenu
        open={moveRowMenuOpen}
        onClose={() => setMoveRowMenuOpen(false)}
        transitionDuration={0}
        anchorEl={moveRowRef.current}
        anchorOrigin={{
          vertical: "top",
          horizontal: "right",
        }}
      >
        <StyledMenuItem
          disableRipple
          onClick={() => {
            onMoveRowUp();
            setMoveRowMenuOpen(false);
          }}
        >
          <ItemNameStyled>{t("cell_context.move_row_up")}</ItemNameStyled>
        </StyledMenuItem>
        <StyledMenuItem
          disableRipple
          onClick={() => {
            onMoveRowDown();
            setMoveRowMenuOpen(false);
          }}
        >
          <ItemNameStyled>{t("cell_context.move_row_down")}</ItemNameStyled>
        </StyledMenuItem>
      </StyledMenu>
      <StyledMenu
        open={moveColumnMenuOpen}
        onClose={() => setMoveColumnMenuOpen(false)}
        transitionDuration={0}
        anchorEl={moveColumnRef.current}
        anchorOrigin={{
          vertical: "top",
          horizontal: "right",
        }}
      >
        <StyledMenuItem
          disableRipple
          onClick={() => {
            onMoveColumnLeft();
            setMoveColumnMenuOpen(false);
          }}
        >
          <ItemNameStyled>{t("cell_context.move_column_left")}</ItemNameStyled>
        </StyledMenuItem>
        <StyledMenuItem
          disableRipple
          onClick={() => {
            onMoveColumnRight();
            setMoveColumnMenuOpen(false);
          }}
        >
          <ItemNameStyled>{t("cell_context.move_column_right")}</ItemNameStyled>
        </StyledMenuItem>
      </StyledMenu>
      <StyledMenu
        open={freezeMenuOpen}
        onClose={() => setFreezeMenuOpen(false)}
        transitionDuration={0}
        anchorEl={freezeRef.current}
        anchorOrigin={{
          vertical: "top",
          horizontal: "right",
        }}
      >
        <StyledMenuItem
          disableRipple
          onClick={() => {
            onFreezeColumns();
            setFreezeMenuOpen(false);
          }}
        >
          <ItemNameStyled>
            {t("cell_context.freeze_columns", { column })}
          </ItemNameStyled>
        </StyledMenuItem>
        <StyledMenuItem
          disableRipple
          onClick={() => {
            onFreezeRows();
            setFreezeMenuOpen(false);
          }}
        >
          <ItemNameStyled>
            {t("cell_context.freeze_rows", { row })}
          </ItemNameStyled>
        </StyledMenuItem>
        <MenuDivider />
        <StyledMenuItem
          disableRipple
          onClick={() => {
            onUnfreezeColumns();
            setFreezeMenuOpen(false);
          }}
        >
          <ItemNameStyled>{t("cell_context.unfreeze_columns")}</ItemNameStyled>
        </StyledMenuItem>
        <StyledMenuItem
          disableRipple
          onClick={() => {
            onUnfreezeRows();
            setFreezeMenuOpen(false);
          }}
        >
          <ItemNameStyled>{t("cell_context.unfreeze_rows")}</ItemNameStyled>
        </StyledMenuItem>
      </StyledMenu>
    </>
  );
};

export const StyledMenu = styled(Menu)({
  "& .MuiPaper-root": {
    borderRadius: 8,
    paddingTop: 4,
    paddingBottom: 4,
  },
  "& .MuiList-padding": {
    padding: 0,
  },
});

export const StyledMenuItem = styled(MenuItem)`
  display: flex;
  justify-content: flex-start;
  font-size: 12px;
  width: calc(100% - 8px);
  min-width: 172px;
  margin: 0px 4px;
  border-radius: 4px;
  padding: 8px;
  height: 32px;
  gap: 8px;
  svg {
    width: 16px;
    height: 16px;
    color: ${theme.palette.grey[600]};
  },
`;

export const DeleteButton = styled(StyledMenuItem)`
  color: ${theme.palette.error.main};
  svg {
    color: ${theme.palette.error.main};
  }
  &:hover {
    background-color: ${theme.palette.error.main}1A;
  }
  &:active {
    background-color: ${theme.palette.error.main}1A;
  }
`;

export const MenuDivider = styled("div")`
  width: 100%;
  margin: auto;
  margin-top: 4px;
  margin-bottom: 4px;
  border-top: 1px solid ${theme.palette.grey[200]};
`;

export const ItemNameStyled = styled("div")`
  font-size: 12px;
  color: ${theme.palette.grey[900]};
  flex-grow: 2;
`;

export const ChevronRightStyled = styled(ChevronRight)`
  width: 16px;
  height: 16px;
  color: ${theme.palette.grey[900]};
`;

export default CellContextMenu;
