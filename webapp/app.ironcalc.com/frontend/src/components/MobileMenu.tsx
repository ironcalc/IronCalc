import styled from "@emotion/styled";
import { IconButton, Menu, Modal } from "@mui/material";
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
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import DeleteWorkbookDialog from "./DeleteWorkbookDialog";
import {
  DeleteButton,
  MenuDivider,
  MenuItemText,
  MenuItemWrapper,
} from "./FileMenu";
import { getModelsMetadata, getSelectedUuid } from "./storage";
import UploadFileDialog from "./UploadFileDialog";

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
  const [isMobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [isMobileLanguageMenuOpen, setMobileLanguageMenuOpen] = useState(false);
  const [isMobileImportMenuOpen, setMobileImportMenuOpen] = useState(false);
  const [isMobileDeleteDialogOpen, setMobileDeleteDialogOpen] = useState(false);
  const mobileMenuAnchor = useRef<HTMLButtonElement>(null);
  const mobileLanguageMenuAnchor = useRef<HTMLLIElement>(null);
  const models = getModelsMetadata();
  const selectedUuid = getSelectedUuid();

  const handleMobileMenuClose = () => {
    setMobileMenuOpen(false);
    setMobileLanguageMenuOpen(false);
  };

  const handleMobileLanguageSelect = (language: string) => {
    i18n.changeLanguage(language);
    props.onLanguageChange(language);
    handleMobileMenuClose();
  };

  return (
    <>
      <MobileMenuButton
        ref={mobileMenuAnchor}
        onClick={() => setMobileMenuOpen(true)}
        disableRipple
        $isActive={isMobileMenuOpen}
      >
        <MenuIcon />
      </MobileMenuButton>
      <Menu
        open={isMobileMenuOpen}
        onClose={handleMobileMenuClose}
        anchorEl={mobileMenuAnchor.current}
        autoFocus={false}
        disableRestoreFocus={true}
        transitionDuration={0}
        sx={{
          "& .MuiPaper-root": {
            borderRadius: "8px",
            padding: "4px 0px",
            boxShadow: "0px 4px 12px rgba(0, 0, 0, 0.15)",
          },
          "& .MuiList-root": { padding: "0" },
          transform: "translate(-4px, 4px)",
        }}
        slotProps={{
          list: {
            "aria-labelledby": "mobile-menu-button",
            tabIndex: -1,
          },
        }}
      >
        <MenuItemWrapper
          onClick={() => {
            props.newModel();
            setMobileMenuOpen(false);
          }}
        >
          <Plus />
          {t("file_bar.file_menu.new_blank_workbook")}
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={() => {
            props.newModelFromTemplate();
            setMobileMenuOpen(false);
          }}
        >
          <Table2 />
          {t("file_bar.file_menu.new_from_template")}
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={() => {
            setMobileImportMenuOpen(true);
            setMobileMenuOpen(false);
          }}
        >
          <FileUp />
          {t("file_bar.file_menu.import.button")}
        </MenuItemWrapper>
        <MenuDivider />
        <MenuItemWrapper
          onClick={async () => {
            await props.onDownload();
            setMobileMenuOpen(false);
          }}
        >
          <FileDown />
          <MenuItemText>{t("file_bar.file_menu.download")}</MenuItemText>
        </MenuItemWrapper>
        <DeleteButton
          onClick={() => {
            setMobileDeleteDialogOpen(true);
            setMobileMenuOpen(false);
          }}
        >
          <Trash2 />
          <MenuItemText>
            {t("file_bar.file_menu.delete_workbook.button")}
          </MenuItemText>
        </DeleteButton>
        <MenuDivider />
        <MenuItemWrapper
          ref={mobileLanguageMenuAnchor}
          onClick={() => setMobileLanguageMenuOpen(!isMobileLanguageMenuOpen)}
          sx={{ justifyContent: "space-between" }}
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
            handleMobileMenuClose();
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
            handleMobileMenuClose();
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
      </Menu>
      <Menu
        open={isMobileLanguageMenuOpen}
        anchorEl={mobileLanguageMenuAnchor.current}
        anchorOrigin={{
          vertical: "top",
          horizontal: "right",
        }}
        transformOrigin={{
          vertical: "top",
          horizontal: "left",
        }}
        transitionDuration={0}
        sx={{
          pointerEvents: "none",
          "& .MuiPaper-root": {
            borderRadius: "8px",
            padding: "4px 0px",
            pointerEvents: "auto",
            marginTop: "-4px",
            boxShadow: "0px 4px 12px rgba(0, 0, 0, 0.15)",
          },
          "& .MuiList-root": { padding: "0" },
        }}
        onClose={handleMobileMenuClose}
      >
        <MenuItemWrapper onClick={() => handleMobileLanguageSelect("en-US")}>
          {i18n.language === "en-US" ? (
            <Check size={16} />
          ) : (
            <IconPlaceholder />
          )}
          English (en-US)
        </MenuItemWrapper>
        <MenuItemWrapper onClick={() => handleMobileLanguageSelect("en-GB")}>
          {i18n.language === "en-GB" ? (
            <Check size={16} />
          ) : (
            <IconPlaceholder />
          )}
          English (en-GB)
        </MenuItemWrapper>
        <MenuItemWrapper onClick={() => handleMobileLanguageSelect("es-ES")}>
          {i18n.language === "es-ES" ? (
            <Check size={16} />
          ) : (
            <IconPlaceholder />
          )}
          Español (es-ES)
        </MenuItemWrapper>
        <MenuItemWrapper onClick={() => handleMobileLanguageSelect("fr-FR")}>
          {i18n.language === "fr-FR" ? (
            <Check size={16} />
          ) : (
            <IconPlaceholder />
          )}
          Français (fr-FR)
        </MenuItemWrapper>
        <MenuItemWrapper onClick={() => handleMobileLanguageSelect("de-DE")}>
          {i18n.language === "de-DE" ? (
            <Check size={16} />
          ) : (
            <IconPlaceholder />
          )}
          Deutsch (de-DE)
        </MenuItemWrapper>
        <MenuItemWrapper onClick={() => handleMobileLanguageSelect("it-IT")}>
          {i18n.language === "it-IT" ? (
            <Check size={16} />
          ) : (
            <IconPlaceholder />
          )}
          Italiano (it-IT)
        </MenuItemWrapper>
      </Menu>
      <Modal
        open={isMobileImportMenuOpen}
        onClose={() => setMobileImportMenuOpen(false)}
        aria-labelledby="mobile-import-modal-title"
        aria-describedby="mobile-import-modal-description"
      >
        <UploadFileDialog
          onClose={() => setMobileImportMenuOpen(false)}
          onModelUpload={props.onModelUpload}
        />
      </Modal>
      <Modal
        open={isMobileDeleteDialogOpen}
        onClose={() => setMobileDeleteDialogOpen(false)}
        aria-labelledby="mobile-delete-dialog-title"
        aria-describedby="mobile-delete-dialog-description"
      >
        <DeleteWorkbookDialog
          onClose={() => setMobileDeleteDialogOpen(false)}
          onConfirm={props.onDelete}
          workbookName={(selectedUuid && models[selectedUuid]?.name) || ""}
        />
      </Modal>
    </>
  );
}

const MobileMenuButton = styled(IconButton)<{ $isActive: boolean }>`
  display: flex;
  height: 32px;
  width: 32px;
  padding: 8px;
  border-radius: 4px;
  background-color: ${(props) => (props.$isActive ? "#e6e6e6" : "transparent")};

  svg {
    stroke-width: 2px;
    stroke: #757575;
    width: 16px;
    height: 16px;
  }
  &:hover {
    background-color: ${(props) => (props.$isActive ? "#e6e6e6" : "#f2f2f2")};
  }
  &:active {
    background-color: #e0e0e0;
  }
`;

const IconPlaceholder = styled.div`
  width: 16px;
  min-width: 16px;
`;
