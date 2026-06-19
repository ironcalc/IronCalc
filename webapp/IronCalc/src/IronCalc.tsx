import type { Model } from "@ironcalc/wasm";
import { forwardRef, useEffect, useImperativeHandle, useRef } from "react";
import { I18nextProvider } from "react-i18next";
import Workbook from "./components/Workbook/Workbook.tsx";
import { WorkbookState } from "./components/workbookState.ts";
import i18n from "./i18n";
import "./theme/theme.css";
import "./index.css";
import { type PartialIronCalcThemeVariables, setThemeVariables } from "./theme";

interface IronCalcProperties {
  model: Model;
  themeVariables?: PartialIronCalcThemeVariables;
  // If we apply a mutation to the model from outside React 
  // (e.g. applying a remote diff), we want to update the
  // canvas without throwing away the editing state. This can
  // be done by incrementing the externalRevision prop.
  externalRevision?: number;
}

export interface IronCalcHandle {
  setLanguage: (language: string) => void;
}

const IronCalc = forwardRef<IronCalcHandle, IronCalcProperties>(
  ({ themeVariables, model, externalRevision = 0 }, ref) => {
    const rootRef = useRef<HTMLDivElement>(null);

    // We keep the WorkbookState as a ref so that it survives re-renders
    // of this component.
    // But we do reset it when model identity is explicitly changed.
    const workbookState = useRef<WorkbookState | null>(null);
    const lastModel = useRef<Model | null>(null);
    if (workbookState.current === null || lastModel.current !== model) {
      workbookState.current = new WorkbookState();
      lastModel.current = model;
    }

    useEffect(() => {
      if (rootRef.current && themeVariables) {
        setThemeVariables(themeVariables, rootRef.current);
      }
    }, [themeVariables]);

    useImperativeHandle(ref, () => ({
      setLanguage(language: string) {
        if (i18n.language !== language) {
          i18n.changeLanguage(language);
          const lang = language.split("-")[0];
          model.setLanguage(lang);
        }
      },
    }));

    return (
      <div ref={rootRef} className="ic-root">
        <I18nextProvider i18n={i18n}>
          <Workbook
            model={model}
            workbookState={workbookState.current}
            externalRevision={externalRevision}
          />
        </I18nextProvider>
      </div>
    );
  },
);

IronCalc.displayName = "IronCalc";

export default IronCalc;
