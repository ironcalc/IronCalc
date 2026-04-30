import "./navigation.css";
import { BookOpen, Keyboard } from "lucide-react";
import { useTranslation } from "react-i18next";

export function HelpMenu(props: {
  isOpen: boolean;
  onOpen: () => void;
  onClose: () => void;
  onHover: () => void;
}) {
  const { t } = useTranslation();

  return (
    <div className="nav-menu-wrapper">
      <button
        type="button"
        className={`nav-menu-trigger${props.isOpen ? " is-active" : ""}`}
        onClick={props.onOpen}
        onMouseEnter={props.onHover}
        aria-haspopup="true"
        aria-expanded={props.isOpen}
      >
        {t("file_bar.help_menu.button")}
      </button>
      {props.isOpen && (
        <div className="nav-menu-panel">
          <button
            type="button"
            className="nav-menu-item"
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
          </button>
          <button
            type="button"
            className="nav-menu-item"
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
          </button>
        </div>
      )}
    </div>
  );
}
