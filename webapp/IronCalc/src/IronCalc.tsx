import "./index.css";
import type { Model } from "@ironcalc/wasm";
import ThemeProvider from "@mui/material/styles/ThemeProvider";
import Workbook from "./components/Workbook/Workbook.tsx";
import { WorkbookState } from "./components/workbookState.ts";
import { theme } from "./theme.ts";
import "./i18n";

interface IronCalcProperties {
  model: Model;
}

function IronCalc(properties: IronCalcProperties) {
  properties.model.setUsers([
    { id: "john@doe.com", sheet: 0, row: 5, column: 6 },
    { id: "micheal@doe.com", sheet: 0, row: 1, column: 6 },
  ]);
  return (
    <ThemeProvider theme={theme}>
      <Workbook model={properties.model} workbookState={new WorkbookState()} />
    </ThemeProvider>
  );
}

export default IronCalc;
