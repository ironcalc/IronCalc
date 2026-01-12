import "./index.css";
import type { Model } from "@ironcalc/wasm";
import { ThemeProvider } from "@mui/material";
import Workbook from "./components/Workbook/Workbook.tsx";
import { WorkbookState } from "./components/workbookState.ts";
import { theme } from "./theme.ts";
import "./i18n";
import { useEffect } from "react";
import { useTranslation } from "react-i18next";

interface IronCalcProperties {
  model: Model;
  language: string;
}

function IronCalc(properties: IronCalcProperties) {
  const { i18n } = useTranslation();

  useEffect(() => {
    if (i18n.language !== properties.language) {
      i18n.changeLanguage(properties.language);
    }
  }, [properties.language, i18n]);
  return (
    <ThemeProvider theme={theme}>
      <Workbook model={properties.model} workbookState={new WorkbookState()} />
    </ThemeProvider>
  );
}

export default IronCalc;
