import { EllipsisVertical, Pin, Table2 } from "lucide-react";
import type React from "react";
import { useTranslation } from "react-i18next";
import DeleteWorkbookDialog from "../DeleteWorkbookDialog";
import {
  getModelsMetadata,
  getSelectedUuid,
  isWorkbookPinned,
} from "../storage";
import { useWorkbookMenu } from "./useWorkbookMenu";
import WorkbookMenu from "./WorkbookMenu";

interface WorkbookListProps {
  setModel: (key: string) => void;
  onDelete: (uuid: string) => void;
  searchQuery: string;
}

function WorkbookList({ setModel, onDelete, searchQuery }: WorkbookListProps) {
  const { t } = useTranslation();
  const {
    menuAnchorEl,
    menuPosition,
    selectedWorkbookUuid,
    isMenuOpen,
    isDeleteDialogOpen,
    workbookToDelete,
    handleMenuOpen,
    handleMenuClose,
    handleDeleteClick,
    handleDeleteConfirm,
    handleDeleteCancel,
    handleDownload,
    handlePinToggle,
    handleDuplicate,
  } = useWorkbookMenu({ setModel, onDelete });

  const selectedUuid = getSelectedUuid();
  const modelsMetadata = getModelsMetadata();
  const normalizedQuery = searchQuery.trim().toLowerCase();
  const isSearchMode = normalizedQuery.length > 0;

  const groupWorkbooks = (meta: ReturnType<typeof getModelsMetadata>) => {
    const now = Date.now();
    const msInDay = 24 * 60 * 60 * 1000;
    const pinned: string[] = [];
    const today: string[] = [];
    const thisMonth: string[] = [];
    const older: string[] = [];

    for (const uuid in meta) {
      const age = now - meta[uuid].createdAt;
      if (meta[uuid].pinned) pinned.push(uuid);
      else if (age < msInDay) today.push(uuid);
      else if (age < 30 * msInDay) thisMonth.push(uuid);
      else older.push(uuid);
    }

    const sortByNewest = (uuids: string[]) =>
      uuids.sort((a, b) => meta[b].createdAt - meta[a].createdAt);

    return {
      pinned: sortByNewest(pinned),
      today: sortByNewest(today),
      thisMonth: sortByNewest(thisMonth),
      older: sortByNewest(older),
    };
  };

  const renderWorkbookItem = (uuid: string) => {
    const isThisMenuOpen = isMenuOpen && selectedWorkbookUuid === uuid;
    return (
      <div
        key={uuid}
        className={`app-ic-drawer-workbook-item${uuid === selectedUuid ? " app-ic-drawer-workbook-item--selected" : ""}`}
      >
        <button
          type="button"
          className="app-ic-drawer-workbook-select"
          disabled={isMenuOpen}
          onClick={() => setModel(uuid)}
        >
          <span className="app-ic-drawer-workbook-icon">
            <Table2 />
          </span>
          <span className="app-ic-drawer-workbook-name">
            {modelsMetadata[uuid].name}
          </span>
        </button>
        <button
          type="button"
          className={`app-ic-drawer-workbook-ellipsis${isThisMenuOpen ? " app-ic-drawer-workbook-ellipsis--open" : ""}`}
          onClick={(e) => handleMenuOpen(e, uuid)}
          onMouseDown={(e) => e.stopPropagation()}
        >
          <EllipsisVertical />
        </button>
      </div>
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

      {menuAnchorEl && selectedWorkbookUuid && (
        <WorkbookMenu
          position={menuPosition}
          isPinned={isWorkbookPinned(selectedWorkbookUuid)}
          onClose={handleMenuClose}
          onDownload={() => handleDownload(selectedWorkbookUuid)}
          onPinToggle={() => handlePinToggle(selectedWorkbookUuid)}
          onDuplicate={() => handleDuplicate(selectedWorkbookUuid)}
          onDelete={() => handleDeleteClick(selectedWorkbookUuid)}
        />
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
