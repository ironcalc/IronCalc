import styled from "@emotion/styled";
import { IconButton, Menu, MenuItem, Modal } from "@mui/material";
import {
  ChevronRight,
  Ellipsis,
  FileDown,
  FileUp,
  Plus,
  Trash2,
} from "lucide-react";
import { useRef, useState } from "react";
import DeleteWorkbookDialog from "./DeleteWorkbookDialog";
import UploadFileDialog from "./UploadFileDialog";
import { getModelsMetadata, getSelectedUuid } from "./storage";

export function ParentMenu(props: {
  newModel: () => void;
  setModel: (key: string) => void;
  onDownload: () => void;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
  onDelete: () => void;
}) {
  const [isParentMenuOpen, setParentMenuOpen] = useState(false);
  const [isFileMenuOpen, setFileMenuOpen] = useState(false);
  const anchorElement = useRef<HTMLButtonElement>(
    null as unknown as HTMLButtonElement,
  );
  const [fileMenuAnchorEl, setFileMenuAnchorEl] = useState<HTMLElement | null>(
    null,
  );

  return (
    <>
      <MenuButton
        onClick={(): void => setParentMenuOpen(true)}
        ref={anchorElement}
      >
        <Ellipsis />
      </MenuButton>
      <Menu
        open={isParentMenuOpen}
        onClose={(): void => setParentMenuOpen(false)}
        anchorEl={anchorElement.current}
        sx={{
          "& .MuiPaper-root": { borderRadius: "8px", padding: "4px 0px" },
          "& .MuiList-root": { padding: "0" },
        }}
      >
        <MenuItemWrapper
          onMouseEnter={(event) => {
            setFileMenuOpen(true);
            setFileMenuAnchorEl(event.currentTarget);
          }}
          onClick={(event) => {
            // Keep current click behavior as fallback
            setFileMenuOpen(true);
            setFileMenuAnchorEl(event.currentTarget);
          }}
        >
          <MenuItemText>File</MenuItemText>
          <ChevronRight />
        </MenuItemWrapper>
        <MenuDivider />
        <MenuItemWrapper
          onClick={() => {
            window.open("https://docs.ironcalc.com", "_blank");
            setParentMenuOpen(false);
          }}
        >
          <MenuItemText>Help</MenuItemText>
        </MenuItemWrapper>
      </Menu>
      <FileMenu
        newModel={props.newModel}
        setModel={props.setModel}
        onDownload={props.onDownload}
        onModelUpload={props.onModelUpload}
        onDelete={props.onDelete}
        isFileMenuOpen={isFileMenuOpen}
        setFileMenuOpen={setFileMenuOpen}
        setParentMenuOpen={setParentMenuOpen} // Pass setParentMenuOpen as a prop
        anchorElement={anchorElement}
        fileMenuAnchorEl={fileMenuAnchorEl}
      />
    </>
  );
}

export function FileMenu(props: {
  newModel: () => void;
  setModel: (key: string) => void;
  onDownload: () => void;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
  onDelete: () => void;
  isFileMenuOpen: boolean;
  setFileMenuOpen: (open: boolean) => void;
  setParentMenuOpen: (open: boolean) => void; // Add setParentMenuOpen to props
  anchorElement: React.RefObject<HTMLButtonElement>;
  fileMenuAnchorEl: HTMLElement | null;
}) {
  const [isImportMenuOpen, setImportMenuOpen] = useState(false);
  const models = getModelsMetadata();
  const selectedUuid = getSelectedUuid();
  const [isDeleteDialogOpen, setDeleteDialogOpen] = useState(false);

  return (
    <>
      <Menu
        open={props.isFileMenuOpen}
        onClose={(): void => props.setFileMenuOpen(false)}
        anchorEl={props.fileMenuAnchorEl || props.anchorElement.current}
        anchorOrigin={{
          vertical: "top",
          horizontal: "right",
        }}
        transformOrigin={{
          vertical: "top",
          horizontal: "left",
        }}
        // To prevent closing parent menu when interacting with submenu
        onMouseLeave={() => {
          if (!isImportMenuOpen && !isDeleteDialogOpen) {
            props.setFileMenuOpen(false);
          }
        }}
        sx={{
          "& .MuiPaper-root": { borderRadius: "8px", padding: "4px 0px" },
          "& .MuiList-root": { padding: "0" },
        }}
      >
        <MenuItemWrapper
          onClick={() => {
            props.newModel();
            props.setFileMenuOpen(false);
            props.setParentMenuOpen(false);
          }}
        >
          <StyledPlus />
          <MenuItemText>New</MenuItemText>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={() => {
            setImportMenuOpen(true);
            props.setFileMenuOpen(false);
            props.setParentMenuOpen(false);
          }}
        >
          <StyledFileUp />
          <MenuItemText>Import</MenuItemText>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={() => {
            props.onDownload();
            props.setParentMenuOpen(false);
          }}
        >
          <StyledFileDown />
          <MenuItemText>Download (.xlsx)</MenuItemText>
        </MenuItemWrapper>
        <MenuDivider />
        <MenuItemWrapper
          onClick={() => {
            setDeleteDialogOpen(true);
            props.setFileMenuOpen(false);
            props.setParentMenuOpen(false);
          }}
        >
          <StyledTrash />
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
          workbookName={selectedUuid ? models[selectedUuid] : ""}
        />
      </Modal>
    </>
  );
}

const MenuButton = styled(IconButton)`
  margin-left: 8px;
  height: 32px;
  width: 32px;
  padding: 8px;
  border-radius: 4px;
  svg {
    stroke-width: 2px;
    stroke: #757575;
    width: 16px;
    height: 16px;
  }
`;

const StyledPlus = styled(Plus)`
  width: 16px;
  height: 16px;
  color: #333333;
  padding-right: 10px;
`;

const StyledFileDown = styled(FileDown)`
  width: 16px;
  height: 16px;
  color: #333333;
  padding-right: 10px;
`;

const StyledFileUp = styled(FileUp)`
  width: 16px;
  height: 16px;
  color: #333333;
  padding-right: 10px;
`;

const StyledTrash = styled(Trash2)`
  width: 16px;
  height: 16px;
  color: #333333;
  padding-right: 10px;
`;

const MenuDivider = styled("div")`
  width: 100%;
  margin: auto;
  margin-top: 4px;
  margin-bottom: 4px;
  border-top: 1px solid #eeeeee;
`;

const MenuItemText = styled("div")`
  color: #000;
  font-size: 12px;
  flex-grow: 1;
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
  svg {
    width: 16px;
    height: 16px;
  }
`;
