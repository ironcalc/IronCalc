import styled from "@emotion/styled";
import { Menu, MenuItem, Modal } from "@mui/material";
import { Check, FileDown, FileUp, Plus, Trash2 } from "lucide-react";
import { useRef, useState } from "react";
import DeleteWorkbookDialog from "./DeleteWorkbookDialog";
import UploadFileDialog from "./UploadFileDialog";
import { getModelsMetadata, getSelectedUuid } from "./storage";

export function FileMenu(props: {
  newModel: () => void;
  setModel: (key: string) => void;
  onDownload: () => void;
  onModelUpload: (blob: ArrayBuffer, fileName: string) => Promise<void>;
  onDelete: () => void;
}) {
  const [isMenuOpen, setMenuOpen] = useState(false);
  const [isImportMenuOpen, setImportMenuOpen] = useState(false);
  const anchorElement = useRef<HTMLDivElement>(null);
  const models = getModelsMetadata();
  const uuids = Object.keys(models);
  const selectedUuid = getSelectedUuid();
  const [isDeleteDialogOpen, setDeleteDialogOpen] = useState(false);

  const elements = [];
  for (const uuid of uuids) {
    elements.push(
      <MenuItemWrapper
        key={uuid}
        onClick={() => {
          props.setModel(uuid);
          setMenuOpen(false);
        }}
      >
        <CheckIndicator>
          {uuid === selectedUuid ? <StyledCheck /> : ""}
        </CheckIndicator>
        <MenuItemText
          style={{
            maxWidth: "240px",
            overflow: "hidden",
            textOverflow: "ellipsis",
          }}
        >
          {models[uuid]}
        </MenuItemText>
      </MenuItemWrapper>,
    );
  }

  return (
    <>
      <FileMenuWrapper
        onClick={(): void => setMenuOpen(true)}
        ref={anchorElement}
      >
        File
      </FileMenuWrapper>
      <Menu
        open={isMenuOpen}
        onClose={(): void => setMenuOpen(false)}
        anchorEl={anchorElement.current}
        sx={{
          "& .MuiPaper-root": { borderRadius: "8px", padding: "4px 0px" },
          "& .MuiList-root": { padding: "0" },
        }}

        // anchorOrigin={properties.anchorOrigin}
      >
        <MenuItemWrapper
          onClick={() => {
            props.newModel();
            setMenuOpen(false);
          }}
        >
          <StyledPlus />
          <MenuItemText>New</MenuItemText>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={() => {
            setImportMenuOpen(true);
            setMenuOpen(false);
          }}
        >
          <StyledFileUp />
          <MenuItemText>Import</MenuItemText>
        </MenuItemWrapper>
        <MenuItemWrapper>
          <StyledFileDown />
          <MenuItemText onClick={props.onDownload}>
            Download (.xlsx)
          </MenuItemText>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={() => {
            setDeleteDialogOpen(true);
            setMenuOpen(false);
          }}
        >
          <StyledTrash />
          <MenuItemText>Delete workbook</MenuItemText>
        </MenuItemWrapper>
        <MenuDivider />
        {elements}
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

const StyledCheck = styled(Check)`
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

const FileMenuWrapper = styled("div")`
  display: flex;
  align-items: center;
  font-size: 12px;
  font-family: Inter;
  padding: 8px;
  border-radius: 4px;
  cursor: pointer;
  &:hover {
    background-color: #f2f2f2;
  }
`;

const CheckIndicator = styled("span")`
  display: flex;
  justify-content: center;
  min-width: 26px;
`;
