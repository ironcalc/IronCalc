import { readFile } from "node:fs/promises";
import { type SelectedView, initSync } from "@ironcalc/wasm";
import { expect, test } from "vitest";
import {
  decreaseDecimalPlaces,
  increaseDecimalPlaces,
} from "../FormatMenu/formatUtil";
import { getFullRangeToString, isNavigationKey } from "../util";

test("checks arrow left is a navigation key", () => {
  expect(isNavigationKey("ArrowLeft")).toBe(true);
  expect(isNavigationKey("Arrow")).toBe(false);
});

test("increase decimals", () => {
  expect(increaseDecimalPlaces('"€"#,##0.00'), '"€"#,##0.000');
  expect(increaseDecimalPlaces("general"), "#,##0.000");
  expect(
    increaseDecimalPlaces('dddd"," mmmm dd"," yyyy'),
    'dddd"," mmmm dd"," yyyy',
  );
});

test("decrease decimals", () => {
  expect(decreaseDecimalPlaces('"€"#,##0.00'), '"€"#,##0.0');
  expect(decreaseDecimalPlaces("general"), "#,##0.0");
  expect(
    decreaseDecimalPlaces('dddd"," mmmm dd"," yyyy'),
    'dddd"," mmmm dd"," yyyy',
  );
});

test("format range to get the full formula", async () => {
  const buffer = await readFile("node_modules/@ironcalc/wasm/wasm_bg.wasm");
  initSync(buffer);

  const selectedView: SelectedView = {
    sheet: 0,
    row: 1,
    column: 8,
    range: [1, 8, 1, 8],
    top_row: 1,
    left_column: 8,
  };
  const worksheetNames = ["Sheet1", "Notes"];

  expect(getFullRangeToString(selectedView, worksheetNames)).toBe(
    "Sheet1!$H$1",
  );
});
