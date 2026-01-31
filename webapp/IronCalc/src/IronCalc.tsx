import type { Model } from "@ironcalc/wasm";
import { ThemeProvider } from "@mui/material";
import { forwardRef, useImperativeHandle } from "react";
import { I18nextProvider } from "react-i18next";
import Workbook from "./components/Workbook/Workbook.tsx";
import { WorkbookState } from "./components/workbookState.ts";
import i18n from "./i18n";
import { theme } from "./theme.ts";

interface IronCalcProperties {
  model: Model;
}

export interface IronCalcHandle {
  setLanguage: (language: string) => void;
}

const IronCalc = forwardRef<IronCalcHandle, IronCalcProperties>(
  (properties, ref) => {
    useImperativeHandle(ref, () => ({
      setLanguage(language: string) {
        if (i18n.language !== language) {
          i18n.changeLanguage(language);
          const lang = language.split("-")[0];
          properties.model.setLanguage(lang);
        }
      },
    }));
    return (
      <ThemeProvider theme={theme}>
        <I18nextProvider i18n={i18n}>
          <Workbook
            model={properties.model}
            workbookState={new WorkbookState()}
          />
        </I18nextProvider>
      </ThemeProvider>
    );
  },
);

IronCalc.displayName = "IronCalc";

export default IronCalc;
