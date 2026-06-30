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
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [workbookToDelete, setWorkbookToDelete] = useState<string | null>(null);
  const [intendedSelection, setIntendedSelection] = useState<string | null>(
    null,
  );
  const [, forceRefresh] = useState(0);

  const selectedUuid = getSelectedUuid();

  useEffect(() => {
    if (intendedSelection && selectedUuid === intendedSelection) {
      setIntendedSelection(null);
    }
  }, [selectedUuid, intendedSelection]);

  const handleMenuOpen = (uuid: string) => {
    setIntendedSelection(uuid);
    setModel(uuid);
  };

  const handleDeleteClick = (uuid: string) => {
    setWorkbookToDelete(uuid);
    setIsDeleteDialogOpen(true);
    setIntendedSelection(null);
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
    forceRefresh((n) => n + 1);
  };

  const handleDuplicate = (uuid: string) => {
    duplicateModel(uuid);
    setIntendedSelection(null);
    forceRefresh((n) => n + 1);
  };

  return {
    isDeleteDialogOpen,
    workbookToDelete,
    handleMenuOpen,
    handleDeleteClick,
    handleDeleteConfirm,
    handleDeleteCancel,
    handleDownload,
    handlePinToggle,
    handleDuplicate,
  };
}
