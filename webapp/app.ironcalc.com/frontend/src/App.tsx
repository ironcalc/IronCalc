import "./App.css";
import styled from "@emotion/styled";
import type { IronCalcHandle } from "@ironcalc/workbook";
// From IronCalc
import { IronCalc, IronCalcIcon, init, Model } from "@ironcalc/workbook";
import { Modal } from "@mui/material";
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

  useEffect(() => {
    async function start() {
      await init();
      const queryString = window.location.search;
      const urlParams = new URLSearchParams(queryString);
      const modelHash = urlParams.get("model");
      const exampleFilename = urlParams.get("example");
      const language = loadDefaultLocaleFromStorage();
      const localeShort = getShortLocaleCode(language);
      // If there is a model name ?model=modelHash we try to load it
      // if there is not, or the loading failed we load an empty model
      if (modelHash) {
        // Get a remote model
        try {
          const model_bytes = await get_model(modelHash);
          const importedModel = Model.from_bytes(model_bytes, localeShort);
          localStorage.removeItem("selected");
          setModel(importedModel);
        } catch (_e) {
          console.log("Failed to load model from hash:", modelHash);
        }
      } else if (exampleFilename) {
        try {
          const model_bytes = await get_documentation_model(exampleFilename);
          const importedModel = Model.from_bytes(model_bytes, localeShort);
          localStorage.removeItem("selected");
          setModel(importedModel);
        } catch (_e) {
          console.log("Failed to load example model:", exampleFilename);
        }
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

  if (!model) {
    return (
      <Loading>
        <IronCalcIcon style={{ width: 24, height: 24, marginBottom: 16 }} />
        <div>{t("loading_screen.message")}</div>
      </Loading>
    );
  }

  // We try to save the model every second
  setInterval(() => {
    const queue = model.flushSendQueue();
    if (queue.length !== 1) {
      saveSelectedModelInStorage(model);
    }
  }, 1000);

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
    <Wrapper>
      <LeftDrawer
        open={isDrawerOpen}
        onClose={() => setIsDrawerOpen(false)}
        newModel={handleNewModel}
        setModel={handleSetModel}
        onDelete={handleDeleteModelByUuid}
        localStorageId={localStorageId}
      />
      <MainContent isDrawerOpen={isDrawerOpen}>
        {isDrawerOpen && (
          <MobileOverlay onClick={() => setIsDrawerOpen(false)} />
        )}
        <FileBar
          model={model}
          onModelUpload={async (arrayBuffer: ArrayBuffer, fileName: string) => {
            const blob = await uploadFile(arrayBuffer, fileName);

            const bytes = new Uint8Array(await blob.arrayBuffer());
            const locale = loadDefaultLocaleFromStorage();
            const localeShort = getShortLocaleCode(locale);
            const newModel = Model.from_bytes(bytes, localeShort);
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
        <IronCalc model={model} ref={ironCalcRef} />
      </MainContent>
      {showWelcomeDialog && (
        <WelcomeDialog
          onClose={() => {
            if (isStorageEmpty()) {
              const createdModel = createNewModel();
              setModel(createdModel);
            }
            setShowWelcomeDialog(false);
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
                const localeShort = getShortLocaleCode(locale);
                const importedModel = Model.from_bytes(
                  model_bytes,
                  localeShort,
                );
                saveModelToStorage(importedModel);
                setModel(importedModel);
                break;
              }
            }
            setShowWelcomeDialog(false);
          }}
        />
      )}
      <Modal
        open={isTemplatesDialogOpen}
        onClose={() => setTemplatesDialogOpen(false)}
        aria-labelledby="templates-dialog-title"
        aria-describedby="templates-dialog-description"
      >
        <TemplatesDialog
          onClose={() => setTemplatesDialogOpen(false)}
          onSelectTemplate={async (fileName) => {
            const model_bytes = await get_documentation_model(fileName);
            const locale = loadDefaultLocaleFromStorage();
            const localeShort = getShortLocaleCode(locale);
            const importedModel = Model.from_bytes(model_bytes, localeShort);
            saveModelToStorage(importedModel);
            setModel(importedModel);
            setTemplatesDialogOpen(false);
          }}
        />
      </Modal>
    </Wrapper>
  );
}

const Wrapper = styled("div")`
  display: flex;
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
`;

const DRAWER_WIDTH = 264;
export const MIN_MAIN_CONTENT_WIDTH_FOR_MOBILE = 768;

const MainContent = styled("div")<{ isDrawerOpen: boolean }>`
  margin-left: ${({ isDrawerOpen }) =>
    isDrawerOpen ? "0px" : `-${DRAWER_WIDTH}px`};
  width: ${({ isDrawerOpen }) =>
    isDrawerOpen ? `calc(100% - ${DRAWER_WIDTH}px)` : "100%"};
  display: flex;
  flex-direction: column;
  position: relative;
  @media (max-width: ${MIN_MAIN_CONTENT_WIDTH_FOR_MOBILE}px) {
    ${({ isDrawerOpen }) =>
      isDrawerOpen && `min-width: ${MIN_MAIN_CONTENT_WIDTH_FOR_MOBILE}px;`}
  }
`;

const MobileOverlay = styled("div")`
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(255, 255, 255, 0.8);
  z-index: 100;
  cursor: pointer;

  @media (min-width: ${MIN_MAIN_CONTENT_WIDTH_FOR_MOBILE + 1}px) {
    display: none;
  }
`;

const Loading = styled("div")`
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-family: "Inter";
  font-size: 14px;
`;

export default App;
