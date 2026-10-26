import {
  Button,
  type IronCalcHandle,
  Menu,
  MenuDivider,
  MenuItem,
  type Model,
} from "@ironcalc/workbook";
import { ClipboardPaste, Copy, Redo2, Scissors, Undo2 } from "lucide-react";
import { type RefObject, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

export function EditMenu(props: {
  ironCalcRef: RefObject<IronCalcHandle | null>;
  model: Model;
  isReadOnly: boolean;
  isOpen: boolean;
  onOpen: () => void;
  onClose: () => void;
  onHover: () => void;
}) {
  const [anchorPosition, setAnchorPosition] = useState({ x: 0, y: 0 });
  const triggerRef = useRef<HTMLButtonElement>(null);
  const { t } = useTranslation();
  const { model, isReadOnly } = props;

  const captureAnchor = () => {
    const rect = triggerRef.current?.getBoundingClientRect();
    if (rect) {
      setAnchorPosition({ x: rect.left, y: rect.bottom + 4 });
    }
  };

  const actions = () => props.ironCalcRef.current?.workbook;

  return (
    <div>
      <Button
        ref={triggerRef}
        variant="ghost"
        id="edit-button"
        aria-haspopup="menu"
        aria-expanded={props.isOpen ? "true" : "false"}
        onClick={() => {
          captureAnchor();
          props.onOpen();
        }}
        onMouseEnter={() => {
          captureAnchor();
          props.onHover();
        }}
      >
        {t("file_bar.edit_menu.button")}
      </Button>

      <Menu
        open={props.isOpen}
        onClose={props.onClose}
        anchorPosition={anchorPosition}
      >
        <MenuItem
          icon={<Undo2 />}
          disabled={isReadOnly || !model.canUndo()}
          onClick={() => actions()?.undo()}
        >
          {t("file_bar.edit_menu.undo")}
        </MenuItem>
        <MenuItem
          icon={<Redo2 />}
          disabled={isReadOnly || !model.canRedo()}
          onClick={() => actions()?.redo()}
        >
          {t("file_bar.edit_menu.redo")}
        </MenuItem>
        <MenuDivider />
        <MenuItem
          icon={<Scissors />}
          disabled={isReadOnly}
          onClick={() => actions()?.cut()}
        >
          {t("file_bar.edit_menu.cut")}
        </MenuItem>
        <MenuItem icon={<Copy />} onClick={() => actions()?.copy()}>
          {t("file_bar.edit_menu.copy")}
        </MenuItem>
        <MenuItem
          icon={<ClipboardPaste />}
          disabled={isReadOnly}
          onClick={() => actions()?.paste()}
        >
          {t("file_bar.edit_menu.paste")}
        </MenuItem>
      </Menu>
    </div>
  );
}
