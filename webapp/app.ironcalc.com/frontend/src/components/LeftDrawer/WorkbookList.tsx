import styled from "@emotion/styled";
import { Menu, MenuItem, Modal } from "@mui/material";
import {
  Copy,
  EllipsisVertical,
  FileDown,
  Pin,
  PinOff,
  Table2,
  Trash2,
} from "lucide-react";
import type React from "react";
import { useEffect, useState } from "react";
import DeleteWorkbookDialog from "../DeleteWorkbookDialog";
import { DeleteButton, MenuDivider, MenuItemWrapper } from "../FileMenu";
import { downloadModel } from "../rpc";
import {
  duplicateModel,
  getModelsMetadata,
  getSelectedUuid,
  isWorkbookPinned,
  selectModelFromStorage,
  togglePinWorkbook,
} from "../storage";

interface WorkbookListProps {
  setModel: (key: string) => void;
  onDelete: (uuid: string) => void;
}

function WorkbookList({ setModel, onDelete }: WorkbookListProps) {
  const [menuAnchorEl, setMenuAnchorEl] = useState<null | HTMLElement>(null);
  const [selectedWorkbookUuid, setSelectedWorkbookUuid] = useState<
    string | null
  >(null);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [workbookToDelete, setWorkbookToDelete] = useState<string | null>(null);
  const [intendedSelection, setIntendedSelection] = useState<string | null>(
    null,
  );

  const selectedUuid = getSelectedUuid();

  // Clear intended selection when selectedUuid changes from outside
  useEffect(() => {
    if (intendedSelection && selectedUuid === intendedSelection) {
      setIntendedSelection(null);
    }
  }, [selectedUuid, intendedSelection]);

  const handleMenuOpen = (
    event: React.MouseEvent<HTMLButtonElement>,
    uuid: string,
  ) => {
    console.log("Menu open", uuid);
    event.stopPropagation();
    setSelectedWorkbookUuid(uuid);
    setMenuAnchorEl(event.currentTarget);
    setIntendedSelection(uuid);
    setModel(uuid);
  };

  const handleMenuClose = () => {
    console.log(
      "Menu closing, selectedWorkbookUuid:",
      selectedWorkbookUuid,
      "intendedSelection:",
      intendedSelection,
    );
    setMenuAnchorEl(null);
    // If we have an intended selection, make sure it's still selected
    if (intendedSelection && intendedSelection !== selectedUuid) {
      console.log("Re-selecting intended workbook:", intendedSelection);
      setModel(intendedSelection);
    }
    // Don't reset selectedWorkbookUuid here - we want to keep track of which workbook was selected
    // The selectedWorkbookUuid will be used for download/delete operations
  };

  const handleDeleteClick = (uuid: string) => {
    console.log("Delete workbook:", uuid);
    setWorkbookToDelete(uuid);
    setIsDeleteDialogOpen(true);
    setIntendedSelection(null);
    handleMenuClose();
  };

  const handleDeleteConfirm = () => {
    if (workbookToDelete) {
      onDelete(workbookToDelete);
      setWorkbookToDelete(null);
    }
    setIsDeleteDialogOpen(false);
  };

  const handleDeleteCancel = () => {
    setWorkbookToDelete(null);
    setIsDeleteDialogOpen(false);
  };

  const handleDownload = async (uuid: string) => {
    try {
      const model = selectModelFromStorage(uuid);
      if (model) {
        const bytes = model.toBytes();
        const fileName = model.getName();
        await downloadModel(bytes, fileName);
      }
    } catch (error) {
      console.error("Failed to download workbook:", error);
    }
  };

  const handlePinToggle = (uuid: string) => {
    togglePinWorkbook(uuid);
    setIntendedSelection(null);
    handleMenuClose();
  };

  const handleDuplicate = (uuid: string) => {
    const duplicatedModel = duplicateModel(uuid);
    if (duplicatedModel) {
      setIntendedSelection(null);
      handleMenuClose();
    }
  };

  // Group workbooks by pinned status and creation date
  const groupWorkbooks = () => {
    const now = Date.now();
    const millisecondsInDay = 24 * 60 * 60 * 1000;
    const millisecondsIn30Days = 30 * millisecondsInDay;

    const pinnedModels = [];
    const modelsCreatedToday = [];
    const modelsCreatedThisMonth = [];
    const olderModels = [];
    const modelsMetadata = getModelsMetadata();

    for (const uuid in modelsMetadata) {
      const createdAt = modelsMetadata[uuid].createdAt;
      const age = now - createdAt;

      if (modelsMetadata[uuid].pinned) {
        pinnedModels.push(uuid);
      } else if (age < millisecondsInDay) {
        modelsCreatedToday.push(uuid);
      } else if (age < millisecondsIn30Days) {
        modelsCreatedThisMonth.push(uuid);
      } else {
        olderModels.push(uuid);
      }
    }

    // Sort each group by creation timestamp (newest first)
    const sortByNewest = (uuids: string[]) =>
      uuids.sort(
        (a, b) => modelsMetadata[b].createdAt - modelsMetadata[a].createdAt,
      );

    return {
      pinnedModels: sortByNewest(pinnedModels),
      modelsCreatedToday: sortByNewest(modelsCreatedToday),
      modelsCreatedThisMonth: sortByNewest(modelsCreatedThisMonth),
      olderModels: sortByNewest(olderModels),
    };
  };

  const {
    pinnedModels,
    modelsCreatedToday,
    modelsCreatedThisMonth,
    olderModels,
  } = groupWorkbooks();

  const renderWorkbookItem = (uuid: string) => {
    const isMenuOpen = menuAnchorEl !== null && selectedWorkbookUuid === uuid;
    const isAnyMenuOpen = menuAnchorEl !== null;
    const models = getModelsMetadata();
    return (
      <WorkbookListItem
        key={uuid}
        onClick={() => {
          // Prevent clicking on list items when any menu is open
          if (isAnyMenuOpen) {
            return;
          }
          setModel(uuid);
        }}
        selected={uuid === selectedUuid}
        disableRipple
        style={{ pointerEvents: isAnyMenuOpen ? "none" : "auto" }}
      >
        <StorageIndicator>
          <Table2 />
        </StorageIndicator>
        <WorkbookListText>{models[uuid].name}</WorkbookListText>
        <EllipsisButton
          onClick={(e) => handleMenuOpen(e, uuid)}
          isOpen={isMenuOpen}
          onMouseDown={(e) => e.stopPropagation()}
          style={{ pointerEvents: "auto" }}
        >
          <EllipsisVertical />
        </EllipsisButton>
      </WorkbookListItem>
    );
  };

  const renderSection = (title: string, uuids: string[]) => {
    if (uuids.length === 0) return null;

    return (
      <SectionContainer key={title}>
        <SectionTitle>
          {title === "Pinned" && <Pin />}
          {title}
        </SectionTitle>
        {uuids.map(renderWorkbookItem)}
      </SectionContainer>
    );
  };

  const models = getModelsMetadata();

  return (
    <>
      {renderSection("Pinned", pinnedModels)}
      {renderSection("Today", modelsCreatedToday)}
      {renderSection("Last 30 Days", modelsCreatedThisMonth)}
      {renderSection("Older", olderModels)}

      <StyledMenu
        anchorEl={menuAnchorEl}
        open={Boolean(menuAnchorEl)}
        onClose={handleMenuClose}
        MenuListProps={{
          dense: true,
        }}
        anchorOrigin={{
          vertical: "bottom",
          horizontal: "right",
        }}
        transformOrigin={{
          vertical: "top",
          horizontal: "right",
        }}
      >
        <MenuItemWrapper
          onClick={() => {
            console.log(
              "Download clicked, selectedWorkbookUuid:",
              selectedWorkbookUuid,
            );
            if (selectedWorkbookUuid) {
              handleDownload(selectedWorkbookUuid);
            }
            setIntendedSelection(null);
            handleMenuClose();
          }}
          disableRipple
        >
          <FileDown />
          Download (.xlsx)
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={() => {
            if (selectedWorkbookUuid) {
              handlePinToggle(selectedWorkbookUuid);
            }
          }}
          disableRipple
        >
          {selectedWorkbookUuid && isWorkbookPinned(selectedWorkbookUuid) ? (
            <PinOff />
          ) : (
            <Pin />
          )}
          {selectedWorkbookUuid && isWorkbookPinned(selectedWorkbookUuid)
            ? "Unpin"
            : "Pin"}
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={() => {
            if (selectedWorkbookUuid) {
              handleDuplicate(selectedWorkbookUuid);
            }
          }}
          disableRipple
        >
          <Copy />
          Duplicate
        </MenuItemWrapper>
        <MenuDivider />
        <DeleteButton
          selected={false}
          onClick={() => {
            if (selectedWorkbookUuid) {
              handleDeleteClick(selectedWorkbookUuid);
            }
          }}
          disableRipple
        >
          <Trash2 size={16} />
          Delete workbook
        </DeleteButton>
      </StyledMenu>

      <Modal
        open={isDeleteDialogOpen}
        onClose={handleDeleteCancel}
        aria-labelledby="delete-dialog-title"
        aria-describedby="delete-dialog-description"
      >
        <DeleteWorkbookDialog
          onClose={handleDeleteCancel}
          onConfirm={handleDeleteConfirm}
          workbookName={workbookToDelete ? models[workbookToDelete].name : ""}
        />
      </Modal>
    </>
  );
}

const StorageIndicator = styled("div")`
  height: 16px;
  width: 16px;
  svg {
    height: 16px;
    width: 16px;
    stroke: #9e9e9e;
  }
`;

const EllipsisButton = styled("button")<{ isOpen: boolean }>`
  background: none;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  height: 24px;
  width: ${({ isOpen }) => (isOpen ? "24px" : "0px")};
  border-radius: 4px;
  color: #333333;
  stroke-width: 2px;
  background-color: ${({ isOpen }) => (isOpen ? "#E0E0E0" : "none")};
  opacity: ${({ isOpen }) => (isOpen ? "1" : "0")};
  &:hover {
    background: #BDBDBD;
    opacity: 1;
  }
  &:active {
    background: #bdbdbd;
    opacity: 1;
  }
`;

const WorkbookListItem = styled(MenuItem)<{ selected: boolean }>`
  display: flex;
  gap: 8px;
  justify-content: flex-start;
  font-size: 14px;
  width: 100%;
  min-width: 172px;
  border-radius: 8px;
  padding: 8px 4px 8px 8px;
  height: 32px;
  min-height: 32px;
  transition: gap 0.5s;
  background-color: ${({ selected }) =>
    selected ? "#e0e0e0 !important" : "transparent"};

  &:hover {
    background-color: #e0e0e0;
    button {
      opacity: 1;
      min-width: 24px;
    }
  }
`;

const WorkbookListText = styled("div")`
  color: #000;
  font-size: 12px;
  width: 100%;
  max-width: 240px;
  overflow: hidden;
  text-overflow: ellipsis;
`;

const StyledMenu = styled(Menu)`
  .MuiPaper-root {
    border-radius: 8px;
    padding: 4px 0px;
    box-shadow: 0px 2px 4px rgba(0, 0, 0, 0.01);
  },
  .MuiList-root {
    padding: 0;
  },
`;

const SectionContainer = styled("div")`
  margin-bottom: 16px;
  display: flex;
  flex-direction: column;
  gap: 2px;
`;

const SectionTitle = styled("div")`
  display: flex;
  align-items: center;
  gap: 4px;
  font-weight: 400;
  color: #9e9e9e;
  margin-bottom: 8px;
  padding: 0px 8px;
  font-size: 12px;
  svg {
    width: 12px;
    height: 12px;
  }
`;

export default WorkbookList;
