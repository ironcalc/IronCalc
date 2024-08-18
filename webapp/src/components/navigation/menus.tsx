import {
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  TextField,
  styled,
} from "@mui/material";
import Menu from "@mui/material/Menu";
import MenuItem from "@mui/material/MenuItem";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { SheetOptions } from "./types";

interface SheetRenameDialogProps {
  isOpen: boolean;
  close: () => void;
  onNameChanged: (name: string) => void;
  defaultName: string;
}

export const SheetRenameDialog = (properties: SheetRenameDialogProps) => {
  const { t } = useTranslation();
  const [name, setName] = useState(properties.defaultName);
  return (
    <Dialog open={properties.isOpen} onClose={properties.close}>
      <DialogTitle>{t("sheet_rename.title")}</DialogTitle>
      <DialogContent dividers>
        <TextField
          defaultValue={name}
          label={t("sheet_rename.label")}
          onClick={(event) => event.stopPropagation()}
          onKeyDown={(event) => {
            event.stopPropagation();
          }}
          onChange={(event) => {
            setName(event.target.value);
          }}
          spellCheck="false"
        />
      </DialogContent>
      <DialogActions>
        <Button
          onClick={() => {
            properties.onNameChanged(name);
          }}
        >
          {t("sheet_rename.rename")}
        </Button>
      </DialogActions>
    </Dialog>
  );
};

interface SheetListMenuProps {
  isOpen: boolean;
  close: () => void;
  anchorEl: HTMLButtonElement | null;
  onSheetSelected: (index: number) => void;
  sheetOptionsList: SheetOptions[];
}

const SheetListMenu = (properties: SheetListMenuProps) => {
  const { isOpen, close, anchorEl, onSheetSelected, sheetOptionsList } =
    properties;

  return (
    <StyledMenu
      open={isOpen}
      onClose={close}
      anchorEl={anchorEl}
      anchorOrigin={{
        vertical: "top",
        horizontal: "left",
      }}
      transformOrigin={{
        vertical: "bottom",
        horizontal: 6,
      }}
    >
      {sheetOptionsList.map((tab, index) => (
        <StyledMenuItem
          key={tab.sheetId}
          onClick={(): void => onSheetSelected(index)}
        >
          <ItemColor style={{ backgroundColor: tab.color }} />
          <ItemName>{tab.name}</ItemName>
        </StyledMenuItem>
      ))}
    </StyledMenu>
  );
};

const StyledMenu = styled(Menu)({
  "& .MuiPaper-root": {
    borderRadius: 8,
    padding: 4,
  },
  "& .MuiList-padding": {
    padding: 0,
  },
});

const StyledMenuItem = styled(MenuItem)({
  padding: 8,
  borderRadius: 4,
});

const ItemColor = styled("div")`
  width: 12px;
  height: 12px;
  border-radius: 4px;
  margin-right: 8px;
`;

const ItemName = styled("div")`
  font-size: 13px;
  color: #333;
`;

export default SheetListMenu;
