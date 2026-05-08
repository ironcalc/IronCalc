import { Copy, FileDown, Pin, PinOff, Trash2 } from "lucide-react";
import { useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import {
  DeleteButton,
  MenuDivider,
  MenuItemWrapper,
} from "../Navigation/FileMenu";

interface WorkbookMenuProps {
  position: { top: number; right: number };
  isPinned: boolean;
  onClose: () => void;
  onDownload: () => void;
  onPinToggle: () => void;
  onDuplicate: () => void;
  onDelete: () => void;
}

function WorkbookMenu({
  position,
  isPinned,
  onClose,
  onDownload,
  onPinToggle,
  onDuplicate,
  onDelete,
}: WorkbookMenuProps) {
  const { t } = useTranslation();
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [onClose]);

  return createPortal(
    <>
      <div
        className="app-ic-drawer-menu-backdrop"
        onClick={onClose}
        role="none"
      />
      <div
        ref={menuRef}
        role="menu"
        className="app-ic-nav-menu"
        style={{ position: "fixed", top: position.top, right: position.right }}
      >
        <MenuItemWrapper onClick={onDownload}>
          <FileDown />
          {t("left_drawer.workbook_menu.download")}
        </MenuItemWrapper>

        <MenuItemWrapper onClick={onPinToggle}>
          {isPinned ? <PinOff /> : <Pin />}
          {isPinned
            ? t("left_drawer.workbook_menu.unpin")
            : t("left_drawer.workbook_menu.pin")}
        </MenuItemWrapper>

        <MenuItemWrapper onClick={onDuplicate}>
          <Copy />
          {t("left_drawer.workbook_menu.duplicate")}
        </MenuItemWrapper>

        <MenuDivider />

        <DeleteButton onClick={onDelete}>
          <Trash2 size={16} />
          {t("left_drawer.workbook_menu.delete")}
        </DeleteButton>
      </div>
    </>,
    document.body,
  );
}

export default WorkbookMenu;
