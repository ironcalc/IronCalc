import "fake-indexeddb/auto";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

// Lightweight fake Model: round-trips its name through `toBytes`/`fromBytes`
// so tests can assert what was persisted to / reloaded from IndexedDB without
// pulling in WASM.
vi.mock("@ironcalc/workbook", () => {
  class Model {
    name: string;
    locale: string;
    language: string;
    constructor(name: string, locale: string, _tz: string, language: string) {
      this.name = name;
      this.locale = locale;
      this.language = language;
    }
    static fromBytes(bytes: Uint8Array, language: string): Model {
      const { name } = JSON.parse(new TextDecoder().decode(bytes));
      return new Model(name, "en", "UTC", language);
    }
    toBytes(): Uint8Array {
      return new TextEncoder().encode(JSON.stringify({ name: this.name }));
    }
    getName(): string {
      return this.name;
    }
    setName(name: string): void {
      this.name = name;
    }
    getLanguage(): string {
      return this.language;
    }
    setLocale(locale: string): void {
      this.locale = locale;
    }
  }
  return { Model };
});

// Deterministic workbook base name so getNewName() yields Workbook1, Workbook2…
vi.mock("../i18n", () => ({
  default: {
    t: (key: string) => (key === "default_workbook_name" ? "Workbook" : key),
  },
}));

type Storage = typeof import("./storage");

// Re-import the storage module with a clean module-level cache (simulates an
// app reload). Closes the current connection first but does NOT touch
// IndexedDB or localStorage, so persisted data survives.
async function reload(): Promise<Storage> {
  await storage.closeStorage();
  vi.resetModules();
  const fresh: Storage = await import("./storage");
  await fresh.initStorage();
  return fresh;
}

function deleteDatabase(): Promise<void> {
  return new Promise((resolve) => {
    const request = indexedDB.deleteDatabase("ironcalc");
    request.onsuccess = () => resolve();
    request.onerror = () => resolve();
    request.onblocked = () => resolve();
  });
}

let storage: Storage;

beforeEach(async () => {
  await storage?.closeStorage();
  await deleteDatabase();
  localStorage.clear();
  vi.resetModules();
  storage = await import("./storage");
  await storage.initStorage();
});

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe("initStorage", () => {
  it("starts empty on a fresh database", () => {
    expect(storage.getModelsMetadata()).toEqual({});
    expect(storage.getSelectedUuid()).toBeNull();
    expect(storage.isStorageEmpty()).toBe(true);
  });
});

describe("preferences stay in localStorage", () => {
  it("persists the default locale", () => {
    storage.saveDefaultLocaleInStorage("fr-FR");
    expect(localStorage.getItem("default_locale")).toBe("fr-FR");
    expect(storage.loadDefaultLocaleFromStorage()).toBe("fr-FR");
  });

  it("persists dark mode", () => {
    storage.saveDarkModeInStorage(true);
    expect(localStorage.getItem("dark_mode")).toBe("true");
    expect(storage.loadDarkModeFromStorage()).toBe(true);
  });

  it("keeps the selected uuid in localStorage", async () => {
    await storage.createNewModel();
    expect(localStorage.getItem("selected")).toBe(storage.getSelectedUuid());
  });
});

describe("createNewModel", () => {
  it("creates, selects and persists a workbook", async () => {
    const model = await storage.createNewModel();
    expect(model.getName()).toBe("Workbook1");
    const uuid = storage.getSelectedUuid();
    expect(uuid).not.toBeNull();
    expect(storage.getModelsMetadata()[uuid as string]).toMatchObject({
      name: "Workbook1",
      pinned: false,
    });
    expect(storage.isStorageEmpty()).toBe(false);

    storage = await reload();
    expect(storage.getModelsMetadata()[uuid as string]?.name).toBe("Workbook1");
    const loaded = await storage.loadSelectedModelFromStorage();
    expect(loaded?.getName()).toBe("Workbook1");
  });

  it("picks the next free default name", async () => {
    await storage.createNewModel();
    const second = await storage.createNewModel();
    expect(second.getName()).toBe("Workbook2");
  });

  it("notifies subscribers", async () => {
    const listener = vi.fn();
    storage.subscribeToStorage(listener);
    await storage.createNewModel();
    expect(listener).toHaveBeenCalled();
  });

  it("returns a new metadata object on each change", async () => {
    const before = storage.getModelsMetadata();
    await storage.createNewModel();
    expect(storage.getModelsMetadata()).not.toBe(before);
  });
});

describe("saveModelToStorage", () => {
  it("stores an existing model under a new uuid and selects it", async () => {
    const { Model } = await import("@ironcalc/workbook");
    const model = new Model("Imported", "en", "UTC", "en");
    await storage.saveModelToStorage(model);
    const uuid = storage.getSelectedUuid() as string;
    expect(storage.getModelsMetadata()[uuid].name).toBe("Imported");
    const loaded = await storage.loadModelFromStorage(uuid);
    expect(loaded?.getName()).toBe("Imported");
  });
});

describe("saveSelectedModelInStorage", () => {
  it("overwrites the bytes of the selected workbook", async () => {
    const model = await storage.createNewModel();
    model.setName("Changed");
    await storage.saveSelectedModelInStorage(model);
    storage = await reload();
    const loaded = await storage.loadSelectedModelFromStorage();
    expect(loaded?.getName()).toBe("Changed");
  });

  it("is a no-op when nothing is selected", async () => {
    const { Model } = await import("@ironcalc/workbook");
    const model = new Model("Orphan", "en", "UTC", "en");
    await storage.saveSelectedModelInStorage(model);
    expect(storage.isStorageEmpty()).toBe(true);
  });
});

describe("updateNameSelectedWorkbook", () => {
  it("updates metadata and bytes", async () => {
    const model = await storage.createNewModel();
    const uuid = storage.getSelectedUuid() as string;
    model.setName("Renamed");
    await storage.updateNameSelectedWorkbook(model, "Renamed");
    expect(storage.getModelsMetadata()[uuid].name).toBe("Renamed");
    storage = await reload();
    expect(storage.getModelsMetadata()[uuid].name).toBe("Renamed");
    const loaded = await storage.loadSelectedModelFromStorage();
    expect(loaded?.getName()).toBe("Renamed");
  });
});

describe("selectModelFromStorage", () => {
  it("switches the selection", async () => {
    await storage.createNewModel();
    const first = storage.getSelectedUuid() as string;
    await storage.createNewModel();
    const model = await storage.selectModelFromStorage(first);
    expect(model?.getName()).toBe("Workbook1");
    expect(storage.getSelectedUuid()).toBe(first);
  });

  it("returns null and keeps the selection for an unknown uuid", async () => {
    await storage.createNewModel();
    const selected = storage.getSelectedUuid();
    const model = await storage.selectModelFromStorage("missing");
    expect(model).toBeNull();
    expect(storage.getSelectedUuid()).toBe(selected);
  });
});

describe("loadSelectedModelFromStorage", () => {
  it("clears a dangling selection without wiping other data", async () => {
    await storage.createNewModel();
    localStorage.setItem("selected", "missing");
    const model = await storage.loadSelectedModelFromStorage();
    expect(model).toBeNull();
    expect(storage.getSelectedUuid()).toBeNull();
    expect(storage.isStorageEmpty()).toBe(false);
  });

  it("clears the selection when bytes cannot be deserialized", async () => {
    await storage.createNewModel();
    const uuid = storage.getSelectedUuid() as string;
    // Corrupt the stored bytes: the fake Model expects JSON.
    const { Model } = await import("@ironcalc/workbook");
    vi.spyOn(Model, "fromBytes").mockImplementationOnce(() => {
      throw new Error("bad bytes");
    });
    const model = await storage.loadSelectedModelFromStorage();
    expect(model).toBeNull();
    expect(storage.getSelectedUuid()).toBeNull();
    // The workbook is still there.
    expect(storage.getModelsMetadata()[uuid]).toBeDefined();
  });
});

describe("togglePinWorkbook", () => {
  it("flips and persists the pinned flag", async () => {
    await storage.createNewModel();
    const uuid = storage.getSelectedUuid() as string;
    await storage.togglePinWorkbook(uuid);
    expect(storage.isWorkbookPinned(uuid)).toBe(true);
    storage = await reload();
    expect(storage.isWorkbookPinned(uuid)).toBe(true);
    await storage.togglePinWorkbook(uuid);
    expect(storage.isWorkbookPinned(uuid)).toBe(false);
  });
});

describe("duplicateModel", () => {
  it("stores a copy with a numbered name without selecting it", async () => {
    await storage.createNewModel();
    const original = storage.getSelectedUuid() as string;
    const copy = await storage.duplicateModel(original);
    expect(copy?.getName()).toBe("Workbook1 (1)");
    expect(storage.getSelectedUuid()).toBe(original);
    const names = Object.values(storage.getModelsMetadata()).map((m) => m.name);
    expect(names.sort()).toEqual(["Workbook1", "Workbook1 (1)"]);

    const second = await storage.duplicateModel(original);
    expect(second?.getName()).toBe("Workbook1 (2)");
  });

  it("returns null for an unknown uuid", async () => {
    expect(await storage.duplicateModel("missing")).toBeNull();
  });
});

describe("deleteModelByUuid", () => {
  it("removes a non-selected workbook and returns null", async () => {
    await storage.createNewModel();
    const first = storage.getSelectedUuid() as string;
    await storage.createNewModel();
    const second = storage.getSelectedUuid() as string;
    const result = await storage.deleteModelByUuid(first);
    expect(result).toBeNull();
    expect(storage.getSelectedUuid()).toBe(second);
    expect(storage.getModelsMetadata()[first]).toBeUndefined();
    storage = await reload();
    expect(await storage.loadModelFromStorage(first)).toBeNull();
  });

  it("selects the newest remaining workbook when deleting the selected one", async () => {
    vi.spyOn(Date, "now").mockReturnValue(1000);
    await storage.createNewModel();
    const first = storage.getSelectedUuid() as string;
    vi.spyOn(Date, "now").mockReturnValue(2000);
    await storage.createNewModel();
    const second = storage.getSelectedUuid() as string;
    vi.spyOn(Date, "now").mockReturnValue(3000);
    await storage.createNewModel();
    const third = storage.getSelectedUuid() as string;

    const result = await storage.deleteModelByUuid(third);
    expect(result?.getName()).toBe("Workbook2");
    expect(storage.getSelectedUuid()).toBe(second);
    expect(storage.getModelsMetadata()[first]).toBeDefined();
  });

  it("creates a fresh workbook when the last one is deleted", async () => {
    await storage.createNewModel();
    const only = storage.getSelectedUuid() as string;
    const result = await storage.deleteModelByUuid(only);
    expect(result?.getName()).toBe("Workbook1");
    const selected = storage.getSelectedUuid();
    expect(selected).not.toBeNull();
    expect(selected).not.toBe(only);
    expect(Object.keys(storage.getModelsMetadata())).toEqual([selected]);
  });

  it("serializes concurrent deletes", async () => {
    await storage.createNewModel();
    const a = storage.getSelectedUuid() as string;
    await storage.createNewModel();
    const b = storage.getSelectedUuid() as string;
    await storage.createNewModel();
    const c = storage.getSelectedUuid() as string;
    // Fire without awaiting, like the multi-select delete in the drawer.
    const results = await Promise.all([
      storage.deleteModelByUuid(c),
      storage.deleteModelByUuid(b),
    ]);
    expect(results[0]?.getName()).toBe("Workbook2");
    expect(results[1]?.getName()).toBe("Workbook1");
    expect(storage.getSelectedUuid()).toBe(a);
    expect(Object.keys(storage.getModelsMetadata())).toEqual([a]);
  });
});

describe("deleteSelectedModel", () => {
  it("returns null when nothing is selected", async () => {
    expect(await storage.deleteSelectedModel()).toBeNull();
  });

  it("deletes the selected workbook", async () => {
    await storage.createNewModel();
    await storage.createNewModel();
    const second = storage.getSelectedUuid() as string;
    const result = await storage.deleteSelectedModel();
    expect(result?.getName()).toBe("Workbook1");
    expect(storage.getModelsMetadata()[second]).toBeUndefined();
  });
});
