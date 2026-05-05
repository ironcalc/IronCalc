import { BookOpen, Keyboard } from "lucide-react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { MenuItemWrapper, MenuPaper } from "./FileMenu";
import "./navigation-menus.css";

export function HelpMenu(props: {
  isOpen: boolean;
  onOpen: () => void;
  onClose: () => void;
  onHover: () => void;
}) {
  const anchorElement = useRef<HTMLButtonElement>(null);
  const menuElement = useRef<HTMLDivElement>(null);
  const [menuStyle, setMenuStyle] = useState<{ left?: number; top?: number }>(
    {},
  );
  const { t } = useTranslation();

  useLayoutEffect(() => {
    if (!props.isOpen || !anchorElement.current) return;

    const update = () => {
      const rect = anchorElement.current?.getBoundingClientRect();
      if (rect) setMenuStyle({ left: rect.left - 4, top: rect.bottom + 4 });
    };

    update();
    window.addEventListener("resize", update);
    window.addEventListener("scroll", update, true);
    return () => {
      window.removeEventListener("resize", update);
      window.removeEventListener("scroll", update, true);
    };
  }, [props.isOpen]);

  useEffect(() => {
    if (!props.isOpen) return;

    const onPointerDown = (event: PointerEvent) => {
      const target = event.target as Node | null;
      if (
        anchorElement.current?.contains(target) ||
        menuElement.current?.contains(target)
      )
        return;
      props.onClose();
    };

    document.addEventListener("pointerdown", onPointerDown, true);
    return () =>
      document.removeEventListener("pointerdown", onPointerDown, true);
  }, [props.isOpen, props.onClose]);

  return (
    <div>
      <button
        type="button"
        ref={anchorElement}
        id="help-button"
        aria-controls={props.isOpen ? "help-menu" : undefined}
        aria-haspopup="true"
        onClick={props.onOpen}
        onMouseEnter={props.onHover}
        className={`app-ic-nav-button${props.isOpen ? " is-active" : ""}`}
      >
        {t("file_bar.help_menu.button")}
      </button>

      {props.isOpen && (
        <MenuPaper id="help-menu" ref={menuElement} style={menuStyle}>
          <MenuItemWrapper
            onClick={() => {
              props.onClose();
              window.open(
                "https://docs.ironcalc.com/web-application/about.html",
                "_blank",
                "noopener,noreferrer",
              );
            }}
          >
            <BookOpen />
            {t("file_bar.help_menu.documentation")}
          </MenuItemWrapper>
          <MenuItemWrapper
            onClick={() => {
              props.onClose();
              window.open(
                "https://docs.ironcalc.com/features/keyboard-shortcuts.html",
                "_blank",
                "noopener,noreferrer",
              );
            }}
          >
            <Keyboard />
            {t("file_bar.help_menu.keyboard_shortcuts")}
          </MenuItemWrapper>
        </MenuPaper>
      )}
    </div>
  );
}
