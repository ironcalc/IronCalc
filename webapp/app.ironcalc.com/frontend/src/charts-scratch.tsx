// Temporary dev page: a workbook with a small table and a chart.
import { IronCalc, init, Model } from "@ironcalc/workbook";
import "@ironcalc/workbook/style.css";
import { createRoot } from "react-dom/client";
import "./fonts.css";
import "./i18n";

async function main() {
  await init();
  const model = new Model("charts", "en", "UTC", "en");
  const rows = [
    ["City", "September", "October", "November"],
    ["London", "123", "167", "120"],
    ["Madrid", "23", "456", "170"],
    ["Rome", "234", "150", "200"],
    ["Paris", "156", "300", "250"],
    ["Amsterdam", "123", "345", "321"],
  ];
  rows.forEach((row, r) =>
    row.forEach((value, c) => model.setUserInput(0, r + 2, c + 2, value)),
  );
  model.setSelectedCell(3, 3);
  const root = document.getElementById("root");
  if (!root) {
    return;
  }
  createRoot(root).render(<IronCalc model={model} canEdit={true} />);
}

main();
