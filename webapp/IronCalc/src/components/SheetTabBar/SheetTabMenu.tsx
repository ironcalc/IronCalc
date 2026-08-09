import {
  ArrowLeft,
  ArrowRight,
  Copy,
  EyeOff,
  PaintBucket,
  TextCursorInput,
  Trash2,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { MenuDivider } from "../Menu/MenuDivider";
import { MenuItem } from "../Menu/MenuItem";

interface SheetTabMenuProps {
  canDelete: boolean;
  onMoveLeft: () => void;
  onMoveRight: () => void;
  onStartEditing: () => void;
  onOpenColorPicker: () => void;
  onDuplicateSheet: () => void;
  onHideSheet: () => void;
  onDeleteSheet: () => void;
}

export function SheetTabMenu({
  canDelete,
  onMoveLeft,
  onMoveRight,
  onStartEditing,
  onOpenColorPicker,
  onDuplicateSheet,
  onHideSheet,
  onDeleteSheet,
}: SheetTabMenuProps) {
  const { t } = useTranslation();

  return (
    <>
      <MenuItem icon={<ArrowLeft />} onClick={onMoveLeft}>
        {t("sheet_tab.move_left")}
      </MenuItem>
      <MenuItem icon={<ArrowRight />} onClick={onMoveRight}>
        {t("sheet_tab.move_right")}
      </MenuItem>
      <MenuItem icon={<TextCursorInput />} onClick={onStartEditing}>
        {t("sheet_tab.rename")}
      </MenuItem>
      <MenuItem icon={<PaintBucket />} onClick={onOpenColorPicker}>
        {t("sheet_tab.change_color")}
      </MenuItem>
      <MenuItem icon={<Copy />} onClick={onDuplicateSheet}>
        {t("sheet_tab.duplicate")}
      </MenuItem>
      <MenuItem icon={<EyeOff />} disabled={!canDelete} onClick={onHideSheet}>
        {t("sheet_tab.hide_sheet")}
      </MenuItem>
      <MenuDivider />
      <MenuItem
        icon={<Trash2 />}
        disabled={!canDelete}
        destructive
        onClick={onDeleteSheet}
      >
        {t("sheet_tab.delete")}
      </MenuItem>
    </>
  );
}
