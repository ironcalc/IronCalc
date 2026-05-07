import {
  BookOpen,
  Check,
  ChevronRight,
  FileDown,
  FileUp,
  Globe,
  Keyboard,
  Menu as MenuIcon,
  Plus,
  Table2,
  Trash2,
} from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import DeleteWorkbookDialog from "../DeleteWorkbookDialog";
import { getModelsMetadata, getSelectedUuid } from "../storage";
import UploadFileDialog from "../UploadFileDialog/UploadFileDialog";
import {
  DeleteButton,
  MenuDivider,
  MenuItemText,
  MenuItemWrapper,
  MenuPaper,
} from "./FileMenu";
import "./navigation-menus.css";
import { useMenuAnchor } from "./useMenuAnchor";
import { useMobileLanguageMenu } from "./useMobileLanguageMenu";

interface MobileMenuProps {
  newModel: () => void;
  newModelFromTemplate: () => void;
  onDownload: () => Promise<void>;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
  onDelete: () => void;
  onLanguageChange: (language: string) => void;
}

export function MobileMenu(props: MobileMenuProps) {
  const { t, i18n } = useTranslation();
  const [isMenuOpen, setMenuOpen] = useState(false);
  const [isImportMenuOpen, setImportMenuOpen] = useState(false);
  const [isDeleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const models = getModelsMetadata();
  const selectedUuid = getSelectedUuid();

  const langMenu = useMobileLanguageMenu();
  const { anchorElement, menuElement, menuStyle } = useMenuAnchor(
    isMenuOpen,
    handleMenuClose,
    [langMenu.menuRef],
  );

  function handleMenuClose() {
    setMenuOpen(false);
    langMenu.close();
  }

  useEffect(() => {
    if (!isMenuOpen) langMenu.close();
  }, [isMenuOpen, langMenu.close]);

  const handleLanguageSelect = (language: string) => {
    i18n.changeLanguage(language);
    props.onLanguageChange(language);
    handleMenuClose();
  };

  return (
    <>
      <button
        type="button"
        ref={anchorElement}
        id="mobile-menu-button"
        aria-haspopup="true"
        className={`app-ic-nav-mobile-button${isMenuOpen ? " is-active" : ""}`}
        onClick={() => setMenuOpen(true)}
      >
        <MenuIcon />
      </button>

      {isMenuOpen && (
        <MenuPaper ref={menuElement} style={menuStyle}>
          <MenuItemWrapper
            onClick={() => {
              props.newModel();
              handleMenuClose();
            }}
          >
            <Plus />
            {t("file_bar.file_menu.new_blank_workbook")}
          </MenuItemWrapper>
          <MenuItemWrapper
            onClick={() => {
              props.newModelFromTemplate();
              handleMenuClose();
            }}
          >
            <Table2 />
            {t("file_bar.file_menu.new_from_template")}
          </MenuItemWrapper>
          <MenuItemWrapper
            onClick={() => {
              setImportMenuOpen(true);
              handleMenuClose();
            }}
          >
            <FileUp />
            {t("file_bar.file_menu.import.button")}
          </MenuItemWrapper>
          <MenuDivider />
          <MenuItemWrapper
            onClick={async () => {
              await props.onDownload();
              handleMenuClose();
            }}
          >
            <FileDown />
            <MenuItemText>{t("file_bar.file_menu.download")}</MenuItemText>
          </MenuItemWrapper>
          <DeleteButton
            onClick={() => {
              setDeleteDialogOpen(true);
              handleMenuClose();
            }}
          >
            <Trash2 />
            <MenuItemText>
              {t("file_bar.file_menu.delete_workbook.button")}
            </MenuItemText>
          </DeleteButton>
          <MenuDivider />
          <MenuItemWrapper
            ref={langMenu.anchorRef}
            onClick={langMenu.toggle}
            className="app-ic-nav-menu-item--space-between"
          >
            <Globe />
            <MenuItemText>
              {t("file_bar.file_menu.default_language")}
            </MenuItemText>
            <ChevronRight size={16} />
          </MenuItemWrapper>
          <MenuDivider />
          <MenuItemWrapper
            onClick={() => {
              handleMenuClose();
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
              handleMenuClose();
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

      {langMenu.isOpen && (
        <div
          ref={langMenu.menuRef}
          role="menu"
          className="app-ic-nav-submenu"
          style={langMenu.style}
        >
          {(
            [
              ["en-US", "English (en-US)"],
              ["en-GB", "English (en-GB)"],
              ["es-ES", "Español (es-ES)"],
              ["fr-FR", "Français (fr-FR)"],
              ["de-DE", "Deutsch (de-DE)"],
              ["it-IT", "Italiano (it-IT)"],
            ] as const
          ).map(([lang, label]) => (
            <MenuItemWrapper
              key={lang}
              onClick={() => handleLanguageSelect(lang)}
            >
              {i18n.language === lang ? (
                <Check size={16} />
              ) : (
                <span className="app-ic-nav-icon-placeholder" />
              )}
              {label}
            </MenuItemWrapper>
          ))}
        </div>
      )}

      {isImportMenuOpen && (
        <UploadFileDialog
          onClose={() => setImportMenuOpen(false)}
          onModelUpload={props.onModelUpload}
        />
      )}

      <DeleteWorkbookDialog
        open={isDeleteDialogOpen}
        onClose={() => setDeleteDialogOpen(false)}
        onConfirm={props.onDelete}
        workbookName={(selectedUuid && models[selectedUuid]?.name) || ""}
      />
    </>
  );
}
