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

// Direct access to the database, bypassing the storage module, to set up
// (and inspect) states the module itself would never produce.
function rawTransaction<T>(
  mode: IDBTransactionMode,
  body: (workbooks: IDBObjectStore, bytes: IDBObjectStore) => T,
): Promise<T> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open("ironcalc", 1);
    request.onerror = () => reject(request.error);
    request.onsuccess = () => {
      const db = request.result;
      const tx = db.transaction(["workbooks", "workbook_bytes"], mode);
      const result = body(
        tx.objectStore("workbooks"),
        tx.objectStore("workbook_bytes"),
      );
      tx.oncomplete = () => {
        db.close();
        resolve(result);
      };
      tx.onerror = () => {
        db.close();
        reject(tx.error);
      };
    };
  });
}

function rawWrite(
  body: (workbooks: IDBObjectStore, bytes: IDBObjectStore) => void,
): Promise<void> {
  return rawTransaction("readwrite", body);
}

async function rawKeys(): Promise<{
  workbooks: IDBValidKey[];
  bytes: IDBValidKey[];
}> {
  const requests = await rawTransaction("readonly", (workbooks, bytes) => ({
    workbooks: workbooks.getAllKeys(),
    bytes: bytes.getAllKeys(),
  }));
  return {
    workbooks: requests.workbooks.result,
    bytes: requests.bytes.result,
  };
}

const GOOD = { name: "Good", createdAt: 1000, pinned: false };
const BYTES = new TextEncoder().encode(JSON.stringify({ name: "Good" }));
// The fake Model expects JSON, so this fails to deserialize.
const CORRUPT = new TextEncoder().encode("garbage");

function corruptBytes(uuid: string): Promise<void> {
  return rawWrite((_workbooks, bytes) => {
    bytes.put(CORRUPT, uuid);
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

  it("persists the theme mode", () => {
    storage.saveThemeModeInStorage("dark");
    expect(localStorage.getItem("dark_mode")).toBe("dark");
    expect(storage.loadThemeModeFromStorage()).toBe("dark");
    storage.saveThemeModeInStorage("system");
    expect(storage.loadThemeModeFromStorage()).toBe("system");
  });

  it("defaults the theme mode to system for missing or unknown values", () => {
    expect(storage.loadThemeModeFromStorage()).toBe("system");
    localStorage.setItem("dark_mode", "garbage");
    expect(storage.loadThemeModeFromStorage()).toBe("system");
  });

  it("keeps the selected uuid in localStorage", async () => {
    await storage.createNewModel();
    expect(localStorage.getItem("selected")).toBe(storage.getSelectedUuid());
  });
});

describe("initLocalStorage", () => {
  it("stamps the version on a fresh install", () => {
    expect(localStorage.getItem("storage_version")).toBe("1");
  });

  it("wipes everything written by another version", () => {
    localStorage.clear();
    // Layout of the versions that kept the workbooks in localStorage.
    const uuid = "7b1a4c7e-1f2d-4a3b-9c8d-0e1f2a3b4c5d";
    localStorage.setItem("models", JSON.stringify({ [uuid]: { name: "Old" } }));
    localStorage.setItem(uuid, "AAAA");
    localStorage.setItem("selected", uuid);
    localStorage.setItem("dark_mode", "true");
    storage.initLocalStorage();
    expect(localStorage.length).toBe(1);
    expect(localStorage.getItem("storage_version")).toBe("1");
  });

  it("does nothing once the version matches", () => {
    localStorage.setItem("dark_mode", "dark");
    storage.initLocalStorage();
    expect(localStorage.getItem("dark_mode")).toBe("dark");
  });

  it("runs as part of initStorage", async () => {
    localStorage.clear();
    localStorage.setItem("models", "{}");
    storage = await reload();
    expect(localStorage.getItem("models")).toBeNull();
    expect(localStorage.getItem("storage_version")).toBe("1");
  });
});

describe("initStorage repairs the database", () => {
  it("keeps a consistent workbook untouched", async () => {
    await rawWrite((workbooks, bytes) => {
      workbooks.put(GOOD, "a");
      bytes.put(BYTES, "a");
    });
    storage = await reload();
    expect(storage.getModelsMetadata()).toEqual({ a: GOOD });
    expect((await storage.loadModelFromStorage("a"))?.getName()).toBe("Good");
  });

  it("removes metadata that has no bytes", async () => {
    await rawWrite((workbooks, bytes) => {
      workbooks.put(GOOD, "a");
      bytes.put(BYTES, "a");
      workbooks.put({ ...GOOD, name: "NoBytes" }, "b");
    });
    storage = await reload();
    expect(Object.keys(storage.getModelsMetadata())).toEqual(["a"]);
    expect((await rawKeys()).workbooks).toEqual(["a"]);
  });

  it("removes bytes that have no metadata", async () => {
    await rawWrite((workbooks, bytes) => {
      workbooks.put(GOOD, "a");
      bytes.put(BYTES, "a");
      bytes.put(BYTES, "orphan");
    });
    storage = await reload();
    expect((await rawKeys()).bytes).toEqual(["a"]);
  });

  it("removes workbooks whose metadata cannot be salvaged", async () => {
    await rawWrite((workbooks, bytes) => {
      workbooks.put("not an object", "a");
      bytes.put(BYTES, "a");
      workbooks.put({ createdAt: 1, pinned: false }, "b");
      bytes.put(BYTES, "b");
      workbooks.put(null, "c");
      bytes.put(BYTES, "c");
    });
    storage = await reload();
    expect(storage.getModelsMetadata()).toEqual({});
    expect(await rawKeys()).toEqual({ workbooks: [], bytes: [] });
  });

  it("fills in missing fields and persists the repaired record", async () => {
    vi.spyOn(Date, "now").mockReturnValue(4242);
    await rawWrite((workbooks, bytes) => {
      workbooks.put({ name: "Partial" }, "a");
      bytes.put(BYTES, "a");
      workbooks.put({ name: "Bad", createdAt: "yesterday", pinned: 1 }, "b");
      bytes.put(BYTES, "b");
    });
    storage = await reload();
    expect(storage.getModelsMetadata()).toEqual({
      a: { name: "Partial", createdAt: 4242, pinned: false },
      b: { name: "Bad", createdAt: 4242, pinned: false },
    });
    vi.restoreAllMocks();
    // The repair was written through, not just applied to the cache.
    storage = await reload();
    expect(storage.getModelsMetadata().a.createdAt).toBe(4242);
  });

  it("removes records with non-string keys", async () => {
    await rawWrite((workbooks, bytes) => {
      workbooks.put(GOOD, 7);
      bytes.put(BYTES, 7);
      bytes.put(BYTES, new Date(0));
    });
    storage = await reload();
    expect(storage.getModelsMetadata()).toEqual({});
    expect(await rawKeys()).toEqual({ workbooks: [], bytes: [] });
  });

  it("clears a selection hint that points at no workbook", async () => {
    await storage.createNewModel();
    localStorage.setItem("selected", "missing");
    storage = await reload();
    expect(localStorage.getItem("selected")).toBeNull();
    expect(storage.isStorageEmpty()).toBe(false);
  });

  it("clears the hint of a workbook that was removed as inconsistent", async () => {
    await rawWrite((workbooks) => {
      workbooks.put(GOOD, "a");
    });
    localStorage.setItem("selected", "a");
    storage = await reload();
    expect(localStorage.getItem("selected")).toBeNull();
  });

  it("keeps a valid hint and opens it on load", async () => {
    await storage.createNewModel();
    await storage.createNewModel();
    const uuid = storage.getSelectedUuid();
    storage = await reload();
    expect(localStorage.getItem("selected")).toBe(uuid);
    // Nothing is active until a workbook is actually opened.
    expect(storage.getSelectedUuid()).toBeNull();
    const model = await storage.loadSelectedModelFromStorage();
    expect(model?.getName()).toBe("Workbook2");
    expect(storage.getSelectedUuid()).toBe(uuid);
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
  it("falls back to the newest workbook when the hint is dangling", async () => {
    await storage.createNewModel();
    const uuid = storage.getSelectedUuid();
    localStorage.setItem("selected", "missing");
    const model = await storage.loadSelectedModelFromStorage();
    expect(model?.getName()).toBe("Workbook1");
    expect(storage.getSelectedUuid()).toBe(uuid);
    expect(localStorage.getItem("selected")).toBe(uuid);
  });

  it("opens the newest workbook when there is no hint", async () => {
    vi.spyOn(Date, "now").mockReturnValue(1000);
    await storage.createNewModel();
    vi.spyOn(Date, "now").mockReturnValue(2000);
    await storage.createNewModel();
    const newest = storage.getSelectedUuid();
    vi.spyOn(Date, "now").mockReturnValue(3000);
    await storage.createNewModel();
    // Delete the newest of all so the hint points nowhere after the reload.
    storage = await reload();
    localStorage.removeItem("selected");
    const model = await storage.loadSelectedModelFromStorage();
    expect(model?.getName()).toBe("Workbook3");
    expect(storage.getSelectedUuid()).not.toBe(newest);
    expect(storage.getSelectedUuid()).toBe(
      Object.keys(storage.getModelsMetadata()).find(
        (uuid) => storage.getModelsMetadata()[uuid].name === "Workbook3",
      ),
    );
  });

  it("returns null when storage is empty", async () => {
    expect(await storage.loadSelectedModelFromStorage()).toBeNull();
    expect(storage.getSelectedUuid()).toBeNull();
  });

  it("removes the workbook and clears the selection when bytes cannot be deserialized", async () => {
    await storage.createNewModel();
    const uuid = storage.getSelectedUuid() as string;
    await corruptBytes(uuid);
    const model = await storage.loadSelectedModelFromStorage();
    expect(model).toBeNull();
    expect(storage.getSelectedUuid()).toBeNull();
    expect(storage.getModelsMetadata()[uuid]).toBeUndefined();
    expect(await rawKeys()).toEqual({ workbooks: [], bytes: [] });
  });
});

describe("unreadable workbooks are removed when opened", () => {
  it("selectModelFromStorage returns null, removes it and keeps the selection", async () => {
    await storage.createNewModel();
    const first = storage.getSelectedUuid() as string;
    await storage.createNewModel();
    const second = storage.getSelectedUuid() as string;
    await corruptBytes(first);
    const listener = vi.fn();
    storage.subscribeToStorage(listener);
    expect(await storage.selectModelFromStorage(first)).toBeNull();
    expect(storage.getSelectedUuid()).toBe(second);
    expect(Object.keys(storage.getModelsMetadata())).toEqual([second]);
    expect(listener).toHaveBeenCalled();
    expect((await rawKeys()).bytes).toEqual([second]);
  });

  it("loadModelFromStorage returns null and removes it", async () => {
    await storage.createNewModel();
    const uuid = storage.getSelectedUuid() as string;
    await corruptBytes(uuid);
    expect(await storage.loadModelFromStorage(uuid)).toBeNull();
    expect(storage.isStorageEmpty()).toBe(true);
    // It was the selected one, so the selection goes too.
    expect(storage.getSelectedUuid()).toBeNull();
  });

  it("duplicateModel returns null and removes it", async () => {
    await storage.createNewModel();
    const uuid = storage.getSelectedUuid() as string;
    await corruptBytes(uuid);
    expect(await storage.duplicateModel(uuid)).toBeNull();
    expect(storage.isStorageEmpty()).toBe(true);
  });

  it("removes a workbook whose bytes vanished", async () => {
    await storage.createNewModel();
    const uuid = storage.getSelectedUuid() as string;
    await rawWrite((_workbooks, bytes) => {
      bytes.delete(uuid);
    });
    expect(await storage.loadModelFromStorage(uuid)).toBeNull();
    expect(storage.isStorageEmpty()).toBe(true);
    expect(storage.getSelectedUuid()).toBeNull();
  });

  it("deleting the selected workbook skips unreadable ones", async () => {
    vi.spyOn(Date, "now").mockReturnValue(1000);
    await storage.createNewModel();
    const first = storage.getSelectedUuid() as string;
    vi.spyOn(Date, "now").mockReturnValue(2000);
    await storage.createNewModel();
    const second = storage.getSelectedUuid() as string;
    vi.spyOn(Date, "now").mockReturnValue(3000);
    await storage.createNewModel();
    const third = storage.getSelectedUuid() as string;
    await corruptBytes(second);

    const result = await storage.deleteModelByUuid(third);
    expect(result?.getName()).toBe("Workbook1");
    expect(storage.getSelectedUuid()).toBe(first);
    expect(Object.keys(storage.getModelsMetadata())).toEqual([first]);
  });

  it("deleting the selected workbook creates a new one when the rest are unreadable", async () => {
    await storage.createNewModel();
    const first = storage.getSelectedUuid() as string;
    await storage.createNewModel();
    const second = storage.getSelectedUuid() as string;
    await corruptBytes(first);

    const result = await storage.deleteModelByUuid(second);
    expect(result?.getName()).toBe("Workbook1");
    const selected = storage.getSelectedUuid() as string;
    expect(selected).not.toBe(first);
    expect(selected).not.toBe(second);
    expect(Object.keys(storage.getModelsMetadata())).toEqual([selected]);
    expect(await rawKeys()).toEqual({
      workbooks: [selected],
      bytes: [selected],
    });
  });
});

describe("the active workbook is per tab", () => {
  it("autosave writes to the workbook this tab opened, not to the shared hint", async () => {
    const first = await storage.createNewModel();
    const firstUuid = storage.getSelectedUuid() as string;
    await storage.createNewModel();
    const secondUuid = storage.getSelectedUuid() as string;
    await storage.selectModelFromStorage(firstUuid);
    // Another tab selects the second workbook.
    localStorage.setItem("selected", secondUuid);

    first.setName("Edited in this tab");
    await storage.saveSelectedModelInStorage(first);

    expect((await storage.loadModelFromStorage(firstUuid))?.getName()).toBe(
      "Edited in this tab",
    );
    expect((await storage.loadModelFromStorage(secondUuid))?.getName()).toBe(
      "Workbook2",
    );
  });

  it("autosave does nothing while showing a workbook that is not stored", async () => {
    const stored = await storage.createNewModel();
    const uuid = storage.getSelectedUuid() as string;
    storage = await reload();
    // Like a shared link: initialized, but nothing opened from storage.
    expect(storage.getSelectedUuid()).toBeNull();
    stored.setName("Remote");
    await storage.saveSelectedModelInStorage(stored);
    expect((await storage.loadModelFromStorage(uuid))?.getName()).toBe(
      "Workbook1",
    );
    // The hint for the next start is untouched.
    expect(localStorage.getItem("selected")).toBe(uuid);
  });

  it("autosave re-creates the workbook if another tab deleted it", async () => {
    const model = await storage.createNewModel();
    const uuid = storage.getSelectedUuid() as string;
    // Another tab deletes it.
    await rawWrite((workbooks, bytes) => {
      workbooks.delete(uuid);
      bytes.delete(uuid);
    });
    model.setName("Still editing");
    await storage.saveSelectedModelInStorage(model);
    expect(storage.getModelsMetadata()[uuid]).toMatchObject({
      name: "Still editing",
      pinned: false,
    });
    storage = await reload();
    expect((await storage.loadModelFromStorage(uuid))?.getName()).toBe(
      "Still editing",
    );
  });
});

describe("cross-tab synchronization", () => {
  // A second instance of the module sharing the same database and
  // localStorage, like another tab.
  async function openOtherTab(): Promise<Storage> {
    vi.resetModules();
    const other: Storage = await import("./storage");
    await other.initStorage();
    return other;
  }

  async function waitFor(condition: () => boolean): Promise<void> {
    for (let i = 0; i < 50 && !condition(); i++) {
      await new Promise((resolve) => setTimeout(resolve, 10));
    }
    expect(condition()).toBe(true);
  }

  it.skipIf(typeof BroadcastChannel === "undefined")(
    "a workbook created in one tab shows up in the other",
    async () => {
      const other = await openOtherTab();
      try {
        await other.createNewModel();
        const uuid = other.getSelectedUuid() as string;
        await waitFor(() => storage.getModelsMetadata()[uuid] !== undefined);
        expect(storage.getModelsMetadata()[uuid].name).toBe("Workbook1");
        // Selection is per tab.
        expect(storage.getSelectedUuid()).toBeNull();
      } finally {
        await other.closeStorage();
      }
    },
  );

  it.skipIf(typeof BroadcastChannel === "undefined")(
    "a workbook deleted in one tab disappears from the other",
    async () => {
      await storage.createNewModel();
      const uuid = storage.getSelectedUuid() as string;
      const other = await openOtherTab();
      try {
        expect(other.getModelsMetadata()[uuid]).toBeDefined();
        await other.deleteModelByUuid(uuid);
        await waitFor(() => storage.getModelsMetadata()[uuid] === undefined);
        // This tab still has it open and keeps saving to it.
        expect(storage.getSelectedUuid()).toBe(uuid);
      } finally {
        await other.closeStorage();
      }
    },
  );
});

describe("without localStorage", () => {
  function denyLocalStorage() {
    const deny = () => {
      throw new DOMException("denied", "SecurityError");
    };
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(deny);
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(deny);
    vi.spyOn(Storage.prototype, "removeItem").mockImplementation(deny);
    vi.spyOn(Storage.prototype, "clear").mockImplementation(deny);
  }

  it("initializes and reads defaults", async () => {
    denyLocalStorage();
    vi.spyOn(console, "warn").mockImplementation(() => {});
    storage = await reload();
    expect(storage.loadThemeModeFromStorage()).toBe("system");
    expect(storage.loadDefaultLocaleFromStorage()).toMatch(
      /^[a-z]{2}-[A-Z]{2}$/,
    );
    expect(storage.getLocalItem("anything")).toBeNull();
  });

  it("still saves and opens workbooks", async () => {
    denyLocalStorage();
    vi.spyOn(console, "warn").mockImplementation(() => {});
    storage = await reload();
    storage.saveThemeModeInStorage("dark");
    const model = await storage.createNewModel();
    const uuid = storage.getSelectedUuid() as string;
    model.setName("Saved");
    await storage.saveSelectedModelInStorage(model);
    expect((await storage.loadModelFromStorage(uuid))?.getName()).toBe("Saved");
    // The hint could not be written, so the next start opens the newest.
    vi.restoreAllMocks();
    storage = await reload();
    expect(localStorage.getItem("selected")).toBeNull();
    expect((await storage.loadSelectedModelFromStorage())?.getName()).toBe(
      "Saved",
    );
  });
});

describe("without IndexedDB", () => {
  function denyIndexedDB() {
    vi.spyOn(indexedDB, "open").mockImplementation(() => {
      throw new DOMException("denied", "SecurityError");
    });
    vi.spyOn(console, "warn").mockImplementation(() => {});
  }

  it("reports that storage is not persistent", async () => {
    expect(storage.isStoragePersistent()).toBe(true);
    denyIndexedDB();
    storage = await reload();
    expect(storage.isStoragePersistent()).toBe(false);
    expect(storage.isStorageEmpty()).toBe(true);
  });

  it("runs every operation in memory", async () => {
    denyIndexedDB();
    storage = await reload();
    const model = await storage.createNewModel();
    const uuid = storage.getSelectedUuid() as string;
    expect(storage.getModelsMetadata()[uuid].name).toBe("Workbook1");

    model.setName("Edited");
    await storage.saveSelectedModelInStorage(model);
    expect((await storage.loadModelFromStorage(uuid))?.getName()).toBe(
      "Edited",
    );

    await storage.updateNameSelectedWorkbook(model, "Renamed");
    expect(storage.getModelsMetadata()[uuid].name).toBe("Renamed");
    await storage.togglePinWorkbook(uuid);
    expect(storage.isWorkbookPinned(uuid)).toBe(true);

    const copy = await storage.duplicateModel(uuid);
    expect(copy?.getName()).toBe("Renamed (1)");
    expect(Object.keys(storage.getModelsMetadata())).toHaveLength(2);

    // Strictly newer than the copy, so it is the one selected after the delete.
    vi.spyOn(Date, "now").mockReturnValue(Date.now() + 10_000);
    await storage.createNewModel();
    const third = storage.getSelectedUuid() as string;
    expect(await storage.selectModelFromStorage(uuid)).not.toBeNull();
    expect(storage.getSelectedUuid()).toBe(uuid);
    const next = await storage.deleteModelByUuid(uuid);
    expect(next).not.toBeNull();
    expect(storage.getSelectedUuid()).toBe(third);
    expect(storage.getModelsMetadata()[uuid]).toBeUndefined();
  });

  it("persists nothing", async () => {
    denyIndexedDB();
    storage = await reload();
    await storage.createNewModel();
    expect(storage.isStorageEmpty()).toBe(false);
    vi.restoreAllMocks();
    storage = await reload();
    expect(storage.isStoragePersistent()).toBe(true);
    expect(storage.isStorageEmpty()).toBe(true);
    // The hint written in memory mode points nowhere and is dropped.
    expect(localStorage.getItem("selected")).toBeNull();
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
    // Distinct timestamps so "newest remaining" is well defined.
    vi.spyOn(Date, "now").mockReturnValue(1000);
    await storage.createNewModel();
    const a = storage.getSelectedUuid() as string;
    vi.spyOn(Date, "now").mockReturnValue(2000);
    await storage.createNewModel();
    const b = storage.getSelectedUuid() as string;
    vi.spyOn(Date, "now").mockReturnValue(3000);
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
