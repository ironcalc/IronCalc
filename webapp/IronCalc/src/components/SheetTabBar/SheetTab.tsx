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

const StyledMenu = styled(Menu)({
  "& .MuiPaper-root": {
    borderRadius: 8,
    padding: "4px 0px",
    marginLeft: -4,
  },
  "& .MuiList-root": {
    padding: 0,
  },
});

const StyledMenuItem = styled(MenuItem)<MenuItemProps>(({ theme }) => ({
  display: "flex",
  justifyContent: "flex-start",
  alignItems: "center",
  gap: 8,
  fontSize: 12,
  width: "calc(100% - 8px)",
  margin: "0px 4px",
  borderRadius: 4,
  padding: 8,
  height: 32,
  "&:disabled": {
    color: "#BDBDBD",
  },
  "& svg": {
    width: 16,
    height: 16,
    color: theme.palette.grey[600],
  },
}));

const TabWrapper = styled("div")<{
  $color: string;
  $selected: boolean;
}>(({ theme, $color, $selected }) => ({
  display: "flex",
  marginRight: 12,
  borderBottom: `3px solid ${$color}`,
  lineHeight: "37px",
  padding: "0px 4px 0px 6px",
  alignItems: "center",
  cursor: "pointer",
  minWidth: 40,
  fontWeight: $selected ? 600 : 400,
  backgroundColor: $selected ? theme.palette.grey[50] : "transparent",
  "&:hover": {
    backgroundColor: `${theme.palette.grey[50]}80`,
  },
}));

const StyledButton = styled(Button, {
  shouldForwardProp: (prop) => prop !== "$active",
})<{ $active: boolean }>(({ theme, $active }) => ({
  width: 16,
  height: 16,
  minWidth: 0,
  padding: 0,
  color: "inherit",
  fontWeight: "inherit",
  borderRadius: 4,
  flexShrink: 0,
  backgroundColor: $active ? theme.palette.grey[300] : "transparent",
  "&:hover": {
    backgroundColor: theme.palette.grey[200],
  },
  "&:active": {
    backgroundColor: theme.palette.grey[300],
  },
  "&:disabled": {
    pointerEvents: "none",
  },
  "& svg": {
    width: 14,
    height: 14,
  },
}));

const Name = styled("div")({
  fontSize: 12,
  marginRight: 5,
  textWrap: "nowrap",
  userSelect: "none",
  width: "100%",
  textAlign: "center",
});

const HiddenMeasure = styled("span")({
  position: "absolute",
  visibility: "hidden",
  whiteSpace: "pre",
  fontSize: 12,
  fontFamily: "Inter",
  fontWeight: "inherit",
  padding: 0,
  margin: 0,
  height: "100%",
  overflow: "hidden",
  pointerEvents: "none",
});

const StyledInput = styled(Input)(({ theme }) => ({
  fontSize: 12,
  fontFamily: "Inter",
  fontWeight: "inherit",
  minWidth: 6,
  marginRight: 2,
  minHeight: "100%",
  flexGrow: 1,
  "& .MuiInputBase-input": {
    fontFamily: "Inter",
    backgroundColor: theme.palette.common.white,
    fontWeight: "inherit",
    padding: "6px 0px",
    border: `1px solid ${theme.palette.primary.main}`,
    borderRadius: 4,
    color: theme.palette.common.black,
    textAlign: "center",
    willChange: "width",
    "&:focus": {
      borderColor: theme.palette.primary.main,
    },
  },
  "&::before, &::after": {
    display: "none",
  },
}));

const MenuDivider = styled("div")(({ theme }) => ({
  width: "100%",
  margin: "auto",
  marginTop: 4,
  marginBottom: 4,
  borderTop: `1px solid ${theme.palette.grey[200]}`,
}));

const DeleteButton = styled(StyledMenuItem)(({ theme }) => ({
  color: theme.palette.error.main,
  "& svg": {
    color: theme.palette.error.main,
  },
  "&:hover": {
    backgroundColor: `${theme.palette.error.main}1A`,
  },
  "&:active": {
    backgroundColor: `${theme.palette.error.main}1A`,
  },
}));

export default SheetTab;
