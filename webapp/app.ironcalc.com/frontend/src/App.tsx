import "./App.css";
import styled from "@emotion/styled";
import { useEffect, useState } from "react";
import { FileBar } from "./components/FileBar";
import LeftDrawer from "./components/LeftDrawer/LeftDrawer";
import WelcomeDialog from "./components/WelcomeDialog/WelcomeDialog";
import {
  get_documentation_model,
  get_model,
  uploadFile,
} from "./components/rpc";
import {
  createNewModel,
  deleteModelByUuid,
  deleteSelectedModel,
  isStorageEmpty,
  loadSelectedModelFromStorage,
  saveModelToStorage,
  saveSelectedModelInStorage,
  selectModelFromStorage,
} from "./components/storage";

// From IronCalc
import { IronCalc, IronCalcIcon, Model, init } from "@ironcalc/workbook";
import { Modal } from "@mui/material";
import TemplatesDialog from "./components/WelcomeDialog/TemplatesDialog";

function App() {
  const [model, setModel] = useState<Model | null>(null);
  const [showWelcomeDialog, setShowWelcomeDialog] = useState(false);
  const [isTemplatesDialogOpen, setTemplatesDialogOpen] = useState(false);
  const [isDrawerOpen, setIsDrawerOpen] = useState(false);
  const [localStorageId, setLocalStorageId] = useState<number>(1);

  useEffect(() => {
    async function start() {
      await init();
      const queryString = window.location.search;
      const urlParams = new URLSearchParams(queryString);
      const modelHash = urlParams.get("model");
      const exampleFilename = urlParams.get("example");
      // If there is a model name ?model=modelHash we try to load it
      // if there is not, or the loading failed we load an empty model
      if (modelHash) {
        // Get a remote model
        try {
          const model_bytes = await get_model(modelHash);
          const importedModel = Model.from_bytes(model_bytes);
          localStorage.removeItem("selected");
          setModel(importedModel);
        } catch (e) {
          alert("Model not found, or failed to load");
        }
      } else if (exampleFilename) {
        try {
          const model_bytes = await get_documentation_model(exampleFilename);
          const importedModel = Model.from_bytes(model_bytes);
          localStorage.removeItem("selected");
          setModel(importedModel);
        } catch (e) {
          alert("Example file not found, or failed to load");
        }
      } else {
        // try to load from local storage
        const newModel = loadSelectedModelFromStorage();
        if (!newModel) {
          setShowWelcomeDialog(true);
          const createdModel = new Model("template", "en", "UTC");
          setModel(createdModel);
        } else {
          setModel(newModel);
        }
      }
    }
    start();
  }, []);

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
        <div>Loading IronCalc</div>
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
            const newModel = Model.from_bytes(bytes);
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
        />
        <IronCalc model={model} />
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
                const importedModel = Model.from_bytes(model_bytes);
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
            const importedModel = Model.from_bytes(model_bytes);
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
const MIN_MAIN_CONTENT_WIDTH_FOR_MOBILE = 440;

const MainContent = styled("div")<{ isDrawerOpen: boolean }>`
  margin-left: ${({ isDrawerOpen }) => (isDrawerOpen ? "0px" : `-${DRAWER_WIDTH}px`)};
  width: ${({ isDrawerOpen }) => (isDrawerOpen ? `calc(100% - ${DRAWER_WIDTH}px)` : "100%")};
  display: flex;
  flex-direction: column;
  position: relative;
  @media (max-width: ${MIN_MAIN_CONTENT_WIDTH_FOR_MOBILE}px) {
    ${({ isDrawerOpen }) => isDrawerOpen && `min-width: ${MIN_MAIN_CONTENT_WIDTH_FOR_MOBILE}px;`}
  }
`;

const MobileOverlay = styled("div")`
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(255, 255, 255, 0.8);
  z-index: 1;
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
