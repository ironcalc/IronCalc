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
}

export interface IronCalcHandle {
  setLanguage: (language: string) => void;
}

const IronCalc = forwardRef<IronCalcHandle, IronCalcProperties>(
  ({ themeVariables, model }, ref) => {
    const rootRef = useRef<HTMLDivElement>(null);

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
          <Workbook model={model} workbookState={new WorkbookState()} />
        </I18nextProvider>
      </div>
    );
  },
);

IronCalc.displayName = "IronCalc";

export default IronCalc;
