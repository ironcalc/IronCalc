import { Button, Menu, MenuItem, styled } from "@mui/material";
import type { MenuItemProps } from "@mui/material";
import {
  ChevronDown,
  EyeOff,
  PaintBucket,
  TextCursorInput,
  Trash2,
} from "lucide-react";
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { theme } from "../../theme";
import ColorPicker from "../ColorPicker/ColorPicker";
import { isInReferenceMode } from "../Editor/util";
import type { WorkbookState } from "../workbookState";
import SheetDeleteDialog from "./SheetDeleteDialog";
import SheetRenameDialog from "./SheetRenameDialog";

interface SheetTabProps {
  name: string;
  color: string;
  selected: boolean;
  onSelected: () => void;
  onColorChanged: (hex: string) => void;
  onRenamed: (name: string) => void;
  canDelete: boolean;
  onDeleted: () => void;
  onHideSheet: () => void;
  workbookState: WorkbookState;
}

function SheetTab(props: SheetTabProps) {
  const { name, color, selected, workbookState, onSelected } = props;
  const [anchorEl, setAnchorEl] = useState<null | HTMLButtonElement>(null);
  const [colorPickerOpen, setColorPickerOpen] = useState(false);
  const colorButton = useRef<HTMLDivElement>(null);
  const open = Boolean(anchorEl);
  const { t } = useTranslation();
  const handleOpen = (event: React.MouseEvent<HTMLButtonElement>) => {
    setAnchorEl(event.currentTarget);
  };
  const handleClose = () => {
    setAnchorEl(null);
  };
  const [renameDialogOpen, setRenameDialogOpen] = useState(false);
  const handleCloseRenameDialog = () => {
    setRenameDialogOpen(false);
  };

  const handleOpenRenameDialog = () => {
    setRenameDialogOpen(true);
  };

  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);

  const handleOpenDeleteDialog = () => {
    setDeleteDialogOpen(true);
  };

  const handleCloseDeleteDialog = () => {
    setDeleteDialogOpen(false);
  };
  return (
    <>
      <TabWrapper
        $color={color}
        $selected={selected}
        onClick={(event: React.MouseEvent) => {
          onSelected();
          event.stopPropagation();
          event.preventDefault();
        }}
        onPointerDown={(event: React.PointerEvent) => {
          // If it is in browse mode stop he event
          const cell = workbookState.getEditingCell();
          if (cell && isInReferenceMode(cell.text, cell.cursorStart)) {
            event.stopPropagation();
            event.preventDefault();
          }
        }}
        ref={colorButton}
      >
        <Name onDoubleClick={handleOpenRenameDialog}>{name}</Name>
        <StyledButton onClick={handleOpen}>
          <ChevronDown />
        </StyledButton>
      </TabWrapper>
      <StyledMenu
        anchorEl={anchorEl}
        open={open}
        onClose={handleClose}
        anchorOrigin={{
          vertical: "top",
          horizontal: "left",
        }}
        transformOrigin={{
          vertical: "bottom",
          horizontal: 6,
        }}
      >
        <StyledMenuItem
          onClick={() => {
            handleOpenRenameDialog();
            handleClose();
          }}
        >
          <TextCursorInput />
          {t("sheet_tab.rename")}
        </StyledMenuItem>
        <StyledMenuItem
          onClick={() => {
            setColorPickerOpen(true);
            handleClose();
          }}
        >
          <PaintBucket />
          {t("sheet_tab.change_color")}
        </StyledMenuItem>
        <StyledMenuItem
          disabled={!props.canDelete}
          onClick={() => {
            props.onHideSheet();
            handleClose();
          }}
        >
          <EyeOff />
          {t("sheet_tab.hide_sheet")}
        </StyledMenuItem>
        <MenuDivider />
        <DeleteButton
          disabled={!props.canDelete}
          onClick={() => {
            handleOpenDeleteDialog();
            handleClose();
          }}
        >
          <Trash2 />
          {t("sheet_tab.delete")}
        </DeleteButton>
      </StyledMenu>
      <SheetRenameDialog
        open={renameDialogOpen}
        onClose={handleCloseRenameDialog}
        defaultName={name}
        onNameChanged={(newName) => {
          props.onRenamed(newName);
          setRenameDialogOpen(false);
        }}
      />
      <ColorPicker
        color={color}
        defaultColor="#FFFFFF"
        title={t("color_picker.no_fill")}
        onChange={(color): void => {
          props.onColorChanged(color);
          setColorPickerOpen(false);
        }}
        onClose={() => {
          setColorPickerOpen(false);
        }}
        anchorEl={colorButton}
        open={colorPickerOpen}
        anchorOrigin={{ vertical: "bottom", horizontal: "right" }}
        transformOrigin={{ vertical: "bottom", horizontal: "left" }}
      />
      <SheetDeleteDialog
        open={deleteDialogOpen}
        onClose={handleCloseDeleteDialog}
        onDelete={() => {
          props.onDeleted();
          handleCloseDeleteDialog();
        }}
        sheetName={name}
      />
    </>
  );
}

const StyledMenu = styled(Menu)`
  & .MuiPaper-root {
    border-radius: 8px;
    padding: 4px 0px;
    margin-left: -4px;
  }
  & .MuiList-root {
    padding: 0;
  }
`;

const StyledMenuItem = styled(MenuItem)<MenuItemProps>(() => ({
  display: "flex",
  justifyContent: "flex-start",
  alignItems: "center",
  gap: "8px",
  fontSize: "12px",
  width: "calc(100% - 8px)",
  margin: "0px 4px",
  borderRadius: "4px",
  padding: "8px",
  height: "32px",
  "&:disabled": {
    color: "#BDBDBD",
  },
  "& svg": {
    width: "16px",
    height: "16px",
    color: `${theme.palette.grey[600]}`,
  },
}));

const TabWrapper = styled("div")<{ $color: string; $selected: boolean }>`
  display: flex;
  margin-right: 12px;
  border-bottom: 3px solid ${(props) => props.$color};
  line-height: 37px;
  padding: 0px 4px;
  align-items: center;
  cursor: pointer;
  font-weight: ${(props) => (props.$selected ? 600 : 400)};
  background-color: ${(props) =>
    props.$selected ? `${theme.palette.grey[50]}80` : "transparent"};
`;

const StyledButton = styled(Button)`
  width: 15px;
  height: 24px;
  min-width: 0px;
  padding: 0px;
  color: inherit;
  font-weight: inherit;
  &:hover {
    background-color: transparent;
  }
  &:active {
    background-color: transparent;
  }
  svg {
    width: 15px;
    height: 15px;
    transition: transform 0.2s;
  }
  &:hover svg {
    transform: translateY(2px);
  }
`;

const Name = styled("div")`
  font-size: 12px;
  margin-right: 5px;
  text-wrap: nowrap;
  user-select: none;
`;

const MenuDivider = styled("div")`
  width: 100%;
  margin: auto;
  margin-top: 4px;
  margin-bottom: 4px;
  border-top: 1px solid ${theme.palette.grey[200]};
`;

const DeleteButton = styled(StyledMenuItem)`
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

export default SheetTab;
