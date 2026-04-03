import type { IronCalcHandle } from "@ironcalc/workbook";
import { IronCalc, init, Model } from "@ironcalc/workbook";
import "@ironcalc/workbook/style.css";
import { useEffect, useRef, useState } from "react";

type LoadWorkbookMessage = {
  type: "ironcalc-load-workbook";
  workbookBytes: Uint8Array | ArrayBuffer;
};

type LoadEmptyWorkbookMessage = {
  type: "ironcalc-load-empty-workbook";
};

type IronCalcMessage = LoadWorkbookMessage | LoadEmptyWorkbookMessage;

function toUint8Array(bytes: Uint8Array | ArrayBuffer): Uint8Array {
  return bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
}

function App() {
  const ironCalcRef = useRef<IronCalcHandle>(null);
  const [model, setModel] = useState<Model | null>(null);

  useEffect(() => {
    let isCancelled = false;

    async function start() {
      await init();

      if (isCancelled) {
        return;
      }

      function onMessage(event: MessageEvent<IronCalcMessage>) {
        const data = event.data;
        if (!data || typeof data !== "object" || !("type" in data)) {
          return;
        }

        if (data.type === "ironcalc-load-workbook") {
          const workbookBytes = toUint8Array(data.workbookBytes);
          const loadedModel = Model.from_bytes(workbookBytes, "en");
          setModel(loadedModel);
          window.removeEventListener("message", onMessage);
          return;
        }

        if (data.type === "ironcalc-load-empty-workbook") {
          const emptyModel = new Model("Workbook", "en", "UTC", "en");
          setModel(emptyModel);
          window.removeEventListener("message", onMessage);
        }
      }

      window.addEventListener("message", onMessage);

      window.parent.postMessage(
        {
          type: "ironcalc-ready",
        },
        "*",
      );
    }

    start();

    return () => {
      isCancelled = true;
    };
  }, []);

  if (!model) {
    return (
      <div className="App">
        <h1>Loading...</h1>
      </div>
    );
  }

  return <IronCalc model={model} ref={ironCalcRef} />;
}

export default App;
