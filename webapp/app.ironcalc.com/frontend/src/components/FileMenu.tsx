import styled from "@emotion/styled";
import { Menu, MenuItem, Modal } from "@mui/material";
import { Check, FileDown, FileUp, Plus, Table2, Trash2 } from "lucide-react";
import { useRef, useState } from "react";
import DeleteWorkbookDialog from "./DeleteWorkbookDialog";
import UploadFileDialog from "./UploadFileDialog";
// import TemplatesDialog from "./WelcomeDialog/TemplatesDialog";
import { getModelsMetadata, getSelectedUuid } from "./storage";

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
  const anchorElement = useRef<HTMLButtonElement>(null);
  const models = getModelsMetadata();
  const selectedUuid = getSelectedUuid();
  const [isDeleteDialogOpen, setDeleteDialogOpen] = useState(false);

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
        File
      </FileMenuWrapper>
      <Menu
        open={isMenuOpen}
        onClose={(): void => setMenuOpen(false)}
        anchorEl={anchorElement.current}
        autoFocus={false}
        disableRestoreFocus={true}
        sx={{
          "& .MuiPaper-root": { borderRadius: "8px", padding: "4px 0px" },
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
          <StyledIcon>
            <Plus />
          </StyledIcon>
          <MenuItemText>New blank workbook</MenuItemText>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={() => {
            props.newModelFromTemplate();
            setMenuOpen(false);
          }}
        >
          <StyledIcon>
            <Table2 />
          </StyledIcon>
          <MenuItemText>New from template</MenuItemText>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={() => {
            setImportMenuOpen(true);
            setMenuOpen(false);
          }}
        >
          <StyledIcon>
            <FileUp />
          </StyledIcon>
          <MenuItemText>Import</MenuItemText>
        </MenuItemWrapper>
        <MenuDivider />
        <MenuItemWrapper onClick={props.onDownload}>
          <StyledIcon>
            <FileDown />
          </StyledIcon>
          <MenuItemText>Download (.xlsx)</MenuItemText>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={() => {
            setDeleteDialogOpen(true);
            setMenuOpen(false);
          }}
        >
          <StyledIcon>
            <Trash2 />
          </StyledIcon>
          <MenuItemText>Delete workbook</MenuItemText>
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

const StyledIcon = styled.div`
  display: flex;
  align-items: center;
  svg {
    width: 16px;
    height: 100%;
    color: #757575;
    padding-right: 10px;
  }
`;

const MenuDivider = styled.div`
  width: 100%;
  margin: auto;
  margin-top: 4px;
  margin-bottom: 4px;
  border-top: 1px solid #eeeeee;
`;

const MenuItemText = styled.div`
  color: #000;
  font-size: 12px;
`;

const MenuItemWrapper = styled(MenuItem)`
  display: flex;
  justify-content: flex-start;
  font-size: 14px;
  width: calc(100% - 8px);
  min-width: 172px;
  margin: 0px 4px;
  border-radius: 4px;
  padding: 8px;
  height: 32px;
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
  background: none;
  &:hover {
    background-color: #f2f2f2;
  }
`;

const CheckIndicator = styled.span`
  display: flex;
  justify-content: center;
  min-width: 26px;
`;
