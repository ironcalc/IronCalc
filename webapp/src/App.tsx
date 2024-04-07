import "./App.css";
import Workbook from "./components/workbook";
import "./i18n";
import { useEffect, useState } from "react";
import init, { Model } from "@ironcalc/wasm";
import { WorkbookState } from "./components/workbookState";

function App() {
  const [model, setModel] = useState<Model | null>(null);
  const [workbookState, setWorkbookState] = useState<WorkbookState | null>(
    null
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
            await (await fetch(`./${modelName}`)).arrayBuffer()
          );
          const _model = Model.from_bytes(model_bytes);
          if (!model) setModel(_model);
        } catch (e) {
          const _model = new Model("en", "UTC");
          if (!model) setModel(_model);
        }
      } else {
        const _model = new Model("en", "UTC");
        if (!model) setModel(_model);
      }
      if (!workbookState) setWorkbookState(new WorkbookState());
    }
    start();
  }, []);

  if (!model || !workbookState) {
    return <div>Loading</div>;
  }

  // We could use context for model, but the problem is that it should initialized to null.
  // Passing the property down makes sure it is always defined.
  return <Workbook model={model} workbookState={workbookState} />;
}

export default App;
