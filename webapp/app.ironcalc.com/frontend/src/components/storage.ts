import { Model } from "@ironcalc/workbook";
import i18n from "../i18n";

const MAX_WORKBOOKS = 50;

// ---------------------------------------------------------------------------
// Storage layout
//
// Small preferences that must be available synchronously (before the wasm
// module is initialized or during render) live in localStorage:
//   - "default_locale": UI language
//   - "dark_mode": theme preference
//   - "selected": uuid of the workbook currently open
//
// Everything that scales with user data lives in IndexedDB:
//   - "workbooks" store: one metadata record per workbook (name, createdAt,
//     pinned), keyed by uuid. Loaded once at startup into an in-memory cache
//     so the UI can read it synchronously during render.
//   - "workbook_bytes" store: the serialized workbook (Uint8Array), keyed by
//     uuid. Only ever read on demand.
//
// Every operation that writes both stores does so in a single transaction so
// metadata and bytes can never get out of sync. Mutating operations are
// serialized through a queue so concurrent callers (e.g. the autosave timer
// and a delete) cannot interleave.
//
// Components subscribe to changes with `subscribeToStorage` (see
// useStorage.ts) and are notified whenever the metadata cache or the selected
// uuid changes.
// ---------------------------------------------------------------------------

const DB_NAME = "ironcalc";
const DB_VERSION = 1;
const WORKBOOKS_STORE = "workbooks";
const BYTES_STORE = "workbook_bytes";

const SELECTED_KEY = "selected";
const DEFAULT_LOCALE_KEY = "default_locale";
const DARK_MODE_KEY = "dark_mode";

export interface WorkbookMetadata {
  name: string;
  createdAt: number;
  pinned: boolean;
}

export type ModelsMetadata = Readonly<Record<string, WorkbookMetadata>>;

// ---------------------------------------------------------------------------
// IndexedDB helpers
// ---------------------------------------------------------------------------

let dbPromise: Promise<IDBDatabase> | null = null;

function deleteDatabase(): Promise<void> {
  return new Promise((resolve) => {
    const request = indexedDB.deleteDatabase(DB_NAME);
    request.onsuccess = () => resolve();
    request.onerror = () => resolve();
    // Other tabs close their connection on `versionchange`, so the deletion
    // goes through shortly; a subsequent `open` is queued behind it anyway.
    request.onblocked = () => resolve();
  });
}

// Opens the database once. Rejects if it cannot be opened (e.g. it was
// created by a newer version of the app) or if it does not have the expected
// object stores.
function tryOpenDatabase(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);
    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains(WORKBOOKS_STORE)) {
        db.createObjectStore(WORKBOOKS_STORE);
      }
      if (!db.objectStoreNames.contains(BYTES_STORE)) {
        db.createObjectStore(BYTES_STORE);
      }
    };
    request.onsuccess = () => {
      const db = request.result;
      if (
        !db.objectStoreNames.contains(WORKBOOKS_STORE) ||
        !db.objectStoreNames.contains(BYTES_STORE)
      ) {
        db.close();
        reject(new Error("IndexedDB database is missing object stores"));
        return;
      }
      // If another tab upgrades (or deletes) the database, drop our
      // connection so the next call reopens it.
      db.onversionchange = () => {
        db.close();
        dbPromise = null;
      };
      resolve(db);
    };
    request.onerror = () => reject(request.error);
  });
}

function openDatabase(): Promise<IDBDatabase> {
  if (dbPromise) {
    return dbPromise;
  }
  const pending = tryOpenDatabase().catch(async (e) => {
    // The database is unusable: throw it away and start from scratch.
    console.warn("Resetting IndexedDB storage", e);
    await deleteDatabase();
    return tryOpenDatabase();
  });
  pending.catch(() => {
    if (dbPromise === pending) {
      dbPromise = null;
    }
  });
  dbPromise = pending;
  return dbPromise;
}

// Closes the open connection (if any). The next storage call reopens it.
// Mainly useful for tests.
export async function closeStorage(): Promise<void> {
  const pending = dbPromise;
  dbPromise = null;
  if (!pending) {
    return;
  }
  try {
    (await pending).close();
  } catch {
    // The connection failed to open; nothing to close.
  }
}

function requestToPromise<T>(request: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

function transactionDone(tx: IDBTransaction): Promise<void> {
  return new Promise((resolve, reject) => {
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
    tx.onabort = () => reject(tx.error);
  });
}

// Runs `body` inside a single readwrite transaction spanning both stores and
// resolves once the transaction has committed.
async function writeTransaction(
  body: (workbooks: IDBObjectStore, bytes: IDBObjectStore) => void,
): Promise<void> {
  const db = await openDatabase();
  const tx = db.transaction([WORKBOOKS_STORE, BYTES_STORE], "readwrite");
  const done = transactionDone(tx);
  body(tx.objectStore(WORKBOOKS_STORE), tx.objectStore(BYTES_STORE));
  return done;
}

async function readBytes(uuid: string): Promise<Uint8Array | undefined> {
  const db = await openDatabase();
  const tx = db.transaction(BYTES_STORE, "readonly");
  return requestToPromise<Uint8Array | undefined>(
    tx.objectStore(BYTES_STORE).get(uuid) as IDBRequest<Uint8Array | undefined>,
  );
}

function isValidMetadata(value: unknown): value is WorkbookMetadata {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const { name, createdAt, pinned } = value as Record<string, unknown>;
  return (
    typeof name === "string" &&
    typeof createdAt === "number" &&
    Number.isFinite(createdAt) &&
    typeof pinned === "boolean"
  );
}

// Reads every metadata record. Records that are not in the expected shape
// are deleted (together with their bytes) and left out of the result.
async function readAllMetadata(): Promise<Record<string, WorkbookMetadata>> {
  const db = await openDatabase();
  const tx = db.transaction(WORKBOOKS_STORE, "readonly");
  const store = tx.objectStore(WORKBOOKS_STORE);
  const [keys, values] = await Promise.all([
    requestToPromise(store.getAllKeys()),
    requestToPromise(store.getAll() as IDBRequest<unknown[]>),
  ]);
  const result: Record<string, WorkbookMetadata> = {};
  const malformed: IDBValidKey[] = [];
  keys.forEach((key, index) => {
    const value = values[index];
    if (typeof key === "string" && isValidMetadata(value)) {
      result[key] = value;
    } else {
      malformed.push(key);
    }
  });
  if (malformed.length > 0) {
    console.warn("Deleting malformed workbooks from storage", malformed);
    await writeTransaction((workbooks, bytesStore) => {
      for (const key of malformed) {
        workbooks.delete(key);
        bytesStore.delete(key);
      }
    });
  }
  return result;
}

// ---------------------------------------------------------------------------
// In-memory cache + subscriptions
// ---------------------------------------------------------------------------

// The cache object is replaced (never mutated) on every change so that React
// can compare snapshots by reference.
let metadataCache: ModelsMetadata = {};

type Listener = () => void;
const listeners = new Set<Listener>();

function notify() {
  for (const listener of listeners) {
    listener();
  }
}

export function subscribeToStorage(listener: Listener): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

function setMetadataCache(next: Record<string, WorkbookMetadata>) {
  metadataCache = next;
  notify();
}

function withMetadata(
  uuid: string,
  metadata: WorkbookMetadata,
): Record<string, WorkbookMetadata> {
  return { ...metadataCache, [uuid]: metadata };
}

function withoutMetadata(uuid: string): Record<string, WorkbookMetadata> {
  const next: Record<string, WorkbookMetadata> = { ...metadataCache };
  delete next[uuid];
  return next;
}

// Serializes mutating operations so they never interleave.
let queue: Promise<unknown> = Promise.resolve();

function serialized<T>(operation: () => Promise<T>): Promise<T> {
  const result = queue.then(operation, operation);
  queue = result.catch(() => undefined);
  return result;
}

// Hydrates the in-memory cache from IndexedDB. Must be awaited once at app
// startup before any of the synchronous getters are used.
export async function initStorage(): Promise<void> {
  let metadata: Record<string, WorkbookMetadata> = {};
  try {
    metadata = await readAllMetadata();
  } catch (e) {
    // Reading failed even though the database opened: delete it and retry
    // once with a fresh, empty database.
    console.warn("Failed to read storage, resetting it", e);
    await closeStorage();
    await deleteDatabase();
    try {
      metadata = await readAllMetadata();
    } catch (e2) {
      console.warn("Failed to initialize storage", e2);
    }
  }
  setMetadataCache(metadata);
  // A selection that does not point to a known workbook is meaningless.
  const selected = getSelectedUuid();
  if (selected !== null && !metadata[selected]) {
    clearSelectedUuid();
  }
}

// ---------------------------------------------------------------------------
// Preferences (localStorage)
// ---------------------------------------------------------------------------

// Returns the default UI language based on the browser settings
// ['en-US', 'en-GB', 'es-ES', 'fr-FR', 'de-DE', 'it-IT']
function getDefaultUILocale(): string {
  const lang = navigator.language || navigator.languages[0] || "en-US";
  if (lang.startsWith("es")) {
    return "es-ES";
  } else if (lang.startsWith("fr")) {
    return "fr-FR";
  } else if (lang.startsWith("de")) {
    return "de-DE";
  } else if (lang === "en-GB") {
    return "en-GB";
  } else if (lang.startsWith("it")) {
    return "it-IT";
  }

  return "en-US";
}

// Converts long language codes to short ones used by the Model
export function getShortLocaleCode(longCode: string): string {
  switch (longCode) {
    case "es-ES": {
      return "es";
    }
    case "fr-FR": {
      return "fr";
    }
    case "de-DE": {
      return "de";
    }
    case "it-IT": {
      return "it";
    }
    case "en-GB": {
      return "en-GB";
    }
    default: {
      return "en";
    }
  }
}

// en-US => en, en-GB => en, es-ES => es, fr-FR => fr, de-DE => de, it-IT => it
export function getLanguageFromLocale(locale: string): string {
  return locale.split("-")[0];
}

export function saveDefaultLocaleInStorage(locale: string) {
  localStorage.setItem(DEFAULT_LOCALE_KEY, locale);
}

const SUPPORTED_LOCALES = [
  "en-US",
  "en-GB",
  "es-ES",
  "fr-FR",
  "de-DE",
  "it-IT",
];

export function loadDefaultLocaleFromStorage(): string {
  const lang = localStorage.getItem(DEFAULT_LOCALE_KEY);
  if (lang && SUPPORTED_LOCALES.includes(lang)) {
    return lang;
  }
  const l = getDefaultUILocale();
  saveDefaultLocaleInStorage(l);
  return l;
}

export type ThemeMode = "light" | "dark";

export function saveThemeModeInStorage(mode: ThemeMode) {
  localStorage.setItem(DARK_MODE_KEY, mode);
}

export function loadThemeModeFromStorage(): ThemeMode {
  return localStorage.getItem(DARK_MODE_KEY) === "dark" ? "dark" : "light";
}

export function getSelectedUuid(): string | null {
  return localStorage.getItem(SELECTED_KEY);
}

function setSelectedUuid(uuid: string) {
  localStorage.setItem(SELECTED_KEY, uuid);
  notify();
}

export function clearSelectedUuid() {
  localStorage.removeItem(SELECTED_KEY);
  notify();
}

// ---------------------------------------------------------------------------
// Workbooks
// ---------------------------------------------------------------------------

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

export function getModelsMetadata(): ModelsMetadata {
  return metadataCache;
}

// check if storage is empty
export function isStorageEmpty(): boolean {
  return Object.keys(metadataCache).length === 0;
}

export function isWorkbookPinned(uuid: string): boolean {
  return metadataCache[uuid]?.pinned || false;
}

// Pick a different name Workbook{N} where N = 1, 2, 3
function getNewName(existingNames: string[]): string {
  const baseName = i18n.t("default_workbook_name");
  let index = 1;
  while (index < MAX_WORKBOOKS) {
    const name = `${baseName}${index}`;
    index += 1;
    if (!existingNames.includes(name)) {
      return name;
    }
  }
  // FIXME: Too many workbooks?
  return `${baseName}-Infinity`;
}

export function createModelWithSafeTimezone(name: string): Model {
  const locale = loadDefaultLocaleFromStorage();
  const language = locale.split("-")[0];
  const localeShort = getShortLocaleCode(locale);
  try {
    const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
    return new Model(name, localeShort, tz, language);
  } catch (e) {
    console.warn("Failed to get timezone, defaulting to UTC", e);
    return new Model(name, localeShort, "UTC", language);
  }
}

// Stores a brand new workbook (bytes + metadata) and optionally selects it.
async function insertWorkbook(
  model: Model,
  name: string,
  select: boolean,
): Promise<void> {
  const uuid = randomUUID();
  const metadata: WorkbookMetadata = {
    name,
    createdAt: Date.now(),
    pinned: false,
  };
  const bytes = model.toBytes();
  await writeTransaction((workbooks, bytesStore) => {
    workbooks.put(metadata, uuid);
    bytesStore.put(bytes, uuid);
  });
  setMetadataCache(withMetadata(uuid, metadata));
  if (select) {
    setSelectedUuid(uuid);
  }
}

function newestUuid(metadata: ModelsMetadata): string | null {
  const uuids = Object.keys(metadata);
  if (uuids.length === 0) {
    return null;
  }
  // On equal timestamps the later entry wins, so workbooks created within
  // the same millisecond still resolve in creation order.
  return uuids.reduce((newest, current) => {
    const newestTime = metadata[newest]?.createdAt || 0;
    const currentTime = metadata[current]?.createdAt || 0;
    return currentTime >= newestTime ? current : newest;
  });
}

// Loads a workbook. Returns null if it does not exist. A workbook whose bytes
// cannot be deserialized is deleted from storage and null is returned.
async function loadModel(uuid: string): Promise<Model | null> {
  const bytes = await readBytes(uuid);
  if (!bytes) {
    return null;
  }
  const language = getLanguageFromLocale(loadDefaultLocaleFromStorage());
  try {
    return Model.fromBytes(bytes, language);
  } catch (e) {
    console.warn("Deleting workbook that could not be deserialized", uuid, e);
    await writeTransaction((workbooks, bytesStore) => {
      workbooks.delete(uuid);
      bytesStore.delete(uuid);
    });
    setMetadataCache(withoutMetadata(uuid));
    if (getSelectedUuid() === uuid) {
      clearSelectedUuid();
    }
    return null;
  }
}

// Creates an empty workbook with a fresh name, stores it and selects it.
export function createNewModel(): Promise<Model> {
  return serialized(async () => {
    const name = getNewName(Object.values(metadataCache).map((m) => m.name));
    const model = createModelWithSafeTimezone(name);
    await insertWorkbook(model, name, true);
    return model;
  });
}

// Stores an existing model (e.g. an uploaded file) as a new workbook and
// selects it.
export function saveModelToStorage(model: Model): Promise<void> {
  return serialized(() => insertWorkbook(model, model.getName(), true));
}

// Loads the currently selected workbook. Returns null if there is none or it
// could not be loaded (in which case the selection is cleared).
export function loadSelectedModelFromStorage(): Promise<Model | null> {
  return serialized(async () => {
    const uuid = getSelectedUuid();
    if (!uuid) {
      return null;
    }
    try {
      const model = await loadModel(uuid);
      if (!model) {
        clearSelectedUuid();
      }
      return model;
    } catch (e) {
      console.warn("Failed to load selected model from storage", e);
      clearSelectedUuid();
      return null;
    }
  });
}

// Loads a workbook without changing the selection.
export function loadModelFromStorage(uuid: string): Promise<Model | null> {
  return serialized(() => loadModel(uuid));
}

// Loads a workbook and makes it the selected one.
export function selectModelFromStorage(uuid: string): Promise<Model | null> {
  return serialized(async () => {
    const model = await loadModel(uuid);
    if (model) {
      setSelectedUuid(uuid);
    }
    return model;
  });
}

// Persists the bytes of the selected workbook.
export function saveSelectedModelInStorage(model: Model): Promise<void> {
  return serialized(async () => {
    const uuid = getSelectedUuid();
    if (!uuid || !metadataCache[uuid]) {
      return;
    }
    const bytes = model.toBytes();
    await writeTransaction((_workbooks, bytesStore) => {
      bytesStore.put(bytes, uuid);
    });
  });
}

export function updateNameSelectedWorkbook(
  model: Model,
  newName: string,
): Promise<void> {
  return serialized(async () => {
    const uuid = getSelectedUuid();
    if (!uuid) {
      return;
    }
    const metadata: WorkbookMetadata = {
      ...(metadataCache[uuid] ?? { createdAt: Date.now(), pinned: false }),
      name: newName,
    };
    const bytes = model.toBytes();
    await writeTransaction((workbooks, bytesStore) => {
      workbooks.put(metadata, uuid);
      bytesStore.put(bytes, uuid);
    });
    setMetadataCache(withMetadata(uuid, metadata));
  });
}

export function togglePinWorkbook(uuid: string): Promise<void> {
  return serialized(async () => {
    const current = metadataCache[uuid];
    if (!current) {
      return;
    }
    const metadata: WorkbookMetadata = { ...current, pinned: !current.pinned };
    await writeTransaction((workbooks) => {
      workbooks.put(metadata, uuid);
    });
    setMetadataCache(withMetadata(uuid, metadata));
  });
}

// Deletes a workbook. If it was the selected one, the newest remaining
// workbook is selected and returned (a new one is created if none is left).
// Otherwise returns null and the selection is untouched.
export function deleteModelByUuid(uuid: string): Promise<Model | null> {
  return serialized(async () => {
    const wasSelected = getSelectedUuid() === uuid;
    await writeTransaction((workbooks, bytesStore) => {
      workbooks.delete(uuid);
      bytesStore.delete(uuid);
    });
    setMetadataCache(withoutMetadata(uuid));
    if (!wasSelected) {
      return null;
    }
    clearSelectedUuid();
    const newest = newestUuid(metadataCache);
    if (!newest) {
      const name = getNewName([]);
      const model = createModelWithSafeTimezone(name);
      await insertWorkbook(model, name, true);
      return model;
    }
    const model = await loadModel(newest);
    if (model) {
      setSelectedUuid(newest);
    }
    return model;
  });
}

export function deleteSelectedModel(): Promise<Model | null> {
  const uuid = getSelectedUuid();
  if (!uuid) {
    return Promise.resolve(null);
  }
  return deleteModelByUuid(uuid);
}

// Duplicates a workbook. The copy is stored but not selected.
export function duplicateModel(uuid: string): Promise<Model | null> {
  return serialized(async () => {
    const original = metadataCache[uuid];
    if (!original) {
      return null;
    }
    const duplicated = await loadModel(uuid);
    if (!duplicated) {
      return null;
    }
    const existingNames = Object.values(metadataCache).map((m) => m.name);

    // Find next available number
    let counter = 1;
    let newName = `${original.name} (${counter})`;
    while (existingNames.includes(newName)) {
      counter++;
      newName = `${original.name} (${counter})`;
    }
    duplicated.setName(newName);
    await insertWorkbook(duplicated, newName, false);
    return duplicated;
  });
}
