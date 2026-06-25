import {
  Button,
  Menu,
  MenuDivider,
  MenuItem,
  MenuItemWithSubmenu,
} from "@ironcalc/workbook";
import { FileDown, FileUp, Globe, Plus, Table2, Trash2 } from "lucide-react";
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

export function FileMenu(props: {
  newModel: () => void;
  newModelFromTemplate: () => void;
  onDownload: () => void;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
  onDelete: () => void;
  onLanguageChange: (language: string) => void;
  isOpen: boolean;
  onOpen: () => void;
  onClose: () => void;
  onHover: () => void;
}) {
  const [isImportMenuOpen, setImportMenuOpen] = useState(false);
  const [isDeleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [anchorPosition, setAnchorPosition] = useState({ x: 0, y: 0 });
  const triggerRef = useRef<HTMLButtonElement>(null);
  const { t, i18n } = useTranslation();
  const models = getModelsMetadata();
  const selectedUuid = getSelectedUuid();

  const captureAnchor = () => {
    const rect = triggerRef.current?.getBoundingClientRect();
    if (rect) {
      setAnchorPosition({ x: rect.left, y: rect.bottom + 4 });
    }
  };

  return (
    <>
      <Button
        ref={triggerRef}
        variant="ghost"
        id="file-menu-button"
        onClick={() => {
          captureAnchor();
          props.onOpen();
        }}
        onMouseEnter={() => {
          captureAnchor();
          props.onHover();
        }}
        aria-haspopup="menu"
        aria-expanded={props.isOpen ? "true" : "false"}
      >
        {t("file_bar.file_menu.button")}
      </Button>

      <Menu
        open={props.isOpen}
        onClose={props.onClose}
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
        workbookName={models[selectedUuid ?? ""]?.name ?? ""}
      />
    </>
  );
}
