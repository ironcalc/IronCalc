import { expect, test } from "vitest";
import {
  cellTextColor,
  isDarkBackground,
} from "../src/components/WorksheetCanvas/util";

const dark = { backgroundColor: "#000000", defaultTextColor: "#E4E4E4" };
const light = { backgroundColor: "#FFFFFF", defaultTextColor: "#2E414D" };

test("dark backgrounds are told from light ones", () => {
  expect(isDarkBackground("#000000")).toBe(true);
  expect(isDarkBackground("#1E1E1E")).toBe(true);
  expect(isDarkBackground("#FFFFFF")).toBe(false);
  expect(isDarkBackground("#F2994A")).toBe(false);
  expect(isDarkBackground("not a color")).toBe(false);
});

test("automatic text follows a dark theme", () => {
  // Text 1, as Excel and openpyxl write it
  expect(cellTextColor([1, 0], "#000000", dark)).toBe("#E4E4E4");
  // Plain black, as Google Sheets writes it
  expect(cellTextColor("#000000", "#000000", dark)).toBe("#E4E4E4");
});

test("chosen colors are kept in a dark theme", () => {
  expect(cellTextColor("#0000FF", "#0000FF", dark)).toBe("#0000FF");
  // Text 1 lightened is a deliberate gray
  expect(cellTextColor([1, 0.5], "#7F7F7F", dark)).toBe("#7F7F7F");
  expect(cellTextColor([4, 0], "#4472C4", dark)).toBe("#4472C4");
});

test("a light theme draws every color as resolved", () => {
  expect(cellTextColor([1, 0], "#000000", light)).toBe("#000000");
  expect(cellTextColor("#000000", "#000000", light)).toBe("#000000");
});
