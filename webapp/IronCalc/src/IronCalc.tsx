import "./index.css";
import type { Model } from "@ironcalc/wasm";
import ThemeProvider from "@mui/material/styles/ThemeProvider";
import Workbook from "./components/workbook.tsx";
import { WorkbookState } from "./components/workbookState.ts";
import { theme } from "./theme.ts";
import "./i18n";

interface IronCalcProperties {
  model: Model;
}

function IronCalc(properties: IronCalcProperties) {
  return (
    <ThemeProvider theme={theme}>
      <Workbook model={properties.model} workbookState={new WorkbookState()} />
    </ThemeProvider>
  );
}

export default IronCalc;
