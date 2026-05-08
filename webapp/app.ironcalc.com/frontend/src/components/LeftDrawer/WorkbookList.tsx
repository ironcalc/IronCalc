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
import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import DeleteWorkbookDialog from "../DeleteWorkbookDialog";
import {
  DeleteButton,
  MenuDivider,
  MenuItemWrapper,
} from "../Navigation/FileMenu";
import { downloadModel } from "../rpc";
import {
  duplicateModel,
  getModelsMetadata,
  getSelectedUuid,
  isWorkbookPinned,
  selectModelFromStorage,
  togglePinWorkbook,
} from "../storage";
import "./left-drawer.css";

interface WorkbookListProps {
  setModel: (key: string) => void;
  onDelete: (uuid: string) => void;
  searchQuery: string;
}

function WorkbookList({ setModel, onDelete, searchQuery }: WorkbookListProps) {
  const { t } = useTranslation();
  const [menuAnchorEl, setMenuAnchorEl] = useState<HTMLButtonElement | null>(
    null,
  );
  const [menuPosition, setMenuPosition] = useState({ top: 0, right: 0 });
  const [selectedWorkbookUuid, setSelectedWorkbookUuid] = useState<
    string | null
  >(null);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [workbookToDelete, setWorkbookToDelete] = useState<string | null>(null);
  const [intendedSelection, setIntendedSelection] = useState<string | null>(
    null,
  );
  const menuRef = useRef<HTMLDivElement>(null);

  const selectedUuid = getSelectedUuid();

  useEffect(() => {
    if (intendedSelection && selectedUuid === intendedSelection) {
      setIntendedSelection(null);
    }
  }, [selectedUuid, intendedSelection]);

  const handleMenuOpen = (
    event: React.MouseEvent<HTMLButtonElement>,
    uuid: string,
  ) => {
    event.stopPropagation();
    const rect = event.currentTarget.getBoundingClientRect();
    setMenuPosition({
      top: rect.bottom + 4,
      right: window.innerWidth - rect.right,
    });
    setSelectedWorkbookUuid(uuid);
    setMenuAnchorEl(event.currentTarget);
    setIntendedSelection(uuid);
    setModel(uuid);
  };

  const handleMenuClose = () => {
    setMenuAnchorEl(null);
    if (intendedSelection && intendedSelection !== selectedUuid) {
      setModel(intendedSelection);
    }
  };

  const handleDeleteClick = (uuid: string) => {
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
        await downloadModel(model.toBytes(), model.getName());
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
    duplicateModel(uuid);
    setIntendedSelection(null);
    handleMenuClose();
  };

  const groupWorkbooks = (
    modelsMetadata: ReturnType<typeof getModelsMetadata>,
  ) => {
    const now = Date.now();
    const msInDay = 24 * 60 * 60 * 1000;

    const pinned: string[] = [];
    const today: string[] = [];
    const thisMonth: string[] = [];
    const older: string[] = [];

    for (const uuid in modelsMetadata) {
      const age = now - modelsMetadata[uuid].createdAt;
      if (modelsMetadata[uuid].pinned) {
        pinned.push(uuid);
      } else if (age < msInDay) {
        today.push(uuid);
      } else if (age < 30 * msInDay) {
        thisMonth.push(uuid);
      } else {
        older.push(uuid);
      }
    }

    const sortByNewest = (uuids: string[]) =>
      uuids.sort(
        (a, b) => modelsMetadata[b].createdAt - modelsMetadata[a].createdAt,
      );

    return {
      pinned: sortByNewest(pinned),
      today: sortByNewest(today),
      thisMonth: sortByNewest(thisMonth),
      older: sortByNewest(older),
    };
  };

  const modelsMetadata = getModelsMetadata();
  const normalizedQuery = searchQuery.trim().toLowerCase();
  const isSearchMode = normalizedQuery.length > 0;
  const isMenuOpen = menuAnchorEl !== null;

  const renderWorkbookItem = (uuid: string) => {
    const isThisMenuOpen = isMenuOpen && selectedWorkbookUuid === uuid;

    return (
      <button
        key={uuid}
        type="button"
        className={`app-ic-drawer-workbook-item${uuid === selectedUuid ? " app-ic-drawer-workbook-item--selected" : ""}`}
        style={{ pointerEvents: isMenuOpen ? "none" : "auto" }}
        onClick={() => {
          if (isMenuOpen) return;
          setModel(uuid);
        }}
      >
        <span className="app-ic-drawer-workbook-icon">
          <Table2 />
        </span>
        <span className="app-ic-drawer-workbook-name">
          {modelsMetadata[uuid].name}
        </span>
        <button
          type="button"
          className={`app-ic-drawer-workbook-ellipsis${isThisMenuOpen ? " app-ic-drawer-workbook-ellipsis--open" : ""}`}
          style={{ pointerEvents: "auto" }}
          onClick={(e) => handleMenuOpen(e, uuid)}
          onMouseDown={(e) => e.stopPropagation()}
        >
          <EllipsisVertical />
        </button>
      </button>
    );
  };

  const renderSection = (title: string, uuids: string[]) => {
    if (uuids.length === 0) return null;
    return (
      <div key={title} className="app-ic-drawer-section">
        <div className="app-ic-drawer-section-title">
          {title === t("left_drawer.pinned") && <Pin />}
          {title}
        </div>
        {uuids.map(renderWorkbookItem)}
      </div>
    );
  };

  let content: React.ReactNode;

  if (isSearchMode) {
    const filteredModels = Object.keys(modelsMetadata)
      .sort((a, b) => modelsMetadata[b].createdAt - modelsMetadata[a].createdAt)
      .filter((uuid) =>
        modelsMetadata[uuid].name.toLowerCase().includes(normalizedQuery),
      );
    content =
      filteredModels.length === 0 ? (
        <div className="app-ic-drawer-no-results">
          {t("left_drawer.search_no_results")}
        </div>
      ) : (
        filteredModels.map(renderWorkbookItem)
      );
  } else {
    const { pinned, today, thisMonth, older } = groupWorkbooks(modelsMetadata);
    content = (
      <>
        {renderSection(t("left_drawer.pinned"), pinned)}
        {renderSection(t("left_drawer.today"), today)}
        {renderSection(t("left_drawer.last_30_days"), thisMonth)}
        {renderSection(t("left_drawer.older"), older)}
      </>
    );
  }

  return (
    <>
      {content}

      {menuAnchorEl &&
        createPortal(
          <>
            <div
              className="app-ic-drawer-menu-backdrop"
              onClick={handleMenuClose}
              role="none"
            />
            <div
              ref={menuRef}
              role="menu"
              className="app-ic-nav-menu"
              style={{
                position: "fixed",
                top: menuPosition.top,
                right: menuPosition.right,
              }}
              onKeyDown={(e) => e.key === "Escape" && handleMenuClose()}
            >
              <MenuItemWrapper
                onClick={() => {
                  if (selectedWorkbookUuid)
                    handleDownload(selectedWorkbookUuid);
                  setIntendedSelection(null);
                  handleMenuClose();
                }}
              >
                <FileDown />
                {t("left_drawer.workbook_menu.download")}
              </MenuItemWrapper>

              <MenuItemWrapper
                onClick={() => {
                  if (selectedWorkbookUuid)
                    handlePinToggle(selectedWorkbookUuid);
                }}
              >
                {selectedWorkbookUuid &&
                isWorkbookPinned(selectedWorkbookUuid) ? (
                  <PinOff />
                ) : (
                  <Pin />
                )}
                {selectedWorkbookUuid && isWorkbookPinned(selectedWorkbookUuid)
                  ? t("left_drawer.workbook_menu.unpin")
                  : t("left_drawer.workbook_menu.pin")}
              </MenuItemWrapper>

              <MenuItemWrapper
                onClick={() => {
                  if (selectedWorkbookUuid)
                    handleDuplicate(selectedWorkbookUuid);
                }}
              >
                <Copy />
                {t("left_drawer.workbook_menu.duplicate")}
              </MenuItemWrapper>

              <MenuDivider />

              <DeleteButton
                onClick={() => {
                  if (selectedWorkbookUuid)
                    handleDeleteClick(selectedWorkbookUuid);
                }}
              >
                <Trash2 size={16} />
                {t("left_drawer.workbook_menu.delete")}
              </DeleteButton>
            </div>
          </>,
          document.body,
        )}

      <DeleteWorkbookDialog
        open={isDeleteDialogOpen}
        onClose={handleDeleteCancel}
        onConfirm={handleDeleteConfirm}
        workbookName={
          workbookToDelete ? modelsMetadata[workbookToDelete].name : ""
        }
      />
    </>
  );
}

export default WorkbookList;
