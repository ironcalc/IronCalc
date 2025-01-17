import { Button, Menu, MenuItem, styled } from "@mui/material";
import type { MenuItemProps } from "@mui/material";
import { ChevronDown } from "lucide-react";
import { useRef, useState } from "react";
import { theme } from "../../theme";
import ColorPicker from "../colorPicker";
import { isInReferenceMode } from "../editor/util";
import type { WorkbookState } from "../workbookState";
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
  const colorButton = useRef(null);
  const open = Boolean(anchorEl);
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
  return (
    <>
      <TabWrapper
        $color={color}
        $selected={selected}
        onClick={(event) => {
          onSelected();
          event.stopPropagation();
          event.preventDefault();
        }}
        onPointerDown={(event) => {
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
          Rename
        </StyledMenuItem>
        <StyledMenuItem
          onClick={() => {
            setColorPickerOpen(true);
            handleClose();
          }}
        >
          Change Color
        </StyledMenuItem>
        <StyledMenuItem
          disabled={!props.canDelete}
          onClick={() => {
            props.onDeleted();
            handleClose();
          }}
        >
          Delete
        </StyledMenuItem>
        <StyledMenuItem
          disabled={!props.canDelete}
          onClick={() => {
            props.onHideSheet();
            handleClose();
          }}
        >
          Hide sheet
        </StyledMenuItem>
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
        onChange={(color): void => {
          props.onColorChanged(color);
          setColorPickerOpen(false);
        }}
        onClose={() => {
          setColorPickerOpen(false);
        }}
        anchorEl={colorButton}
        open={colorPickerOpen}
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
  justifyContent: "space-between",
  fontSize: "12px",
  width: "calc(100% - 8px)",
  margin: "0px 4px",
  borderRadius: "4px",
  padding: "8px",
  height: "32px",
  "&:disabled": {
    color: "#BDBDBD",
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

export default SheetTab;
