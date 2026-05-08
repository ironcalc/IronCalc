import type React from "react";
import { useEffect, useState } from "react";
import { downloadModel } from "../rpc";
import {
  duplicateModel,
  getSelectedUuid,
  selectModelFromStorage,
  togglePinWorkbook,
} from "../storage";

interface Options {
  setModel: (uuid: string) => void;
  onDelete: (uuid: string) => void;
}

export function useWorkbookMenu({ setModel, onDelete }: Options) {
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
    handleMenuClose();
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

  return {
    menuAnchorEl,
    menuPosition,
    selectedWorkbookUuid,
    isMenuOpen: menuAnchorEl !== null,
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
  };
}
