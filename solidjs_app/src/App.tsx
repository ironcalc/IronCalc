import { Show, createResource } from "solid-js";
// import "./App.css";
// import solidLogo from "./assets/solid.svg";

import init, { Model } from "@ironcalc/wasm";
import Workbook from "./components/Workbook";

const fetchModel = async () => {
  await init();
  // const model_bytes = new Uint8Array(
  //     await (await fetch("./example.ic")).arrayBuffer(),
  //   );
  //   const _model = Model.from_bytes(model_bytes);*/
  const model = new Model("en", "UTC");
  model.setUserInput(0, 1, 1, "=1+1");
  return model;
};

function App() {
  const [model] = createResource(fetchModel);

  return (
    <Show when={model()} fallback={<div>Loading...</div>}>
      {(model) => <Workbook model={model()} />}
    </Show>
  );
}

export default App;
