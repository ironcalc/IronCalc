import "./App.css";
import type { IronCalcHandle } from "@ironcalc/workbook";
// From IronCalc
import {
  darkThemeVariables,
  IronCalc,
  IronCalcIcon,
  init,
  Model,
} from "@ironcalc/workbook";
import "@ironcalc/workbook/style.css";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { FileBar } from "./components/FileBar";
import LeftDrawer from "./components/LeftDrawer/LeftDrawer";
import { NoticeBar } from "./components/NoticeBar";
import { get_documentation_model, get_model } from "./components/rpc";
import {
  clearSelectedUuid,
  createModelFromFile,
  createModelWithSafeTimezone,
  createNewModel,
  deleteModelByUuid,
  deleteSelectedModel,
  getLanguageFromLocale,
  getShortLocaleCode,
  initStorage,
  isStorageEmpty,
  loadDefaultLocaleFromStorage,
  loadSelectedModelFromStorage,
  loadThemeModeFromStorage,
  saveDefaultLocaleInStorage,
  saveModelToStorage,
  saveSelectedModelInStorage,
  saveThemeModeInStorage,
  selectModelFromStorage,
  subscribeToStorage,
  systemPrefersDark,
  type ThemeMode,
  watchSystemDarkMode,
} from "./components/storage";
import TemplatesDialog from "./components/WelcomeDialog/TemplatesDialog";
import WelcomeDialog from "./components/WelcomeDialog/WelcomeDialog";

function App() {
  const [model, setModel] = useState<Model | null>(null);
  const [showWelcomeDialog, setShowWelcomeDialog] = useState(false);
  const [isTemplatesDialogOpen, setTemplatesDialogOpen] = useState(false);
  const [isDrawerOpen, setIsDrawerOpen] = useState(false);
  const [themeMode, setThemeMode] = useState<ThemeMode>(
    loadThemeModeFromStorage,
  );
  const [systemIsDark, setSystemIsDark] = useState(systemPrefersDark);
  useEffect(() => watchSystemDarkMode(setSystemIsDark), []);
  const isDarkMode =
    themeMode === "system" ? systemIsDark : themeMode === "dark";

  const ironCalcRef = useRef<IronCalcHandle>(null);
  // A workbook opened from a link (?model= or ?example=) stays read-only until
  // a copy is saved. Every handler that opens another workbook must reset it.
  const [isReadOnly, setIsReadOnly] = useState(false);
  // A workbook above MAX_PERSISTED_WORKBOOK_BYTES is kept in memory only:
  // serializing it for IndexedDB needs more memory than wasm32 allows. It can
  // be edited, but nothing is saved, shared or exported.
  const [isSessionOnly, setIsSessionOnly] = useState(false);
  // Guards against overlapping autosaves and repeated error alerts.
  const isSavingRef = useRef(false);
  const saveErrorReportedRef = useRef(false);

  const { t, i18n } = useTranslation();

  const reportSaveError = useCallback(
    (e: unknown) => {
      console.error("Failed to save workbook", e);
      if (!saveErrorReportedRef.current) {
        saveErrorReportedRef.current = true;
        alert(t("errors.storage_save_failed"));
      }
    },
    [t],
  );

  const handleLanguageChange = (language: string) => {
    if (ironCalcRef.current) {
      ironCalcRef.current.setLanguage(language);
      saveDefaultLocaleInStorage(language);
      if (model) {
        model.setLocale(getShortLocaleCode(language));
        saveSelectedModelInStorage(model).catch(reportSaveError);
      }
    }
  };

  const handleThemeModeChange = (mode: ThemeMode) => {
    setThemeMode(mode);
    saveThemeModeInStorage(mode);
  };

  // biome-ignore lint/correctness/useExhaustiveDependencies: Run only for i18n.language dependency
  useEffect(() => {
    async function start() {
      await init();
      await initStorage();
      const queryString = window.location.search;
      const urlParams = new URLSearchParams(queryString);
      const modelHash = urlParams.get("model");
      const exampleFilename = urlParams.get("example");
      const language = loadDefaultLocaleFromStorage();
      const languageId = getLanguageFromLocale(language);
      // If there is a model name ?model=modelHash we try to load it
      // if there is not, or the loading failed we load an empty model
      let loadedModel: Model | null = null;
      if (modelHash) {
        // Get a remote model
        try {
          const model_bytes = await get_model(modelHash);
          loadedModel = Model.fromBytes(model_bytes, languageId);
          clearSelectedUuid();
        } catch (_e) {
          console.error(_e);
          alert(t("errors.model_not_found"));
          console.log("Failed to load model from hash:", modelHash);
        }
      } else if (exampleFilename) {
        try {
          const model_bytes = await get_documentation_model(exampleFilename);
          loadedModel = Model.fromBytes(model_bytes, languageId);
          clearSelectedUuid();
        } catch (_e) {
          console.error(_e);
          alert(t("errors.example_not_found"));
          console.log("Failed to load example model:", exampleFilename);
        }
      }

      if (loadedModel) {
        setModel(loadedModel);
        setIsReadOnly(true);
      } else {
        // try to load from storage
        const result = await loadSelectedModelFromStorage();
        if (result) {
          setModel(result);
        } else if (isStorageEmpty()) {
          setShowWelcomeDialog(true);
          const createdModel = createModelWithSafeTimezone("template");
          setModel(createdModel);
        } else {
          // There are workbooks but none could be selected: start a new one
          setModel(await createNewModel());
        }
      }
      i18n.changeLanguage(language);
      setTimeout(() => {
        if (ironCalcRef.current) {
          ironCalcRef.current.setLanguage(language);
        }
      }, 0);
    }
    start();
  }, [i18n.changeLanguage]);

  // Keeps the tab title in sync with the workbook name. We subscribe to the
  // storage directly instead of using a hook so that a rename (or any other
  // storage change) does not re-render App and, with it, the whole sheet.
  useEffect(() => {
    const update = () => {
      const workbookName = model?.getName();
      document.title = workbookName ? `${workbookName} - IronCalc` : "IronCalc";
    };
    update();
    return subscribeToStorage(update);
  }, [model]);

  useEffect(() => {
    if (!model || isSessionOnly) return;
    // We try to save the model every second
    const interval = setInterval(() => {
      if (isSavingRef.current) {
        // The previous save is still running; changes are picked up next tick
        return;
      }
      const queue = model.flushSendQueue();
      if (queue.length !== 1) {
        isSavingRef.current = true;
        saveSelectedModelInStorage(model)
          .catch(reportSaveError)
          .finally(() => {
            isSavingRef.current = false;
          });
      }
    }, 1000);
    return () => clearInterval(interval);
  }, [model, isSessionOnly, reportSaveError]);

  if (!model) {
    return (
      <div className="app-ic-loading">
        <IronCalcIcon style={{ width: 24, height: 24, marginBottom: 16 }} />
        <div>{t("loading_screen.message")}</div>
      </div>
    );
  }

  // Shows a local (editable) workbook and drops the query string, so a refresh
  // opens this workbook instead of the shared or example one from the URL.
  const showLocalModel = (localModel: Model) => {
    setModel(localModel);
    setIsReadOnly(false);
    setIsSessionOnly(false);
    window.history.replaceState(null, "", window.location.pathname);
  };

  // Opens an imported .xlsx or .ic file as a new local workbook. Files above
  // MAX_PERSISTED_WORKBOOK_BYTES are not stored, see isSessionOnly.
  const openImportedFile = async (file: File) => {
    const newModel = await createModelFromFile(file);
    if (file.size > MAX_PERSISTED_WORKBOOK_BYTES) {
      // Saves and renames go to the selected workbook, so none must be selected
      clearSelectedUuid();
      showLocalModel(newModel);
      setIsSessionOnly(true);
      return;
    }
    await saveModelToStorage(newModel);
    showLocalModel(newModel);
  };

  // Handlers for model changes that also update our models state
  const handleNewModel = async () => {
    showLocalModel(await createNewModel());
  };

  const handleSetModel = async (uuid: string) => {
    const newModel = await selectModelFromStorage(uuid);
    if (newModel) {
      showLocalModel(newModel);
    }
  };

  const handleDeleteModel = async () => {
    const newModel = await deleteSelectedModel();
    if (newModel) {
      showLocalModel(newModel);
    }
  };

  const handleDeleteModelByUuid = async (uuid: string) => {
    const newModel = await deleteModelByUuid(uuid);
    if (newModel) {
      showLocalModel(newModel);
    }
  };

  // Stores the open (shared or example) workbook as a new local workbook.
  const handleSaveCopy = async () => {
    try {
      await saveModelToStorage(model);
      showLocalModel(model);
    } catch (e) {
      reportSaveError(e);
    }
  };

  // We could use context for model, but the problem is that it should initialized to null.
  // Passing the property down makes sure it is always defined.

  return (
    <div className="app-ic-wrapper">
      <LeftDrawer
        open={isDrawerOpen}
        newModel={handleNewModel}
        setModel={handleSetModel}
        onDelete={handleDeleteModelByUuid}
      />
      <div
        className={`app-ic-main-content${isDrawerOpen ? " app-ic-main-content--open" : ""}`}
        style={{
          marginLeft: isDrawerOpen ? 0 : -DRAWER_WIDTH,
          width: isDrawerOpen ? `calc(100% - ${DRAWER_WIDTH}px)` : "100%",
        }}
      >
        <FileBar
          model={model}
          onModelUpload={openImportedFile}
          newModel={handleNewModel}
          newModelFromTemplate={() => {
            setTemplatesDialogOpen(true);
          }}
          setModel={handleSetModel}
          onDelete={handleDeleteModel}
          isReadOnly={isReadOnly}
          isSessionOnly={isSessionOnly}
          isDrawerOpen={isDrawerOpen}
          setIsDrawerOpen={setIsDrawerOpen}
          onLanguageChange={handleLanguageChange}
          themeMode={themeMode}
          onThemeModeChange={handleThemeModeChange}
        />
        {isReadOnly && (
          <NoticeBar
            message={t("unsaved_bar.message")}
            actionLabel={t("unsaved_bar.save_copy")}
            onAction={handleSaveCopy}
          />
        )}
        {isSessionOnly && <NoticeBar message={t("session_only_bar.message")} />}
        <IronCalc
          model={model}
          ref={ironCalcRef}
          themeVariables={isDarkMode ? darkThemeVariables : undefined}
          canEdit={!isReadOnly}
        />
        {isDrawerOpen && (
          <div
            className="app-ic-mobile-overlay"
            onClick={() => setIsDrawerOpen(false)}
            role="none"
          />
        )}
      </div>
      {showWelcomeDialog && (
        <WelcomeDialog
          onClose={async () => {
            if (isStorageEmpty()) {
              const createdModel = await createNewModel();
              setModel(createdModel);
            }
            setShowWelcomeDialog(false);
          }}
          onOpenTemplates={() => {
            setShowWelcomeDialog(false);
            setTemplatesDialogOpen(true);
          }}
          onModelUpload={async (file) => {
            await openImportedFile(file);
            setShowWelcomeDialog(false);
          }}
          onSelectTemplate={async (templateId) => {
            switch (templateId) {
              case "blank": {
                const createdModel = await createNewModel();
                setModel(createdModel);
                break;
              }
              default: {
                const model_bytes = await get_documentation_model(templateId);
                const locale = loadDefaultLocaleFromStorage();
                const languageId = getLanguageFromLocale(locale);
                const importedModel = Model.fromBytes(model_bytes, languageId);
                importedModel.setName(
                  t(`welcome_dialog.templates.${templateId}`),
                );
                await saveModelToStorage(importedModel);
                setModel(importedModel);
                break;
              }
            }
            setShowWelcomeDialog(false);
          }}
        />
      )}
      <TemplatesDialog
        open={isTemplatesDialogOpen}
        onClose={() => setTemplatesDialogOpen(false)}
        onSelectTemplate={async (fileName) => {
          const model_bytes = await get_documentation_model(fileName);
          const locale = loadDefaultLocaleFromStorage();
          const languageId = getLanguageFromLocale(locale);
          const importedModel = Model.fromBytes(model_bytes, languageId);
          importedModel.setName(t(`welcome_dialog.templates.${fileName}`));
          await saveModelToStorage(importedModel);
          showLocalModel(importedModel);
          setTemplatesDialogOpen(false);
        }}
      />
    </div>
  );
}

const DRAWER_WIDTH = 264;
// Imported files larger than this are not stored in the browser (session only).
// A model of that size plus its serialized bytes gets close to the 4GB wasm32
// memory limit.
const MAX_PERSISTED_WORKBOOK_BYTES = 200 * 1024 * 1024;
export const MIN_MAIN_CONTENT_WIDTH_FOR_MOBILE = 768;

export default App;
