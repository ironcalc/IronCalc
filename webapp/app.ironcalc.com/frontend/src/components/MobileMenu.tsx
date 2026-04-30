import "./mobile-menu.css";
import { IconButton } from "@ironcalc/workbook";
import {
  BookOpen,
  Check,
  ChevronDown,
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
import { useState } from "react";
import { useTranslation } from "react-i18next";
import DeleteWorkbookDialog from "./DeleteWorkbookDialog";
import {
  DeleteButton,
  MenuDivider,
  MenuItemText,
  MenuItemWrapper,
} from "./Navigation/FileMenu";
import { getModelsMetadata, getSelectedUuid } from "./storage";
import UploadFileDialog from "./UploadFileDialog/UploadFileDialog";

interface MobileMenuProps {
  newModel: () => void;
  newModelFromTemplate: () => void;
  onDownload: () => Promise<void>;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
  onDelete: () => void;
  onLanguageChange: (language: string) => void;
}

const LANGUAGES = [
  { code: "en-US", label: "English (en-US)" },
  { code: "en-GB", label: "English (en-GB)" },
  { code: "es-ES", label: "Español (es-ES)" },
  { code: "fr-FR", label: "Français (fr-FR)" },
  { code: "de-DE", label: "Deutsch (de-DE)" },
  { code: "it-IT", label: "Italiano (it-IT)" },
];

export function MobileMenu(props: MobileMenuProps) {
  const { t, i18n } = useTranslation();
  const [isMobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [isMobileLanguageMenuOpen, setMobileLanguageMenuOpen] = useState(false);
  const [isMobileImportMenuOpen, setMobileImportMenuOpen] = useState(false);
  const [isMobileDeleteDialogOpen, setMobileDeleteDialogOpen] = useState(false);
  const models = getModelsMetadata();
  const selectedUuid = getSelectedUuid();

  const handleClose = () => {
    setMobileMenuOpen(false);
    setMobileLanguageMenuOpen(false);
  };

  const handleLanguageSelect = (language: string) => {
    i18n.changeLanguage(language);
    props.onLanguageChange(language);
    handleClose();
  };

  return (
    <>
      <div className="mobile-menu-wrapper">
        <IconButton
          icon={<MenuIcon />}
          aria-label={t("file_bar.file_menu.button")}
          size="md"
          variant="ghost"
          className={`mobile-menu-trigger${isMobileMenuOpen ? " is-active" : ""}`}
          onClick={() => setMobileMenuOpen(true)}
        />
        {isMobileMenuOpen && (
          <>
            {/* biome-ignore lint/a11y/noStaticElementInteractions: backdrop */}
            <div className="mobile-menu-backdrop" onMouseDown={handleClose} />
            <div className="mobile-menu-panel">
              <MenuItemWrapper
                onClick={() => {
                  props.newModel();
                  handleClose();
                }}
              >
                <Plus />
                {t("file_bar.file_menu.new_blank_workbook")}
              </MenuItemWrapper>
              <MenuItemWrapper
                onClick={() => {
                  props.newModelFromTemplate();
                  handleClose();
                }}
              >
                <Table2 />
                {t("file_bar.file_menu.new_from_template")}
              </MenuItemWrapper>
              <MenuItemWrapper
                onClick={() => {
                  setMobileImportMenuOpen(true);
                  handleClose();
                }}
              >
                <FileUp />
                {t("file_bar.file_menu.import.button")}
              </MenuItemWrapper>
              <MenuDivider />
              <MenuItemWrapper
                onClick={async () => {
                  await props.onDownload();
                  handleClose();
                }}
              >
                <FileDown />
                <MenuItemText>{t("file_bar.file_menu.download")}</MenuItemText>
              </MenuItemWrapper>
              <DeleteButton
                onClick={() => {
                  setMobileDeleteDialogOpen(true);
                  handleClose();
                }}
              >
                <Trash2 />
                <MenuItemText>
                  {t("file_bar.file_menu.delete_workbook.button")}
                </MenuItemText>
              </DeleteButton>
              <MenuDivider />
              <MenuItemWrapper
                onClick={() =>
                  setMobileLanguageMenuOpen(!isMobileLanguageMenuOpen)
                }
                style={{ justifyContent: "space-between" }}
              >
                <Globe />
                <MenuItemText>
                  {t("file_bar.file_menu.default_language")}
                </MenuItemText>
                {isMobileLanguageMenuOpen ? (
                  <ChevronDown size={16} />
                ) : (
                  <ChevronRight size={16} />
                )}
              </MenuItemWrapper>
              {isMobileLanguageMenuOpen && (
                <div className="mobile-menu-language-submenu">
                  {LANGUAGES.map(({ code, label }) => (
                    <MenuItemWrapper
                      key={code}
                      onClick={() => handleLanguageSelect(code)}
                    >
                      {i18n.language === code ? (
                        <Check size={16} />
                      ) : (
                        <div className="nav-menu-icon-placeholder" />
                      )}
                      {label}
                    </MenuItemWrapper>
                  ))}
                </div>
              )}
              <MenuDivider />
              <MenuItemWrapper
                onClick={() => {
                  handleClose();
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
                  handleClose();
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
            </div>
          </>
        )}
      </div>
      {isMobileImportMenuOpen && (
        <UploadFileDialog
          onClose={() => setMobileImportMenuOpen(false)}
          onModelUpload={props.onModelUpload}
        />
      )}
      <DeleteWorkbookDialog
        open={isMobileDeleteDialogOpen}
        onClose={() => setMobileDeleteDialogOpen(false)}
        onConfirm={props.onDelete}
        workbookName={(selectedUuid && models[selectedUuid]?.name) || ""}
      />
    </>
  );
}
