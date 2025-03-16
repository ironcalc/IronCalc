import styled from "@emotion/styled";
import { Button, IconButton, Menu, MenuItem, Modal } from "@mui/material";
import {
  ChevronRight,
  EllipsisVertical,
  FileDown,
  FileUp,
  Plus,
  Trash2,
} from "lucide-react";
import { useRef, useState } from "react";
import DeleteWorkbookDialog from "./DeleteWorkbookDialog";
import UploadFileDialog from "./UploadFileDialog";
import { getModelsMetadata, getSelectedUuid } from "./storage";

export function DesktopMenu(props: {
  newModel: () => void;
  setModel: (key: string) => void;
  onDownload: () => void;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
  onDelete: () => void;
}) {
  const [isFileMenuOpen, setFileMenuOpen] = useState(false);
  const anchorElement = useRef<HTMLButtonElement>(
    null as unknown as HTMLButtonElement,
  );

  return (
    <>
      <FileBarButton
        onClick={(): void => setFileMenuOpen(!isFileMenuOpen)}
        ref={anchorElement}
        disableRipple
        isOpen={isFileMenuOpen}
      >
        File
      </FileBarButton>
      <FileMenu
        newModel={props.newModel}
        setModel={props.setModel}
        onDownload={props.onDownload}
        onModelUpload={props.onModelUpload}
        onDelete={props.onDelete}
        isFileMenuOpen={isFileMenuOpen}
        setFileMenuOpen={setFileMenuOpen}
        setMobileMenuOpen={() => {}}
        anchorElement={anchorElement}
      />
    </>
  );
}

export function MobileMenu(props: {
  newModel: () => void;
  setModel: (key: string) => void;
  onDownload: () => void;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
  onDelete: () => void;
}) {
  const [isMobileMenuOpen, setMobileMenuOpen] = useState(false);
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
        onClick={(): void => setMobileMenuOpen(true)}
        ref={anchorElement}
        disableRipple
      >
        <EllipsisVertical />
      </MenuButton>
      <StyledMenu
        open={isMobileMenuOpen}
        onClose={(): void => setMobileMenuOpen(false)}
        anchorEl={anchorElement.current}
      >
        <MenuItemWrapper
          onClick={(event) => {
            setFileMenuOpen(true);
            setFileMenuAnchorEl(event.currentTarget);
          }}
          disableRipple
        >
          <MenuItemText>File</MenuItemText>
          <ChevronRight />
        </MenuItemWrapper>
        <MenuDivider />
        <MenuItemWrapper
          onClick={() => {
            window.open("https://docs.ironcalc.com", "_blank");
            setMobileMenuOpen(false);
          }}
          disableRipple
        >
          <MenuItemText>Help</MenuItemText>
        </MenuItemWrapper>
      </StyledMenu>
      <FileMenu
        newModel={props.newModel}
        setModel={props.setModel}
        onDownload={props.onDownload}
        onModelUpload={props.onModelUpload}
        onDelete={props.onDelete}
        isFileMenuOpen={isFileMenuOpen}
        setFileMenuOpen={setFileMenuOpen}
        setMobileMenuOpen={setMobileMenuOpen}
        anchorElement={anchorElement}
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
  setMobileMenuOpen: (open: boolean) => void;
  anchorElement: React.RefObject<HTMLButtonElement>;
}) {
  const [isImportMenuOpen, setImportMenuOpen] = useState(false);
  const models = getModelsMetadata();
  const selectedUuid = getSelectedUuid();
  const [isDeleteDialogOpen, setDeleteDialogOpen] = useState(false);

  return (
    <>
      <StyledMenu
        open={props.isFileMenuOpen}
        onClose={(): void => props.setFileMenuOpen(false)}
        anchorEl={props.anchorElement.current}
        anchorOrigin={{
          vertical: "bottom",
          horizontal: "left",
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
      >
        <MenuItemWrapper
          onClick={() => {
            props.newModel();
            props.setFileMenuOpen(false);
            props.setMobileMenuOpen(false);
          }}
          disableRipple
        >
          <StyledPlus />
          <MenuItemText>New</MenuItemText>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={() => {
            setImportMenuOpen(true);
            props.setFileMenuOpen(false);
            props.setMobileMenuOpen(false);
          }}
          disableRipple
        >
          <StyledFileUp />
          <MenuItemText>Import</MenuItemText>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={() => {
            props.onDownload();
            props.setMobileMenuOpen(false);
          }}
          disableRipple
        >
          <StyledFileDown />
          <MenuItemText>Download (.xlsx)</MenuItemText>
        </MenuItemWrapper>
        <MenuDivider />
        <MenuItemWrapper
          onClick={() => {
            setDeleteDialogOpen(true);
            props.setFileMenuOpen(false);
            props.setMobileMenuOpen(false);
          }}
          disableRipple
        >
          <StyledTrash />
          <MenuItemText>Delete workbook</MenuItemText>
        </MenuItemWrapper>
      </StyledMenu>
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
          workbookName={selectedUuid ? models[selectedUuid]?.name || "" : ""}
        />
      </Modal>
    </>
  );
}

const MenuButton = styled(IconButton)`
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
  &:hover {
    background-color: #f2f2f2;
  }
  &:active {
    background-color: #e0e0e0;
  }
`;

const FileBarButton = styled(Button)<{ isOpen: boolean }>`
  display: flex;
  flex-direction: row;
  align-items: center;
  font-size: 12px;
  height: 32px;
  width: auto;
  padding: 4px 8px;
  font-weight: 400;
  min-width: 0px;
  text-transform: capitalize;
  color: #333333;
  background-color: ${({ isOpen }) => (isOpen ? "#f2f2f2" : "none")};
  &:hover {
    background-color: #f2f2f2;
  }
  &:active {
    background-color: #e0e0e0;
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
  min-height: 32px;
  svg {
    width: 16px;
    height: 16px;
  }
`;

const StyledMenu = styled(Menu)`
  .MuiPaper-root {
    border-radius: 8px;
    padding: 4px 0px;
  },
  .MuiList-root {
    padding: 0;
  },
`;
