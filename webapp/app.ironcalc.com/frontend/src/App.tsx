import "./App.css";
import type { IronCalcHandle } from "@ironcalc/workbook";
import { IronCalc, IronCalcIcon, init, Model } from "@ironcalc/workbook";
import "@ironcalc/workbook/style.css";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { FileBar } from "./components/FileBar";
import LeftDrawer from "./components/LeftDrawer/LeftDrawer";
import {
  get_documentation_model,
  get_model,
  uploadFile,
} from "./components/rpc";
import {
  createModelWithSafeTimezone,
  createNewModel,
  deleteModelByUuid,
  deleteSelectedModel,
  getLanguageFromLocale,
  getShortLocaleCode,
  isStorageEmpty,
  loadDefaultLocaleFromStorage,
  loadSelectedModelFromStorage,
  saveDefaultLocaleInStorage,
  saveModelToStorage,
  saveSelectedModelInStorage,
  selectModelFromStorage,
} from "./components/storage";
import TemplatesDialog from "./components/WelcomeDialog/TemplatesDialog";
import WelcomeDialog from "./components/WelcomeDialog/WelcomeDialog";

export const MIN_MAIN_CONTENT_WIDTH_FOR_MOBILE = 600;

function App() {
  const [model, setModel] = useState<Model | null>(null);
  const [showWelcomeDialog, setShowWelcomeDialog] = useState(false);
  const [isTemplatesDialogOpen, setTemplatesDialogOpen] = useState(false);
  const [isDrawerOpen, setIsDrawerOpen] = useState(false);
  const [localStorageId, setLocalStorageId] = useState<number>(1);

  const ironCalcRef = useRef<IronCalcHandle>(null);

  const handleLanguageChange = (language: string) => {
    if (ironCalcRef.current) {
      ironCalcRef.current.setLanguage(language);
      saveDefaultLocaleInStorage(language);
      if (model) {
        model.setLocale(getShortLocaleCode(language));
        saveSelectedModelInStorage(model);
      }
    }
  };

  const { t, i18n } = useTranslation();

  // biome-ignore lint/correctness/useExhaustiveDependencies: Run only for i18n.language dependency
  useEffect(() => {
    async function start() {
      await init();
      const queryString = window.location.search;
      const urlParams = new URLSearchParams(queryString);
      const modelHash = urlParams.get("model");
      const exampleFilename = urlParams.get("example");
      const language = loadDefaultLocaleFromStorage();
      const languageId = getLanguageFromLocale(language);
      let loadedModel: Model | null = null;
      if (modelHash) {
        try {
          const model_bytes = await get_model(modelHash);
          loadedModel = Model.from_bytes(model_bytes, languageId);
          localStorage.removeItem("selected");
        } catch (_e) {
          console.error(_e);
          alert(t("errors.model_not_found"));
        }
      } else if (exampleFilename) {
        try {
          const model_bytes = await get_documentation_model(exampleFilename);
          loadedModel = Model.from_bytes(model_bytes, languageId);
          localStorage.removeItem("selected");
        } catch (_e) {
          console.error(_e);
          alert(t("errors.example_not_found"));
        }
      }

      if (loadedModel) {
        setModel(loadedModel);
      } else {
        const result = loadSelectedModelFromStorage();
        if (!result) {
          setShowWelcomeDialog(true);
          const createdModel = createModelWithSafeTimezone("template");
          setModel(createdModel);
        } else {
          setModel(result);
        }
      }
      i18n.changeLanguage(language);
      setTimeout(() => {
        ironCalcRef.current?.setLanguage(language);
      }, 0);
    }
    start();
  }, [i18n.changeLanguage]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: localStorageId needed to detect name changes (model mutates internally)
  useEffect(() => {
    if (model) {
      const workbookName = model.getName();
      document.title = workbookName ? `${workbookName} - IronCalc` : "IronCalc";
    } else {
      document.title = "IronCalc";
    }
  }, [model, localStorageId]);

  useEffect(() => {
    if (!model) return;
    const interval = setInterval(() => {
      const queue = model.flushSendQueue();
      if (queue.length !== 1) {
        saveSelectedModelInStorage(model);
      }
    }, 1000);
    return () => clearInterval(interval);
  }, [model]);

  if (!model) {
    return (
      <div className="app-loading">
        <IronCalcIcon style={{ width: 24, height: 24, marginBottom: 16 }} />
        <div>{t("loading_screen.message")}</div>
      </div>
    );
  }

  const handleNewModel = () => setModel(createNewModel());

  const handleSetModel = (uuid: string) => {
    const newModel = selectModelFromStorage(uuid);
    if (newModel) setModel(newModel);
  };

  const handleDeleteModel = () => {
    const newModel = deleteSelectedModel();
    if (newModel) setModel(newModel);
  };

  const handleDeleteModelByUuid = (uuid: string) => {
    const newModel = deleteModelByUuid(uuid);
    if (newModel) setModel(newModel);
  };

  return (
    <div className="app-wrapper">
      <LeftDrawer
        open={isDrawerOpen}
        onClose={() => setIsDrawerOpen(false)}
        newModel={handleNewModel}
        setModel={handleSetModel}
        onDelete={handleDeleteModelByUuid}
        localStorageId={localStorageId}
      />
      <div
        className={`app-main-content${isDrawerOpen ? " app-main-content--drawer-open" : ""}`}
      >
        {isDrawerOpen && (
          <button
            type="button"
            className="app-mobile-overlay"
            aria-label="Close sidebar"
            onClick={() => setIsDrawerOpen(false)}
          />
        )}
        <FileBar
          model={model}
          onModelUpload={async (arrayBuffer: ArrayBuffer, fileName: string) => {
            const blob = await uploadFile(arrayBuffer, fileName);
            const bytes = new Uint8Array(await blob.arrayBuffer());
            const locale = loadDefaultLocaleFromStorage();
            const languageId = getLanguageFromLocale(locale);
            const newModel = Model.from_bytes(bytes, languageId);
            saveModelToStorage(newModel);
            setModel(newModel);
          }}
          newModel={handleNewModel}
          newModelFromTemplate={() => setTemplatesDialogOpen(true)}
          setModel={handleSetModel}
          onDelete={handleDeleteModel}
          isDrawerOpen={isDrawerOpen}
          setIsDrawerOpen={setIsDrawerOpen}
          setLocalStorageId={setLocalStorageId}
          onLanguageChange={handleLanguageChange}
        />
        <IronCalc model={model} ref={ironCalcRef} />
      </div>
      {showWelcomeDialog && (
        <WelcomeDialog
          onClose={() => {
            if (isStorageEmpty()) setModel(createNewModel());
            setShowWelcomeDialog(false);
          }}
          onSelectTemplate={async (templateId) => {
            if (templateId === "blank") {
              setModel(createNewModel());
            } else {
              const model_bytes = await get_documentation_model(templateId);
              const locale = loadDefaultLocaleFromStorage();
              const languageId = getLanguageFromLocale(locale);
              const importedModel = Model.from_bytes(model_bytes, languageId);
              saveModelToStorage(importedModel);
              setModel(importedModel);
            }
            setShowWelcomeDialog(false);
          }}
        />
      )}
      {isTemplatesDialogOpen && (
        <TemplatesDialog
          onClose={() => setTemplatesDialogOpen(false)}
          onSelectTemplate={async (fileName) => {
            const model_bytes = await get_documentation_model(fileName);
            const locale = loadDefaultLocaleFromStorage();
            const languageId = getLanguageFromLocale(locale);
            const importedModel = Model.from_bytes(model_bytes, languageId);
            saveModelToStorage(importedModel);
            setModel(importedModel);
            setTemplatesDialogOpen(false);
          }}
        />
      )}
    </div>
  );
}

export default App;
