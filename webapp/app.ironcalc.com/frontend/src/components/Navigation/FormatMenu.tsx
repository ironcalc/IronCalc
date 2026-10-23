import {
  Button,
  type IronCalcHandle,
  Menu,
  MenuItem,
} from "@ironcalc/workbook";
import { Bold } from "lucide-react";
import { type RefObject, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

export function FormatMenu(props: {
  // The widget's handle: the menu runs its actions through it
  ironCalcRef: RefObject<IronCalcHandle | null>;
  isReadOnly: boolean;
  isOpen: boolean;
  onOpen: () => void;
  onClose: () => void;
  onHover: () => void;
}) {
  const [anchorPosition, setAnchorPosition] = useState({ x: 0, y: 0 });
  const triggerRef = useRef<HTMLButtonElement>(null);
  const { t } = useTranslation();

  const captureAnchor = () => {
    const rect = triggerRef.current?.getBoundingClientRect();
    if (rect) {
      setAnchorPosition({ x: rect.left, y: rect.bottom + 4 });
    }
  };

  return (
    <div>
      <Button
        ref={triggerRef}
        variant="ghost"
        id="format-button"
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
        {t("file_bar.format_menu.button")}
      </Button>

      <Menu
        open={props.isOpen}
        onClose={props.onClose}
        anchorPosition={anchorPosition}
      >
        <MenuItem
          icon={<Bold />}
          disabled={props.isReadOnly}
          onClick={() => props.ironCalcRef.current?.workbook?.toggleBold()}
        >
          {t("file_bar.format_menu.bold")}
        </MenuItem>
      </Menu>
    </div>
  );
}
