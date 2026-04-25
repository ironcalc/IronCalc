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
    const controller = new AbortController();
    const { signal } = controller;

    // Derive the parent's origin from document.referrer so we can restrict
    // postMessage targets and validate incoming message origins. Falls back to
    // null when the referrer is absent (e.g. referrerpolicy="no-referrer").
    const parentOrigin = document.referrer
      ? new URL(document.referrer).origin
      : null;

    async function start() {
      await init();

      if (signal.aborted) {
        return;
      }

      function onMessage(event: MessageEvent<IronCalcMessage>) {
        // Only accept messages sent by the direct parent frame.
        if (event.source !== window.parent) {
          return;
        }
        // When we know the parent origin, reject messages from anywhere else.
        if (parentOrigin && event.origin !== parentOrigin) {
          return;
        }

        const data = event.data;
        if (!data || typeof data !== "object" || !("type" in data)) {
          return;
        }

        if (data.type === "ironcalc-load-workbook") {
          const workbookBytes = toUint8Array(data.workbookBytes);
          const loadedModel = Model.from_bytes(workbookBytes, "en");
          setModel(loadedModel);
          controller.abort();
          return;
        }

        if (data.type === "ironcalc-load-empty-workbook") {
          const emptyModel = new Model("Workbook", "en", "UTC", "en");
          setModel(emptyModel);
          controller.abort();
        }
      }

      // { signal } means the listener is removed automatically on abort,
      // covering both successful load (above) and effect cleanup (below).
      window.addEventListener("message", onMessage, { signal });

      window.parent.postMessage(
        {
          type: "ironcalc-ready",
        },
        parentOrigin ?? "*",
      );
    }

    start();

    return () => {
      controller.abort();
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
