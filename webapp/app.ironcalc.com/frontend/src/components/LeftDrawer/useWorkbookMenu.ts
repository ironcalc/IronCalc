import { useEffect, useState } from "react";
import { downloadModel } from "../rpc";
import {
  duplicateModel,
  loadModelFromStorage,
  togglePinWorkbook,
} from "../storage";
import { useSelectedUuid } from "../useStorage";

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

  const selectedUuid = useSelectedUuid();

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
      const model = await loadModelFromStorage(uuid);
      if (model) {
        await downloadModel(model.toBytes(), model.getName());
      }
    } catch (error) {
      console.error("Failed to download workbook:", error);
    }
  };

  const handlePinToggle = (uuid: string) => {
    togglePinWorkbook(uuid).catch((e) =>
      console.error("Failed to toggle pin:", e),
    );
    setIntendedSelection(null);
  };

  const handleDuplicate = (uuid: string) => {
    duplicateModel(uuid).catch((e) =>
      console.error("Failed to duplicate workbook:", e),
    );
    setIntendedSelection(null);
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
