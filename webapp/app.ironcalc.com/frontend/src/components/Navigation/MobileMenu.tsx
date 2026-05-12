import {
  IconButton,
  Menu,
  MenuDivider,
  MenuItem,
  MenuItemWithSubmenu,
} from "@ironcalc/workbook";
import {
  BookOpen,
  FileDown,
  FileUp,
  Globe,
  Info,
  Keyboard,
  Menu as MenuIcon,
  Plus,
  Table2,
  Trash2,
} from "lucide-react";
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import DeleteWorkbookDialog from "../DeleteWorkbookDialog";
import { getModelsMetadata, getSelectedUuid } from "../storage";
import UploadFileDialog from "../UploadFileDialog/UploadFileDialog";

const LANGUAGES = [
  ["en-US", "English"],
  ["en-GB", "English"],
  ["es-ES", "Español"],
  ["fr-FR", "Français"],
  ["de-DE", "Deutsch"],
  ["it-IT", "Italiano"],
] as const;

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
  const [anchorPosition, setAnchorPosition] = useState({ x: 0, y: 0 });
  const triggerRef = useRef<HTMLButtonElement>(null);
  const models = getModelsMetadata();
  const selectedUuid = getSelectedUuid();

  const handleOpen = () => {
    const rect = triggerRef.current?.getBoundingClientRect();
    if (rect) {
      setAnchorPosition({ x: rect.left, y: rect.bottom + 4 });
    }
    setMenuOpen(true);
  };

  return (
    <>
      <IconButton
        ref={triggerRef}
        id="mobile-menu-button"
        aria-label="Open menu"
        aria-haspopup="menu"
        aria-expanded={isMenuOpen ? "true" : "false"}
        size="md"
        icon={<MenuIcon />}
        onClick={handleOpen}
      />

      <Menu
        open={isMenuOpen}
        onClose={() => setMenuOpen(false)}
        anchorPosition={anchorPosition}
      >
        <MenuItem icon={<Plus />} onClick={() => props.newModel()}>
          {t("file_bar.file_menu.new_blank_workbook")}
        </MenuItem>
        <MenuItem
          icon={<Table2 />}
          onClick={() => props.newModelFromTemplate()}
        >
          {t("file_bar.file_menu.new_from_template")}
        </MenuItem>
        <MenuItem icon={<FileUp />} onClick={() => setImportMenuOpen(true)}>
          {t("file_bar.file_menu.import.button")}
        </MenuItem>
        <MenuDivider />
        <MenuItem icon={<FileDown />} onClick={() => props.onDownload()}>
          {t("file_bar.file_menu.download")}
        </MenuItem>
        <MenuItem
          destructive
          icon={<Trash2 />}
          onClick={() => setDeleteDialogOpen(true)}
        >
          {t("file_bar.file_menu.delete_workbook.button")}
        </MenuItem>
        <MenuDivider />
        <MenuItemWithSubmenu
          icon={<Globe />}
          submenu={LANGUAGES.map(([lang, label]) => (
            <MenuItem
              key={lang}
              checked={i18n.language === lang}
              secondaryText={lang}
              onClick={() => {
                i18n.changeLanguage(lang);
                props.onLanguageChange(lang);
              }}
            >
              {label}
            </MenuItem>
          ))}
        >
          {t("file_bar.file_menu.default_language")}
        </MenuItemWithSubmenu>
        <MenuDivider />
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
