import { expect, test } from "vitest";
import {
  cellTextColor,
  defaultCellTextColor,
  isDarkBackground,
} from "../src/components/WorksheetCanvas/util";

const dark = {
  backgroundColor: "#000000",
  defaultTextColor: "#E4E4E4",
  filledCellTextColor: "#2E414D",
};
const light = {
  backgroundColor: "#FFFFFF",
  defaultTextColor: "#2E414D",
  filledCellTextColor: "#2E414D",
};

test("dark backgrounds are told from light ones", () => {
  expect(isDarkBackground("#000000")).toBe(true);
  expect(isDarkBackground("#1E1E1E")).toBe(true);
  expect(isDarkBackground("#FFFFFF")).toBe(false);
  expect(isDarkBackground("#F2994A")).toBe(false);
  expect(isDarkBackground("not a color")).toBe(false);
});

test("automatic text follows a dark theme", () => {
  // Text 1, as Excel and openpyxl write it
  expect(cellTextColor([1, 0], "#000000", dark, null)).toBe("#E4E4E4");
  // Plain black, as Google Sheets writes it
  expect(cellTextColor("#000000", "#000000", dark, null)).toBe("#E4E4E4");
});

test("automatic text is light on a dark fill, in either theme", () => {
  expect(cellTextColor([1, 0], "#000000", dark, "#202020")).toBe("#E4E4E4");
  expect(cellTextColor("#000000", "#000000", light, "#3D1616")).toBe("#FFFFFF");
});

test("automatic text stays dark on a light fill, in either theme", () => {
  expect(cellTextColor([1, 0], "#000000", dark, "#FFF2CC")).toBe("#000000");
  expect(cellTextColor([1, 0], "#000000", light, "#FFF2CC")).toBe("#000000");
});

test("chosen colors are kept", () => {
  expect(cellTextColor("#0000FF", "#0000FF", dark, null)).toBe("#0000FF");
  expect(cellTextColor("#0000FF", "#0000FF", dark, "#202020")).toBe("#0000FF");
  // Text 1 lightened is a deliberate gray
  expect(cellTextColor([1, 0.5], "#7F7F7F", dark, null)).toBe("#7F7F7F");
  expect(cellTextColor([4, 0], "#4472C4", dark, null)).toBe("#4472C4");
});

test("a light theme draws unfilled cells as resolved", () => {
  expect(cellTextColor([1, 0], "#000000", light, null)).toBe("#000000");
  expect(cellTextColor("#000000", "#000000", light, null)).toBe("#000000");
});

test("text without a font color follows its background", () => {
  expect(defaultCellTextColor(dark, null)).toBe("#E4E4E4");
  expect(defaultCellTextColor(light, null)).toBe("#2E414D");
  expect(defaultCellTextColor(dark, "#FFF2CC")).toBe("#2E414D");
  expect(defaultCellTextColor(dark, "#202020")).toBe("#E4E4E4");
  expect(defaultCellTextColor(light, "#202020")).toBe("#FFFFFF");
});
