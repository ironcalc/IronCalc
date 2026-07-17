import "./App.css";
import type { IronCalcHandle } from "@ironcalc/workbook";
// From IronCalc
import {
  CollabProvider,
  IronCalc,
  IronCalcIcon,
  init,
  Model,
} from "@ironcalc/workbook";
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

// The collaboration relay server (webapp/../collab-server); the room name
// comes from the `?room=` URL parameter.
function collabServerUrl(): string {
  return (
    import.meta.env.VITE_COLLAB_SERVER_URL ??
    `ws://${window.location.hostname}:9000`
  );
}

function App() {
  const [model, setModel] = useState<Model | null>(null);
  const [collabProvider, setCollabProvider] = useState<CollabProvider | null>(
    null,
  );
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
      const collabRoom = urlParams.get("room");
      const language = loadDefaultLocaleFromStorage();
      const languageId = getLanguageFromLocale(language);

      if (collabRoom) {
        // Collaborative session: the room document is authoritative, so we
        // start from a blank workbook and let the sync handshake fill it in.
        const collabModel = createModelWithSafeTimezone(collabRoom);
        const provider = new CollabProvider(
          collabModel,
          `${collabServerUrl()}/${encodeURIComponent(collabRoom)}`,
        );
        provider.connect();
        setModel(collabModel);
        setCollabProvider(provider);
        i18n.changeLanguage(language);
        setTimeout(() => {
          ironCalcRef.current?.setLanguage(language);
        }, 0);
        return;
      }
      // If there is a model name ?model=modelHash we try to load it
      // if there is not, or the loading failed we load an empty model
      let loadedModel: Model | null = null;
      if (modelHash) {
        // Get a remote model
        try {
          const model_bytes = await get_model(modelHash);
          loadedModel = Model.from_bytes(model_bytes, languageId);
          localStorage.removeItem("selected");
        } catch (_e) {
          console.error(_e);
          alert(t("errors.model_not_found"));
          console.log("Failed to load model from hash:", modelHash);
        }
      } else if (exampleFilename) {
        try {
          const model_bytes = await get_documentation_model(exampleFilename);
          loadedModel = Model.from_bytes(model_bytes, languageId);
          localStorage.removeItem("selected");
        } catch (_e) {
          console.error(_e);
          alert(t("errors.example_not_found"));
          console.log("Failed to load example model:", exampleFilename);
        }
      }

      if (loadedModel) {
        setModel(loadedModel);
      } else {
        // try to load from local storage
        const result = loadSelectedModelFromStorage();
        if (!result) {
          setShowWelcomeDialog(true);
          const createdModel = createModelWithSafeTimezone("template");
          setModel(createdModel);
        } else {
          const newModel = result;
          setModel(newModel);
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
    // Collaborative models live on the relay server, not in local storage.
    if (!model || collabProvider) return;
    // We try to save the model every second
    const interval = setInterval(() => {
      const queue = model.flushSendQueue();
      if (queue.length !== 1) {
        saveSelectedModelInStorage(model);
      }
    }, 1000);
    return () => clearInterval(interval);
  }, [model, collabProvider]);

  useEffect(() => {
    if (!collabProvider) return;
    // Withdraw our presence when the tab goes away.
    const goodbye = () => collabProvider.destroy();
    window.addEventListener("beforeunload", goodbye);
    return () => {
      window.removeEventListener("beforeunload", goodbye);
      collabProvider.destroy();
    };
  }, [collabProvider]);

  if (!model) {
    return (
      <div className="app-ic-loading">
        <IronCalcIcon style={{ width: 24, height: 24, marginBottom: 16 }} />
        <div>{t("loading_screen.message")}</div>
      </div>
    );
  }

  // Handlers for model changes that also update our models state
  const handleNewModel = () => {
    const newModel = createNewModel();
    setModel(newModel);
  };

  const handleSetModel = (uuid: string) => {
    const newModel = selectModelFromStorage(uuid);
    if (newModel) {
      setModel(newModel);
    }
  };

  const handleDeleteModel = () => {
    const newModel = deleteSelectedModel();
    if (newModel) {
      setModel(newModel);
    }
  };

  const handleDeleteModelByUuid = (uuid: string) => {
    const newModel = deleteModelByUuid(uuid);
    if (newModel) {
      setModel(newModel);
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
        localStorageId={localStorageId}
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
          newModelFromTemplate={() => {
            setTemplatesDialogOpen(true);
          }}
          setModel={handleSetModel}
          onDelete={handleDeleteModel}
          isDrawerOpen={isDrawerOpen}
          setIsDrawerOpen={setIsDrawerOpen}
          setLocalStorageId={setLocalStorageId}
          onLanguageChange={handleLanguageChange}
        />
        <IronCalc
          model={model}
          ref={ironCalcRef}
          collabProvider={collabProvider ?? undefined}
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
          onClose={() => {
            if (isStorageEmpty()) {
              const createdModel = createNewModel();
              setModel(createdModel);
            }
            setShowWelcomeDialog(false);
          }}
          onOpenTemplates={() => {
            setShowWelcomeDialog(false);
            setTemplatesDialogOpen(true);
          }}
          onModelUpload={async (arrayBuffer: ArrayBuffer, fileName: string) => {
            const blob = await uploadFile(arrayBuffer, fileName);
            const bytes = new Uint8Array(await blob.arrayBuffer());
            const locale = loadDefaultLocaleFromStorage();
            const languageId = getLanguageFromLocale(locale);
            const newModel = Model.from_bytes(bytes, languageId);
            saveModelToStorage(newModel);
            setModel(newModel);
          }}
          onSelectTemplate={async (templateId) => {
            switch (templateId) {
              case "blank": {
                const createdModel = createNewModel();
                setModel(createdModel);
                break;
              }
              default: {
                const model_bytes = await get_documentation_model(templateId);
                const locale = loadDefaultLocaleFromStorage();
                const languageId = getLanguageFromLocale(locale);
                const importedModel = Model.from_bytes(model_bytes, languageId);
                saveModelToStorage(importedModel);
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
          const importedModel = Model.from_bytes(model_bytes, languageId);
          saveModelToStorage(importedModel);
          setModel(importedModel);
          setTemplatesDialogOpen(false);
        }}
      />
    </div>
  );
}

const DRAWER_WIDTH = 264;
export const MIN_MAIN_CONTENT_WIDTH_FOR_MOBILE = 768;

export default App;
