import type { MenuItemProps } from "@mui/material";
import { Button, Input, Menu, MenuItem, styled } from "@mui/material";
import {
  ChevronDown,
  EyeOff,
  PaintBucket,
  TextCursorInput,
  Trash2,
} from "lucide-react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { theme } from "../../theme";
import ColorPicker from "../ColorPicker/ColorPicker";
import { isInReferenceMode } from "../Editor/util";
import type { WorkbookState } from "../workbookState";
import SheetDeleteDialog from "./SheetDeleteDialog";

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
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const [colorPickerOpen, setColorPickerOpen] = useState(false);
  const colorButton = useRef<HTMLDivElement>(null);
  const open = Boolean(anchorEl);
  const { t } = useTranslation();
  const handleOpen = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
  };
  const handleClose = () => {
    setAnchorEl(null);
  };
  const handleContextMenu = (event: React.MouseEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.stopPropagation();
    onSelected();
    setAnchorEl(event.currentTarget);
  };

  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);

  const handleOpenDeleteDialog = () => {
    setDeleteDialogOpen(true);
  };

  const handleCloseDeleteDialog = () => {
    setDeleteDialogOpen(false);
  };

  const [isEditing, setIsEditing] = useState(false);
  const [editingName, setEditingName] = useState(name);
  const inputRef = useRef<HTMLInputElement>(null);
  const measureRef = useRef<HTMLSpanElement>(null);
  const [inputWidth, setInputWidth] = useState<number>(0);

  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditing]);

  useEffect(() => {
    if (!isEditing) {
      setEditingName(name);
    }
  }, [name, isEditing]);

  // We want to change the layout only when editingName changes, but the layout is controlled by the hidden measure element (measureRef).
  // biome-ignore lint/correctness/useExhaustiveDependencies: false
  useLayoutEffect(() => {
    if (isEditing && measureRef.current) {
      const width = measureRef.current.offsetWidth;
      setInputWidth(Math.max(width + 8, 6));
    }
  }, [editingName, isEditing]);

  const handleStartEditing = () => {
    setEditingName(name);
    setInputWidth(Math.max(name.length * 7 + 8, 6));
    setIsEditing(true);
  };

  const handleSave = () => {
    if (editingName.trim() !== "") {
      props.onRenamed(editingName.trim());
      setIsEditing(false);
    } else {
      setEditingName(name);
      setIsEditing(false);
    }
  };

  const handleCancel = () => {
    setEditingName(name);
    setIsEditing(false);
  };
  return (
    <>
      <TabWrapper
        $color={color}
        $selected={selected}
        onClick={(event: React.MouseEvent) => {
          if (!isEditing) {
            onSelected();
          }
          event.stopPropagation();
          event.preventDefault();
        }}
        onDoubleClick={(event: React.MouseEvent) => {
          event.stopPropagation();
          event.preventDefault();
          handleStartEditing();
        }}
        onContextMenu={handleContextMenu}
        onPointerDown={(event: React.PointerEvent) => {
          const cell = workbookState.getEditingCell();
          if (cell && isInReferenceMode(cell.text, cell.cursorStart)) {
            event.stopPropagation();
            event.preventDefault();
          }
        }}
        ref={colorButton}
      >
        {isEditing ? (
          <>
            <HiddenMeasure ref={measureRef}>{editingName || " "}</HiddenMeasure>
            <StyledInput
              inputRef={inputRef}
              value={editingName}
              onChange={(e) => setEditingName(e.target.value)}
              style={{ width: `${inputWidth}px` }}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  handleSave();
                } else if (e.key === "Escape") {
                  e.preventDefault();
                  handleCancel();
                }
                e.stopPropagation();
              }}
              onBlur={() => {
                handleSave();
              }}
              onClick={(e) => e.stopPropagation()}
              spellCheck={false}
            />
            <StyledButton disableRipple disabled $active={false}>
              <ChevronDown />
            </StyledButton>
          </>
        ) : (
          <>
            <Name>{name}</Name>
            <StyledButton onClick={handleOpen} disableRipple $active={open}>
              <ChevronDown />
            </StyledButton>
          </>
        )}
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
            handleStartEditing();
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
  padding: 0px 4px 0px 6px;
  align-items: center;
  cursor: pointer;
  min-width: 40px;
  font-weight: ${(props) => (props.$selected ? 600 : 400)};
  background-color: ${(props) =>
    props.$selected ? `${theme.palette.grey[50]}` : "transparent"};
  &:hover {
    background-color: ${theme.palette.grey[50]}80;
  }
`;

const StyledButton = styled(Button)<{ $active: boolean }>`
  width: 16px;
  height: 16px;
  min-width: 0px;
  padding: 0px;
  color: inherit;
  font-weight: inherit;
  border-radius: 4px;
  flex-shrink: 0;
  background-color: ${(props) =>
    props.$active ? `${theme.palette.grey[300]}` : "transparent"};
  &:hover {
    background-color: ${theme.palette.grey[200]};
  }
  &:active {
    background-color: ${theme.palette.grey[300]};
  }
  &:disabled {
    pointer-events: none;
  }
  svg {
    width: 14px;
    height: 14px;
  }
`;

const Name = styled("div")`
  font-size: 12px;
  margin-right: 5px;
  text-wrap: nowrap;
  user-select: none;
  width: 100%;
  text-align: center;
`;

const HiddenMeasure = styled("span")`
  position: absolute;
  visibility: hidden;
  white-space: pre;
  font-size: 12px;
  font-family: Inter;
  font-weight: inherit;
  padding: 0;
  margin: 0;
  height: 100%;
  overflow: hidden;
  pointer-events: none;
`;

const StyledInput = styled(Input)`
  font-size: 12px;
  font-family: Inter;
  font-weight: inherit;
  min-width: 6px;
  margin-right: 2px;
  min-height: 100%;
  flex-grow: 1;
  & .MuiInputBase-input {
    font-family: Inter;
    background-color: ${theme.palette.common.white};
    font-weight: inherit;
    padding: 6px 0px;
    border: 1px solid ${theme.palette.primary.main};
    border-radius: 4px;
    color: ${theme.palette.common.black};
    text-align: center;
    will-change: width;
    &:focus {
      border-color: ${theme.palette.primary.main};
    }
  }

  &::before,
  &::after {
    display: none;
  }
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
