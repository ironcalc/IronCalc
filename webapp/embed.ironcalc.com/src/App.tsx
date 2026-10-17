import type { IronCalcHandle } from "@ironcalc/workbook";
import { darkThemeVariables, IronCalc, init, Model } from "@ironcalc/workbook";
import "@ironcalc/workbook/style.css";
import { useEffect, useRef, useState } from "react";

type IronCalcTheme = "light" | "dark" | "system";

type IronCalcMessage =
  | { type: "ironcalc:init:v1" }
  | { type: "ironcalc:ready:v1" }
  | { type: "ironcalc:load-workbook:v1"; workbookBytes: ArrayBuffer }
  | { type: "ironcalc:load-empty:v1" }
  | { type: "ironcalc:set-theme:v1"; theme: IronCalcTheme };

const DARK_SCHEME_QUERY = "(prefers-color-scheme: dark)";

function parseTheme(value: unknown): IronCalcTheme {
  return value === "dark" || value === "system" ? value : "light";
}

/** The host passes the initial theme in the URL so the loading screen
 * already has the right colours before the postMessage handshake completes. */
function initialTheme(): IronCalcTheme {
  return parseTheme(new URLSearchParams(window.location.search).get("theme"));
}

function systemPrefersDark(): boolean {
  return window.matchMedia?.(DARK_SCHEME_QUERY).matches ?? false;
}

function App() {
  const ironCalcRef = useRef<IronCalcHandle>(null);
  const [model, setModel] = useState<Model | null>(null);
  const [theme, setTheme] = useState<IronCalcTheme>(initialTheme);
  const [systemIsDark, setSystemIsDark] = useState(systemPrefersDark);
  const isDarkMode = theme === "system" ? systemIsDark : theme === "dark";

  useEffect(() => {
    const query = window.matchMedia?.(DARK_SCHEME_QUERY);
    if (!query) {
      return;
    }
    const listener = (event: MediaQueryListEvent) =>
      setSystemIsDark(event.matches);
    query.addEventListener("change", listener);
    return () => query.removeEventListener("change", listener);
  }, []);

  // Drives `color-scheme` and the page background (see index.css), which
  // matters before the workbook is mounted and for native scrollbars.
  useEffect(() => {
    document.documentElement.dataset.theme = isDarkMode ? "dark" : "light";
  }, [isDarkMode]);

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
        setModel(Model.fromBytes(new Uint8Array(data.workbookBytes), "en"));
      }
      if (data.type === "ironcalc:load-empty:v1") {
        setModel(new Model("Workbook", "en", "UTC", "en"));
      }
      if (data.type === "ironcalc:set-theme:v1") {
        setTheme(parseTheme(data.theme));
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

  return (
    <IronCalc
      model={model}
      ref={ironCalcRef}
      themeVariables={isDarkMode ? darkThemeVariables : undefined}
    />
  );
}

export default App;
