import styled from "@emotion/styled";
import { Menu, MenuItem, Modal } from "@mui/material";
import { FileDown, FileUp, Plus, Table2, Trash2 } from "lucide-react";
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
          <Plus />
          New blank workbook
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={() => {
            props.newModelFromTemplate();
            setMenuOpen(false);
          }}
        >
          <Table2 />
          New from template
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={() => {
            setImportMenuOpen(true);
            setMenuOpen(false);
          }}
        >
          <FileUp />
          Import
        </MenuItemWrapper>
        <MenuDivider />
        <MenuItemWrapper onClick={props.onDownload}>
          <FileDown />
          Download (.xlsx)
        </MenuItemWrapper>
        <DeleteButton
          onClick={() => {
            setDeleteDialogOpen(true);
            setMenuOpen(false);
          }}
        >
          <Trash2 />
          Delete workbook
        </DeleteButton>
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
    width: 16px;
    height: 100%;
    color: #757575;
  }
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
