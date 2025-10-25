import { Model } from "@ironcalc/workbook";
import { base64ToBytes, bytesToBase64 } from "./util";

const MAX_WORKBOOKS = 50;

type ModelsMetadata = Record<
  string,
  { name: string; createdAt: number; pinned?: boolean }
>;

function randomUUID(): string {
  try {
    return crypto.randomUUID();
  } catch {
    // Fallback for environments without crypto.randomUUID()
    return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, (c) => {
      const r = (Math.random() * 16) | 0;
      const v = c === "x" ? r : (r & 0x3) | 0x8;
      return v.toString(16);
    });
  }
}

export function updateNameSelectedWorkbook(model: Model, newName: string) {
  const uuid = localStorage.getItem("selected");
  if (uuid) {
    const modelsJson = localStorage.getItem("models");
    if (modelsJson) {
      try {
        const models = JSON.parse(modelsJson);
        if (models[uuid]) {
          models[uuid].name = newName;
        } else {
          models[uuid] = { name: newName, createdAt: Date.now() };
        }
        localStorage.setItem("models", JSON.stringify(models));
      } catch (e) {
        console.warn("Failed saving new name");
      }
    }
    const modeBytes = model.toBytes();
    localStorage.setItem(uuid, bytesToBase64(modeBytes));
  }
}

export function getModelsMetadata(): ModelsMetadata {
  let modelsJson = localStorage.getItem("models");
  if (!modelsJson) {
    modelsJson = "{}";
  }
  const models = JSON.parse(modelsJson);

  // Migrate old format to new format
  const migratedModels: ModelsMetadata = {};
  for (const [uuid, value] of Object.entries(models)) {
    if (typeof value === "string") {
      // Old format: just the name string
      migratedModels[uuid] = { name: value, createdAt: Date.now() };
    } else if (typeof value === "object" && value !== null && "name" in value) {
      // New format: object with name and createdAt
      migratedModels[uuid] = value as { name: string; createdAt: number };
    }
  }

  // Save migrated data back to localStorage
  if (JSON.stringify(models) !== JSON.stringify(migratedModels)) {
    localStorage.setItem("models", JSON.stringify(migratedModels));
  }

  return migratedModels;
}

// Pick a different name Workbook{N} where N = 1, 2, 3
function getNewName(existingNames: string[]): string {
  const baseName = "Workbook";
  let index = 1;
  while (index < MAX_WORKBOOKS) {
    const name = `${baseName}${index}`;
    index += 1;
    if (!existingNames.includes(name)) {
      return name;
    }
  }
  // FIXME: Too many workbooks?
  return "Workbook-Infinity";
}

export function createNewModel(): Model {
  const models = getModelsMetadata();
  const name = getNewName(Object.values(models).map((m) => m.name));

  const model = new Model(name, "en", "UTC");
  const uuid = randomUUID();
  localStorage.setItem("selected", uuid);
  localStorage.setItem(uuid, bytesToBase64(model.toBytes()));

  models[uuid] = { name, createdAt: Date.now() };
  localStorage.setItem("models", JSON.stringify(models));
  return model;
}

export function loadSelectedModelFromStorage(): Model | null {
  const uuid = localStorage.getItem("selected");
  if (uuid) {
    // We try to load the selected model
    const modelBytesString = localStorage.getItem(uuid);
    if (modelBytesString) {
      return Model.from_bytes(base64ToBytes(modelBytesString));
    }
  }
  return null;
}

// check if storage is empty
export function isStorageEmpty(): boolean {
  const modelsJson = localStorage.getItem("models");
  if (!modelsJson) {
    return true;
  }
  try {
    const models = JSON.parse(modelsJson);
    return Object.keys(models).length === 0;
  } catch (e) {
    return true;
  }
}

export function saveSelectedModelInStorage(model: Model) {
  const uuid = localStorage.getItem("selected");
  if (uuid) {
    const modeBytes = model.toBytes();
    localStorage.setItem(uuid, bytesToBase64(modeBytes));
  }
}

export function saveModelToStorage(model: Model) {
  const uuid = randomUUID();
  localStorage.setItem("selected", uuid);
  localStorage.setItem(uuid, bytesToBase64(model.toBytes()));
  let modelsJson = localStorage.getItem("models");
  if (!modelsJson) {
    modelsJson = "{}";
  }
  const models = JSON.parse(modelsJson);
  models[uuid] = { name: model.getName(), createdAt: Date.now() };
  localStorage.setItem("models", JSON.stringify(models));
}

export function selectModelFromStorage(uuid: string): Model | null {
  localStorage.setItem("selected", uuid);
  const modelBytesString = localStorage.getItem(uuid);
  if (modelBytesString) {
    return Model.from_bytes(base64ToBytes(modelBytesString));
  }
  return null;
}

export function getSelectedUuid(): string | null {
  return localStorage.getItem("selected");
}

export function deleteSelectedModel(): Model | null {
  const uuid = localStorage.getItem("selected");
  if (!uuid) {
    return null;
  }
  localStorage.removeItem(uuid);
  const metadata = getModelsMetadata();
  delete metadata[uuid];
  localStorage.setItem("models", JSON.stringify(metadata));
  const uuids = Object.keys(metadata);
  if (uuids.length === 0) {
    return createNewModel();
  }
  return selectModelFromStorage(uuids[0]);
}

export function deleteModelByUuid(uuid: string): Model | null {
  localStorage.removeItem(uuid);
  const metadata = getModelsMetadata();
  delete metadata[uuid];
  localStorage.setItem("models", JSON.stringify(metadata));

  // If this was the selected model, we need to select a different one
  const selectedUuid = localStorage.getItem("selected");
  if (selectedUuid === uuid) {
    const uuids = Object.keys(metadata);
    if (uuids.length === 0) {
      return createNewModel();
    }
    // Find the newest workbook by creation timestamp
    const newestUuid = uuids.reduce((newest, current) => {
      const newestTime = metadata[newest]?.createdAt || 0;
      const currentTime = metadata[current]?.createdAt || 0;
      return currentTime > newestTime ? current : newest;
    });
    return selectModelFromStorage(newestUuid);
  }

  // If it wasn't the selected model, return the currently selected model
  if (selectedUuid) {
    const modelBytesString = localStorage.getItem(selectedUuid);
    if (modelBytesString) {
      return Model.from_bytes(base64ToBytes(modelBytesString));
    }
  }

  // Fallback to creating a new model if no valid selected model
  return createNewModel();
}

export function togglePinWorkbook(uuid: string): void {
  const metadata = getModelsMetadata();
  if (metadata[uuid]) {
    metadata[uuid].pinned = !metadata[uuid].pinned;
    localStorage.setItem("models", JSON.stringify(metadata));
  }
}

export function isWorkbookPinned(uuid: string): boolean {
  const metadata = getModelsMetadata();
  return metadata[uuid]?.pinned || false;
}

export function duplicateModel(uuid: string): Model | null {
  const originalModel = selectModelFromStorage(uuid);
  if (!originalModel) return null;

  const duplicatedModel = Model.from_bytes(originalModel.toBytes());
  const models = getModelsMetadata();
  const originalName = models[uuid]?.name || "Workbook";
  const existingNames = Object.values(models).map((m) => m.name);

  // Find next available number
  let counter = 1;
  let newName = `${originalName} (${counter})`;
  while (existingNames.includes(newName)) {
    counter++;
    newName = `${originalName} (${counter})`;
  }

  duplicatedModel.setName(newName);

  const newUuid = randomUUID();
  localStorage.setItem("selected", newUuid);
  localStorage.setItem(newUuid, bytesToBase64(duplicatedModel.toBytes()));

  models[newUuid] = { name: newName, createdAt: Date.now() };
  localStorage.setItem("models", JSON.stringify(models));

  return duplicatedModel;
}
