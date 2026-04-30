import "./workbook-list.css";
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

interface WorkbookListProps {
  setModel: (key: string) => void;
  onDelete: (uuid: string) => void;
  searchQuery: string;
}

function WorkbookList({ setModel, onDelete, searchQuery }: WorkbookListProps) {
  const { t } = useTranslation();
  const [openMenuUuid, setOpenMenuUuid] = useState<string | null>(null);
  const [menuPosition, setMenuPosition] = useState<{
    top: number;
    right: number;
  } | null>(null);
  const [selectedWorkbookUuid, setSelectedWorkbookUuid] = useState<
    string | null
  >(null);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [workbookToDelete, setWorkbookToDelete] = useState<string | null>(null);
  const [intendedSelection, setIntendedSelection] = useState<string | null>(
    null,
  );

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
    setOpenMenuUuid(uuid);
    setSelectedWorkbookUuid(uuid);
    setIntendedSelection(uuid);
    setModel(uuid);
  };

  const handleMenuClose = () => {
    setOpenMenuUuid(null);
    setMenuPosition(null);
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

  const groupWorkbooks = (
    modelsMetadata: ReturnType<typeof getModelsMetadata>,
  ) => {
    const now = Date.now();
    const millisecondsInDay = 24 * 60 * 60 * 1000;
    const millisecondsIn30Days = 30 * millisecondsInDay;

    const pinnedModels = [];
    const modelsCreatedToday = [];
    const modelsCreatedThisMonth = [];
    const olderModels = [];

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

  const modelsMetadata = getModelsMetadata();
  const normalizedQuery = searchQuery.trim().toLowerCase();
  const isSearchMode = normalizedQuery.length > 0;
  const isAnyMenuOpen = openMenuUuid !== null;

  let pinnedModels: string[] = [];
  let modelsCreatedToday: string[] = [];
  let modelsCreatedThisMonth: string[] = [];
  let olderModels: string[] = [];

  const renderWorkbookItem = (uuid: string) => {
    const isMenuOpen = openMenuUuid === uuid;

    return (
      <button
        key={uuid}
        type="button"
        className={[
          "workbook-list-item",
          uuid === selectedUuid ? "workbook-list-item--selected" : "",
        ]
          .filter(Boolean)
          .join(" ")}
        onClick={() => {
          if (isAnyMenuOpen) return;
          setModel(uuid);
        }}
        style={{ pointerEvents: isAnyMenuOpen ? "none" : "auto" }}
      >
        <span className="workbook-list-item-icon">
          <Table2 />
        </span>
        <span className="workbook-list-item-name">
          {modelsMetadata[uuid].name}
        </span>
        <button
          type="button"
          className={[
            "workbook-list-ellipsis",
            isMenuOpen ? "workbook-list-ellipsis--open" : "",
          ]
            .filter(Boolean)
            .join(" ")}
          onClick={(e) => handleMenuOpen(e, uuid)}
          onMouseDown={(e) => e.stopPropagation()}
          style={{ pointerEvents: "auto" }}
        >
          <EllipsisVertical />
        </button>
      </button>
    );
  };

  const renderSection = (title: string, uuids: string[]) => {
    if (uuids.length === 0) return null;
    return (
      <div key={title} className="workbook-list-section">
        <div className="workbook-list-section-title">
          {title === t("left_drawer.pinned") && <Pin />}
          {title}
        </div>
        {uuids.map(renderWorkbookItem)}
      </div>
    );
  };

  let filteredModels: string[] = [];

  if (isSearchMode) {
    filteredModels = Object.keys(modelsMetadata)
      .sort((a, b) => modelsMetadata[b].createdAt - modelsMetadata[a].createdAt)
      .filter((uuid) =>
        modelsMetadata[uuid].name.toLowerCase().includes(normalizedQuery),
      );
  } else {
    ({ pinnedModels, modelsCreatedToday, modelsCreatedThisMonth, olderModels } =
      groupWorkbooks(modelsMetadata));
  }

  const isNoResults = isSearchMode && filteredModels.length === 0;

  return (
    <>
      {isSearchMode ? (
        filteredModels.map(renderWorkbookItem)
      ) : (
        <>
          {renderSection(t("left_drawer.pinned"), pinnedModels)}
          {renderSection(t("left_drawer.today"), modelsCreatedToday)}
          {renderSection(t("left_drawer.last_30_days"), modelsCreatedThisMonth)}
          {renderSection(t("left_drawer.older"), olderModels)}
        </>
      )}
      {isNoResults && (
        <div className="workbook-list-no-results">
          {t("left_drawer.search_no_results")}
        </div>
      )}

      {openMenuUuid &&
        menuPosition &&
        createPortal(
          <>
            {/* biome-ignore lint/a11y/noStaticElementInteractions: backdrop captures outside clicks */}
            <div
              className="workbook-list-menu-backdrop"
              onMouseDown={handleMenuClose}
            />
            <div
              className="workbook-list-menu-panel"
              style={{ top: menuPosition.top, right: menuPosition.right }}
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
