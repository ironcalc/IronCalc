import {
  Check,
  ChevronRight,
  FileDown,
  FileUp,
  Globe,
  Plus,
  Table2,
  Trash2,
} from "lucide-react";
import { type ComponentProps, forwardRef, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import DeleteWorkbookDialog from "../DeleteWorkbookDialog";
import { getModelsMetadata, getSelectedUuid } from "../storage";
import UploadFileDialog from "../UploadFileDialog/UploadFileDialog";
import "./navigation-menus.css";
import { useLanguageSubmenu } from "./useLanguageSubmenu";
import { useMenuAnchor } from "./useMenuAnchor";

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
  const { t, i18n } = useTranslation();
  const models = getModelsMetadata();
  const selectedUuid = getSelectedUuid();

  const langMenu = useLanguageSubmenu();
  const { anchorElement, menuElement, menuStyle } = useMenuAnchor(
    props.isOpen,
    props.onClose,
    [langMenu.menuRef],
  );

  useEffect(() => {
    if (!props.isOpen) langMenu.close();
  }, [props.isOpen, langMenu.close]);

  const handleMainMenuClose = () => {
    props.onClose();
    langMenu.close();
  };

  const handleLanguageItemSelect = (language: string) => {
    i18n.changeLanguage(language);
    props.onLanguageChange(language);
    handleMainMenuClose();
  };

  return (
    <>
      <button
        type="button"
        id="file-menu-button"
        onClick={props.onOpen}
        onMouseEnter={props.onHover}
        ref={anchorElement}
        className={`app-ic-nav-button${props.isOpen ? " is-active" : ""}`}
        aria-haspopup="true"
      >
        {t("file_bar.file_menu.button")}
      </button>

      {props.isOpen && (
        <MenuPaper ref={menuElement} style={menuStyle}>
          <MenuItemWrapper
            onClick={() => {
              props.newModel();
              props.onClose();
            }}
          >
            <Plus />
            {t("file_bar.file_menu.new_blank_workbook")}
          </MenuItemWrapper>
          <MenuItemWrapper
            onClick={() => {
              props.newModelFromTemplate();
              props.onClose();
            }}
          >
            <Table2 />
            {t("file_bar.file_menu.new_from_template")}
          </MenuItemWrapper>
          <MenuItemWrapper
            onClick={() => {
              setImportMenuOpen(true);
              props.onClose();
            }}
          >
            <FileUp />
            {t("file_bar.file_menu.import.button")}
          </MenuItemWrapper>
          <MenuDivider />
          <MenuItemWrapper
            onClick={() => {
              props.onDownload();
              props.onClose();
            }}
          >
            <FileDown />
            <MenuItemText>{t("file_bar.file_menu.download")}</MenuItemText>
          </MenuItemWrapper>
          <DeleteButton
            onClick={() => {
              setDeleteDialogOpen(true);
              props.onClose();
            }}
          >
            <Trash2 />
            <MenuItemText>
              {t("file_bar.file_menu.delete_workbook.button")}
            </MenuItemText>
          </DeleteButton>
          <MenuDivider />
          <MenuItemWrapper
            ref={langMenu.anchorElement}
            onMouseEnter={langMenu.handleMouseEnter}
            onMouseLeave={langMenu.handleMouseLeave}
            className="app-ic-nav-menu-item--space-between"
          >
            <Globe />
            <MenuItemText>
              {t("file_bar.file_menu.default_language")}
            </MenuItemText>
            <ChevronRight size={16} />
          </MenuItemWrapper>
        </MenuPaper>
      )}

      {langMenu.isOpen && (
        <div
          ref={langMenu.menuRef}
          role="menu"
          className="app-ic-nav-submenu"
          style={langMenu.menuStyle}
          onMouseEnter={langMenu.handleMouseEnter}
          onMouseLeave={langMenu.handleMouseLeave}
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
              onClick={() => handleLanguageItemSelect(lang)}
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
        workbookName={models[selectedUuid ?? ""]?.name ?? ""}
      />
    </>
  );
}

export const MenuPaper = forwardRef<HTMLDivElement, ComponentProps<"div">>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      className={`app-ic-nav-menu${className ? ` ${className}` : ""}`}
      {...props}
    />
  ),
);
MenuPaper.displayName = "MenuPaper";

export const MenuItemWrapper = forwardRef<
  HTMLButtonElement,
  ComponentProps<"button">
>(({ className, ...props }, ref) => (
  <button
    ref={ref}
    type="button"
    className={`app-ic-nav-menu-item${className ? ` ${className}` : ""}`}
    {...props}
  />
));
MenuItemWrapper.displayName = "MenuItemWrapper";

export function MenuItemText({ className, ...props }: ComponentProps<"div">) {
  return (
    <div
      className={`app-ic-nav-menu-item-text${className ? ` ${className}` : ""}`}
      {...props}
    />
  );
}

export function MenuDivider() {
  return <div className="app-ic-nav-menu-divider" />;
}

export const DeleteButton = forwardRef<
  HTMLButtonElement,
  ComponentProps<"button">
>(({ className, ...props }, ref) => (
  <button
    ref={ref}
    type="button"
    className={`app-ic-nav-menu-item app-ic-nav-menu-item--delete${className ? ` ${className}` : ""}`}
    {...props}
  />
));
DeleteButton.displayName = "DeleteButton";
