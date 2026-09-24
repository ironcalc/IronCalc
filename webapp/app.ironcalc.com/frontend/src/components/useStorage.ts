import { useSyncExternalStore } from "react";
import {
  getModelsMetadata,
  getSelectedUuid,
  type ModelsMetadata,
  subscribeToStorage,
} from "./storage";

// Re-renders the component whenever the workbook list changes.
export function useModelsMetadata(): ModelsMetadata {
  return useSyncExternalStore(subscribeToStorage, getModelsMetadata);
}

// Re-renders the component whenever the selected workbook changes.
export function useSelectedUuid(): string | null {
  return useSyncExternalStore(subscribeToStorage, getSelectedUuid);
}
