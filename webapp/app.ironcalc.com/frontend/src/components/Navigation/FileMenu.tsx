import "./navigation.css";
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
import {
  type ButtonHTMLAttributes,
  forwardRef,
  type ReactNode,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import DeleteWorkbookDialog from "../DeleteWorkbookDialog";
import { getModelsMetadata, getSelectedUuid } from "../storage";
import UploadFileDialog from "../UploadFileDialog/UploadFileDialog";

export function FileMenu(props: {
  newModel: () => void;
  newModelFromTemplate: () => void;
  setModel: (key: string) => void;
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
  const models = getModelsMetadata();
  const selectedUuid = getSelectedUuid();
  const { t, i18n } = useTranslation();

  const handleLanguageSelect = (language: string) => {
    i18n.changeLanguage(language);
    props.onLanguageChange(language);
    props.onClose();
  };

  const languages = [
    { code: "en-US", label: "English (en-US)" },
    { code: "en-GB", label: "English (en-GB)" },
    { code: "es-ES", label: "Español (es-ES)" },
    { code: "fr-FR", label: "Français (fr-FR)" },
    { code: "de-DE", label: "Deutsch (de-DE)" },
    { code: "it-IT", label: "Italiano (it-IT)" },
  ];

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
        {t("file_bar.file_menu.button")}
      </button>
      {props.isOpen && (
        <div className="nav-menu-panel">
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
          <div className="nav-menu-item nav-menu-item--has-submenu">
            <Globe />
            <MenuItemText>
              {t("file_bar.file_menu.default_language")}
            </MenuItemText>
            <ChevronRight size={16} />
            <div className="nav-menu-submenu">
              {languages.map(({ code, label }) => (
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
          </div>
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
        workbookName={selectedUuid ? models[selectedUuid].name : ""}
      />
    </div>
  );
}

type MenuItemProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  children: ReactNode;
};

export const MenuItemWrapper = forwardRef<HTMLButtonElement, MenuItemProps>(
  function MenuItemWrapper({ children, className, ...rest }, ref) {
    return (
      <button
        ref={ref}
        type="button"
        className={["nav-menu-item", className].filter(Boolean).join(" ")}
        {...rest}
      >
        {children}
      </button>
    );
  },
);

export function DeleteButton({ children, className, ...rest }: MenuItemProps) {
  return (
    <MenuItemWrapper
      className={["nav-menu-item--delete", className].filter(Boolean).join(" ")}
      {...rest}
    >
      {children}
    </MenuItemWrapper>
  );
}

export function MenuDivider() {
  return <hr className="nav-menu-divider" />;
}

export function MenuItemText({ children }: { children: ReactNode }) {
  return <div className="nav-menu-item-text">{children}</div>;
}

export function MenuPaper({ children }: { children: ReactNode }) {
  return <div className="nav-menu-panel">{children}</div>;
}
