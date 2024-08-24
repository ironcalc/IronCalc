import "./App.css";
import Workbook from "./components/workbook";
import "./i18n";
import styled from "@emotion/styled";
import init, { Model } from "@ironcalc/wasm";
import { useEffect, useState } from "react";
import { WorkbookState } from "./components/workbookState";

function App() {
  const [model, setModel] = useState<Model | null>(null);
  const [workbookState, setWorkbookState] = useState<WorkbookState | null>(
    null,
  );
  useEffect(() => {
    async function start() {
      await init();
      const queryString = window.location.search;
      const urlParams = new URLSearchParams(queryString);
      const modelName = urlParams.get("model");
      // If there is a model name ?model=example.ic we try to load it
      // if there is not, or the loading failed we load an empty model
      if (modelName) {
        try {
          const model_bytes = new Uint8Array(
            await (await fetch(`./${modelName}`)).arrayBuffer(),
          );
          setModel(Model.from_bytes(model_bytes));
        } catch (e) {
          setModel(new Model("en", "UTC"));
        }
      } else {
        setModel(new Model("en", "UTC"));
      }
      setWorkbookState(new WorkbookState());
    }
    start();
  }, []);

  if (!model || !workbookState) {
    return <Loading>Loading</Loading>;
  }

  // We could use context for model, but the problem is that it should initialized to null.
  // Passing the property down makes sure it is always defined.
  return <Workbook model={model} workbookState={workbookState} />;
}

const Loading = styled("div")`
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 36px;
`;

export default App;
