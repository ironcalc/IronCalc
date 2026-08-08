import type { IronCalcHandle } from "@ironcalc/workbook";
import { IronCalc, init, Model } from "@ironcalc/workbook";
import "@ironcalc/workbook/style.css";
import { useEffect, useRef, useState } from "react";

type IronCalcMessage =
  | { type: "ironcalc:init:v1" }
  | { type: "ironcalc:ready:v1" }
  | { type: "ironcalc:load-workbook:v1"; workbookBytes: ArrayBuffer }
  | { type: "ironcalc:load-empty:v1" };

function App() {
  const ironCalcRef = useRef<IronCalcHandle>(null);
  const [model, setModel] = useState<Model | null>(null);

  useEffect(() => {
    let active = true;
    let parentOrigin: string | null = null;
    let wasmReady = false;
    let pendingInit = false;

    function onMessage(event: MessageEvent<IronCalcMessage>) {
      if (event.source !== window.parent) {
        return;
      }
      if (parentOrigin && event.origin !== parentOrigin) {
        return;
      }

      const data = event.data;

      if (data.type === "ironcalc:init:v1") {
        parentOrigin = event.origin;
        if (wasmReady) {
          window.parent.postMessage(
            { type: "ironcalc:ready:v1" },
            parentOrigin,
          );
        } else {
          // WASM still loading — defer the reply until init() completes
          pendingInit = true;
        }
        return;
      }

      if (parentOrigin && event.origin !== parentOrigin) {
        return;
      }

      if (data.type === "ironcalc:load-workbook:v1") {
        setModel(Model.from_bytes(new Uint8Array(data.workbookBytes), "en"));
      }
      if (data.type === "ironcalc:load-empty:v1") {
        setModel(new Model("Workbook", "en", "UTC", "en"));
      }
    }

    async function start() {
      // Register before awaiting WASM so we never miss ironcalc:init:v1
      window.addEventListener("message", onMessage);
      await init();
      if (!active) {
        return;
      }
      wasmReady = true;
      if (pendingInit && parentOrigin) {
        window.parent.postMessage({ type: "ironcalc:ready:v1" }, parentOrigin);
      }
    }

    start();

    return () => {
      active = false;
      window.removeEventListener("message", onMessage);
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
