import styled from "@emotion/styled";
import { Menu, MenuItem, Modal } from "@mui/material";
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
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import DeleteWorkbookDialog from "./DeleteWorkbookDialog";
import { getModelsMetadata, getSelectedUuid } from "./storage";
import UploadFileDialog from "./UploadFileDialog";

export function FileMenu(props: {
  newModel: () => void;
  newModelFromTemplate: () => void;
  setModel: (key: string) => void;
  onDownload: () => void;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
  onDelete: () => void;
}) {
  const [isMenuOpen, setMenuOpen] = useState(false);
  const [isImportMenuOpen, setImportMenuOpen] = useState(false);
  const [isLanguageMenuOpen, setLanguageMenuOpen] = useState(false);
  const anchorElement = useRef<HTMLButtonElement>(null);
  const languageMenuAnchor = useRef<HTMLLIElement>(null);
  const models = getModelsMetadata();
  const selectedUuid = getSelectedUuid();
  const [isDeleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const { t, i18n } = useTranslation();

  const handleMainMenuClose = () => {
    setMenuOpen(false);
    setLanguageMenuOpen(false);
  };

  const handleLanguageItemSelect = (language: string) => {
    i18n.changeLanguage(language);
    handleMainMenuClose();
  };

  return (
    <>
      <FileMenuWrapper
        type="button"
        id="file-menu-button"
        onClick={(): void => setMenuOpen(true)}
        ref={anchorElement}
        $isActive={isMenuOpen}
        aria-haspopup="true"
      >
        {t("file_bar.file_menu.button")}
      </FileMenuWrapper>
      <Menu
        open={isMenuOpen}
        onClose={handleMainMenuClose}
        anchorEl={anchorElement.current}
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
            "aria-labelledby": "file-menu-button",
            tabIndex: -1,
          },
        }}
      >
        <MenuItemWrapper
          onClick={() => {
            props.newModel();
            setMenuOpen(false);
          }}
        >
          <Plus />
          {t("file_bar.file_menu.new_blank_workbook")}
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={() => {
            props.newModelFromTemplate();
            setMenuOpen(false);
          }}
        >
          <Table2 />
          {t("file_bar.file_menu.new_from_template")}
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={() => {
            setImportMenuOpen(true);
            setMenuOpen(false);
          }}
        >
          <FileUp />
          {t("file_bar.file_menu.import.button")}
        </MenuItemWrapper>
        <MenuDivider />
        <MenuItemWrapper onClick={props.onDownload}>
          <FileDown />
          <MenuItemText>{t("file_bar.file_menu.download")}</MenuItemText>
        </MenuItemWrapper>
        <DeleteButton
          onClick={() => {
            setDeleteDialogOpen(true);
            setMenuOpen(false);
          }}
        >
          <Trash2 />
          <MenuItemText>
            {t("file_bar.file_menu.delete_workbook.button")}
          </MenuItemText>
        </DeleteButton>
        <MenuDivider />
        <MenuItemWrapper
          ref={languageMenuAnchor}
          onMouseEnter={() => setLanguageMenuOpen(true)}
          onMouseLeave={() => setLanguageMenuOpen(false)}
          sx={{ justifyContent: "space-between" }}
        >
          <Globe />
          <MenuItemText>
            {t("file_bar.file_menu.display_language")}
          </MenuItemText>
          <ChevronRight size={16} />
        </MenuItemWrapper>
      </Menu>
      <Menu
        open={isLanguageMenuOpen}
        anchorEl={languageMenuAnchor.current}
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
        onClose={handleMainMenuClose}
        slotProps={{
          paper: {
            onMouseEnter: () => setLanguageMenuOpen(true),
            onMouseLeave: () => setLanguageMenuOpen(false),
          },
        }}
      >
        <MenuItemWrapper onClick={() => handleLanguageItemSelect("en-US")}>
          {i18n.language === "en-US" ? (
            <Check size={16} />
          ) : (
            <IconPlaceholder />
          )}
          English
        </MenuItemWrapper>
        <MenuItemWrapper onClick={() => handleLanguageItemSelect("es-ES")}>
          {i18n.language === "es-ES" ? (
            <Check size={16} />
          ) : (
            <IconPlaceholder />
          )}
          Español
        </MenuItemWrapper>
        <MenuItemWrapper onClick={() => handleLanguageItemSelect("fr-FR")}>
          {i18n.language === "fr-FR" ? (
            <Check size={16} />
          ) : (
            <IconPlaceholder />
          )}
          Français
        </MenuItemWrapper>
        <MenuItemWrapper onClick={() => handleLanguageItemSelect("de-DE")}>
          {i18n.language === "de-DE" ? (
            <Check size={16} />
          ) : (
            <IconPlaceholder />
          )}
          Deutsch
        </MenuItemWrapper>
        <MenuItemWrapper onClick={() => handleLanguageItemSelect("it-IT")}>
          {i18n.language === "it-IT" ? (
            <Check size={16} />
          ) : (
            <IconPlaceholder />
          )}
          Italiano
        </MenuItemWrapper>
      </Menu>
      <Modal
        open={isImportMenuOpen}
        onClose={() => {
          setImportMenuOpen(false);
        }}
        aria-labelledby="modal-modal-title"
        aria-describedby="modal-modal-description"
      >
        <UploadFileDialog
          onClose={() => {
            setImportMenuOpen(false);
          }}
          onModelUpload={props.onModelUpload}
        />
      </Modal>
      <Modal
        open={isDeleteDialogOpen}
        onClose={() => setDeleteDialogOpen(false)}
        aria-labelledby="delete-dialog-title"
        aria-describedby="delete-dialog-description"
      >
        <DeleteWorkbookDialog
          onClose={() => setDeleteDialogOpen(false)}
          onConfirm={props.onDelete}
          workbookName={selectedUuid ? models[selectedUuid].name : ""}
        />
      </Modal>
    </>
  );
}

export const MenuDivider = styled.div`
  width: 100%;
  margin: auto;
  margin-top: 4px;
  margin-bottom: 4px;
  border-top: 1px solid #eeeeee;
`;

export const MenuItemWrapper = styled(MenuItem)`
  display: flex;
  justify-content: flex-start;
  font-size: 14px;
  width: calc(100% - 8px);
  min-width: 172px;
  margin: 0px 4px;
  border-radius: 4px;
  padding: 8px;
  height: 32px;
  color: #000;
  font-size: 12px;
  gap: 8px;
  svg {
    max-width: 16px;
    min-width: 16px;
    max-height: 16px;
    min-height: 16px;
    color: #757575;
  }
`;

export const MenuItemText = styled("div")`
  width: 100%;
`;

const IconPlaceholder = styled.div`
  width: 16px;
  min-width: 16px;
`;

const FileMenuWrapper = styled.button<{ $isActive: boolean }>`
  display: flex;
  align-items: center;
  font-size: 12px;
  font-family: Inter;
  padding: 8px;
  border-radius: 4px;
  cursor: pointer;
  background-color: ${(props) => (props.$isActive ? "#e6e6e6" : "transparent")};
  border: none;
  &:hover {
    background-color: #f2f2f2;
  }
`;

export const DeleteButton = styled(MenuItemWrapper)`
  color: #EB5757;
  svg {
    color: #EB5757;
  }
  &:hover {
    background-color: #EB57571A;
  }
  &:active {
    background-color: #EB57571A;
  }
`;
