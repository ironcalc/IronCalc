import { Button, Menu, MenuItem, styled } from "@mui/material";
import { ChevronDown } from "lucide-react";
import { useRef, useState } from "react";
import ColorPicker from "../colorPicker";
import { SheetRenameDialog } from "./menus";
interface SheetProps {
  name: string;
  color: string;
  selected: boolean;
  onSelected: () => void;
  onColorChanged: (hex: string) => void;
  onRenamed: (name: string) => void;
  onDeleted: () => void;
}
function Sheet(props: SheetProps) {
  const { name, color, selected, onSelected } = props;
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
      <Wrapper
        style={{ borderBottomColor: color, fontWeight: selected ? 600 : 400 }}
        onClick={() => {
          onSelected();
        }}
        ref={colorButton}
      >
        <Name>{name}</Name>
        <StyledButton onClick={handleOpen}>
          <ChevronDown />
        </StyledButton>
      </Wrapper>
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
        <MenuItem
          onClick={() => {
            handleOpenRenameDialog();
            handleClose();
          }}
        >
          Rename
        </MenuItem>
        <MenuItem
          onClick={() => {
            setColorPickerOpen(true);
            handleClose();
          }}
        >
          Change Color
        </MenuItem>
        <MenuItem
          onClick={() => {
            props.onDeleted();
            handleClose();
          }}
        >
          {" "}
          Delete
        </MenuItem>
      </StyledMenu>
      <SheetRenameDialog
        isOpen={renameDialogOpen}
        close={handleCloseRenameDialog}
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

const StyledMenu = styled(Menu)``;

const StyledButton = styled(Button)`
  width: 15px;
  height: 24px;
  min-width: 0px;
  padding: 0px;
  color: inherit;
  font-weight: inherit;
  svg {
    width: 15px;
    height: 15px;
  }
`;

const Wrapper = styled("div")`
  display: flex;
  margin-left: 20px;
  border-bottom: 3px solid;
  border-top: 3px solid white;
  line-height: 34px;
  align-items: center;
`;

const Name = styled("div")`
  font-size: 12px;
  margin-right: 5px;
  text-wrap: nowrap;
`;

export default Sheet;
