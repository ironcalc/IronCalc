import { Button, Menu, MenuDivider, MenuItem } from "@ironcalc/workbook";
import { BookOpen, Info, Keyboard } from "lucide-react";
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";

export function HelpMenu(props: {
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
        id="help-button"
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
        {t("file_bar.help_menu.button")}
      </Button>

      <Menu
        open={props.isOpen}
        onClose={props.onClose}
        anchorPosition={anchorPosition}
      >
        <MenuItem
          icon={<BookOpen />}
          onClick={() => {
            window.open(
              "https://docs.ironcalc.com/web-application/about.html",
              "_blank",
              "noopener,noreferrer",
            );
          }}
        >
          {t("file_bar.help_menu.documentation")}
        </MenuItem>
        <MenuItem
          icon={<Keyboard />}
          onClick={() => {
            window.open(
              "https://docs.ironcalc.com/features/keyboard-shortcuts.html",
              "_blank",
              "noopener,noreferrer",
            );
          }}
        >
          {t("file_bar.help_menu.keyboard_shortcuts")}
        </MenuItem>
        <MenuDivider />
        <MenuItem
          icon={<Info />}
          onClick={() => {
            window.open(
              "https://www.ironcalc.com",
              "_blank",
              "noopener,noreferrer",
            );
          }}
        >
          {t("file_bar.help_menu.about")}
        </MenuItem>
      </Menu>
    </div>
  );
}
