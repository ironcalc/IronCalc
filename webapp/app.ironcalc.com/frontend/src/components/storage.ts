import { Model } from "@ironcalc/workbook";
import i18n from "../i18n";

const MAX_WORKBOOKS = 50;

// ---------------------------------------------------------------------------
// Storage layout
//
// Small preferences that must be available synchronously (before the wasm
// module is initialized or during render) live in localStorage:
//   - "storage_version": layout version of the keys below. When it does not
//     match, `initLocalStorage` wipes localStorage (earlier versions kept the
//     workbooks themselves there) and stamps the current version.
//   - "default_locale": UI language
//   - "dark_mode": theme preference
//   - "selected": uuid of the workbook currently open
//   - "localStorageAlertDismissed": the drawer warning was closed
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
// Still, nothing stops another version of the app (or a crash in the middle
// of an upgrade) from leaving the database in a state this code does not
// expect, so `initStorage` checks both stores once at startup and removes
// anything inconsistent: see `readAndRepairMetadata`. Bytes are only checked
// when a workbook is opened: one that fails to deserialize is removed on the
// spot (see `openWorkbook`). There is no backwards compatibility promise for
// the byte format yet, so keeping unreadable workbooks around has no value.
//
// Components subscribe to changes with `subscribeToStorage` (see
// useStorage.ts) and are notified whenever the metadata cache or the selected
// uuid changes.
// ---------------------------------------------------------------------------

const DB_NAME = "ironcalc";
const DB_VERSION = 1;
const WORKBOOKS_STORE = "workbooks";
const BYTES_STORE = "workbook_bytes";

const STORAGE_VERSION_KEY = "storage_version";
const STORAGE_VERSION = "1";
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

function openDatabase(): Promise<IDBDatabase> {
  if (dbPromise) {
    return dbPromise;
  }
  dbPromise = new Promise((resolve, reject) => {
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
      // If another tab upgrades the database, drop our connection so the
      // next call reopens it.
      db.onversionchange = () => {
        db.close();
        dbPromise = null;
      };
      resolve(db);
    };
    request.onerror = () => {
      dbPromise = null;
      reject(request.error);
    };
  });
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

// Returns a well-formed copy of a stored metadata record, or null if it is
// beyond repair. A record needs a name; a missing or invalid `createdAt` or
// `pinned` gets a default.
function sanitizeMetadata(value: unknown): WorkbookMetadata | null {
  if (typeof value !== "object" || value === null) {
    return null;
  }
  const { name, createdAt, pinned } = value as Record<string, unknown>;
  if (typeof name !== "string") {
    return null;
  }
  return {
    name,
    createdAt:
      typeof createdAt === "number" && Number.isFinite(createdAt)
        ? createdAt
        : Date.now(),
    pinned: pinned === true,
  };
}

function sameMetadata(a: WorkbookMetadata, b: WorkbookMetadata): boolean {
  return (
    a.name === b.name && a.createdAt === b.createdAt && a.pinned === b.pinned
  );
}

// Reads all workbook metadata and, in the same transaction, brings both
// stores back to a consistent state:
//   - a workbook must be keyed by a string, have usable metadata and have
//     bytes; otherwise it is removed from both stores
//   - metadata with missing fields is rewritten with defaults
//   - bytes that belong to no workbook are removed
// The bytes themselves are not decoded here (that would mean loading every
// workbook); a workbook whose bytes are corrupt is dealt with when opened.
async function readAndRepairMetadata(): Promise<
  Record<string, WorkbookMetadata>
> {
  const db = await openDatabase();
  const tx = db.transaction([WORKBOOKS_STORE, BYTES_STORE], "readwrite");
  const done = transactionDone(tx);
  const workbooks = tx.objectStore(WORKBOOKS_STORE);
  const bytes = tx.objectStore(BYTES_STORE);
  const [keys, values, bytesKeys] = await Promise.all([
    requestToPromise(workbooks.getAllKeys()),
    requestToPromise(workbooks.getAll() as IDBRequest<unknown[]>),
    requestToPromise(bytes.getAllKeys()),
  ]);

  const bytesKeySet = new Set(bytesKeys.filter((k) => typeof k === "string"));
  const result: Record<string, WorkbookMetadata> = {};
  let removed = 0;
  let repaired = 0;

  keys.forEach((key, index) => {
    const metadata =
      typeof key === "string" && bytesKeySet.has(key)
        ? sanitizeMetadata(values[index])
        : null;
    if (metadata === null) {
      workbooks.delete(key);
      removed += 1;
      return;
    }
    const uuid = key as string;
    if (!sameMetadata(metadata, values[index] as WorkbookMetadata)) {
      workbooks.put(metadata, uuid);
      repaired += 1;
    }
    result[uuid] = metadata;
  });

  for (const key of bytesKeys) {
    if (typeof key !== "string" || !(key in result)) {
      bytes.delete(key);
      removed += 1;
    }
  }

  await done;
  if (removed > 0 || repaired > 0) {
    console.warn(
      `Storage check: removed ${removed} inconsistent record(s), repaired ${repaired}`,
    );
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

// Brings localStorage to the current layout: anything written by a different
// version is wiped. Idempotent and synchronous, so it can (and should) run
// before the first render, which reads the preferences synchronously.
export function initLocalStorage(): void {
  if (localStorage.getItem(STORAGE_VERSION_KEY) === STORAGE_VERSION) {
    return;
  }
  localStorage.clear();
  localStorage.setItem(STORAGE_VERSION_KEY, STORAGE_VERSION);
}

// Hydrates the in-memory cache from IndexedDB. Must be awaited once at app
// startup before any of the synchronous getters are used.
export async function initStorage(): Promise<void> {
  initLocalStorage();
  try {
    setMetadataCache(await readAndRepairMetadata());
  } catch (e) {
    console.warn("Failed to initialize storage", e);
    setMetadataCache({});
  }
  // The selection must point at a workbook we know about.
  const selected = getSelectedUuid();
  if (selected !== null && !metadataCache[selected]) {
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

export function loadDefaultLocaleFromStorage(): string {
  const lang = localStorage.getItem(DEFAULT_LOCALE_KEY);
  if (lang) {
    return lang;
  }
  const l = getDefaultUILocale();
  saveDefaultLocaleInStorage(l);
  return l;
}

/** "system" follows the OS colour scheme (and tracks its changes). */
export type ThemeMode = "light" | "dark" | "system";

export function saveThemeModeInStorage(mode: ThemeMode) {
  localStorage.setItem(DARK_MODE_KEY, mode);
}

export function loadThemeModeFromStorage(): ThemeMode {
  const stored = localStorage.getItem(DARK_MODE_KEY);
  switch (stored) {
    case "light":
    case "dark":
    case "system":
      return stored;
    default:
      return "system";
  }
}

const DARK_SCHEME_QUERY = "(prefers-color-scheme: dark)";

export function systemPrefersDark(): boolean {
  return window.matchMedia?.(DARK_SCHEME_QUERY).matches ?? false;
}

/** Calls `onChange` whenever the OS colour scheme flips; returns the unsubscribe. */
export function watchSystemDarkMode(
  onChange: (isDark: boolean) => void,
): () => void {
  const query = window.matchMedia?.(DARK_SCHEME_QUERY);
  if (!query) {
    return () => {};
  }
  const listener = (event: MediaQueryListEvent) => onChange(event.matches);
  query.addEventListener("change", listener);
  return () => query.removeEventListener("change", listener);
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
  return uuids.reduce((newest, current) => {
    const newestTime = metadata[newest]?.createdAt || 0;
    const currentTime = metadata[current]?.createdAt || 0;
    return currentTime > newestTime ? current : newest;
  });
}

// Removes a workbook that turned out to be unreadable, together with its
// selection if it was the selected one.
async function yankWorkbook(uuid: string, reason: unknown): Promise<void> {
  console.warn(`Removing unreadable workbook ${uuid}`, reason);
  try {
    await writeTransaction((workbooks, bytesStore) => {
      workbooks.delete(uuid);
      bytesStore.delete(uuid);
    });
  } catch (e) {
    console.warn(`Failed to remove workbook ${uuid}`, e);
  }
  setMetadataCache(withoutMetadata(uuid));
  if (getSelectedUuid() === uuid) {
    clearSelectedUuid();
  }
}

// The one way to get a Model out of the database. Returns null if there is
// no such workbook or if it could not be deserialized, in which case the
// workbook is removed. Must run inside `serialized`.
async function openWorkbook(uuid: string): Promise<Model | null> {
  const bytes = await readBytes(uuid);
  if (bytes === undefined) {
    if (metadataCache[uuid]) {
      await yankWorkbook(uuid, "no bytes");
    }
    return null;
  }
  const language = getLanguageFromLocale(loadDefaultLocaleFromStorage());
  try {
    return Model.fromBytes(bytes, language);
  } catch (e) {
    await yankWorkbook(uuid, e);
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

// Loads the currently selected workbook. Returns null, and clears the
// selection, if there is none or it could not be loaded.
export function loadSelectedModelFromStorage(): Promise<Model | null> {
  return serialized(async () => {
    const uuid = getSelectedUuid();
    if (!uuid) {
      return null;
    }
    const model = await openWorkbook(uuid);
    if (!model) {
      clearSelectedUuid();
    }
    return model;
  });
}

// Loads a workbook without changing the selection.
export function loadModelFromStorage(uuid: string): Promise<Model | null> {
  return serialized(() => openWorkbook(uuid));
}

// Loads a workbook and makes it the selected one. The selection is untouched
// if the workbook could not be loaded.
export function selectModelFromStorage(uuid: string): Promise<Model | null> {
  return serialized(async () => {
    const model = await openWorkbook(uuid);
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
// readable workbook is selected and returned (a new one is created if none is
// left). Otherwise returns null and the selection is untouched.
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
    // Unreadable workbooks are removed by openWorkbook, so each iteration
    // shrinks the candidates.
    let newest = newestUuid(metadataCache);
    while (newest) {
      const model = await openWorkbook(newest);
      if (model) {
        setSelectedUuid(newest);
        return model;
      }
      newest = newestUuid(metadataCache);
    }
    const name = getNewName([]);
    const model = createModelWithSafeTimezone(name);
    await insertWorkbook(model, name, true);
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
    const duplicated = await openWorkbook(uuid);
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
