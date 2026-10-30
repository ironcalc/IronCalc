import { readFile } from "node:fs/promises";
import { initSync } from "@ironcalc/wasm";
import { beforeAll, expect, test } from "vitest";
import {
  firstFreeName,
  getFormulaSegments,
} from "../src/components/RightDrawer/NamedFunctions/formulaSegments";

beforeAll(async () => {
  const buffer = await readFile("node_modules/@ironcalc/wasm/wasm_bg.wasm");
  initSync({ module: buffer });
});

test("arguments become chips, the rest is text", () => {
  const segments = getFormulaSegments("(revenue - cost) / revenue", [
    "revenue",
    "cost",
  ]);
  expect(segments).toEqual([
    { text: "(", kind: "text" },
    { text: "revenue", kind: "argument" },
    { text: " - ", kind: "text" },
    { text: "cost", kind: "argument" },
    { text: ") / ", kind: "text" },
    { text: "revenue", kind: "argument" },
  ]);
});

test("unknown names reported by the check are flagged, functions are not", () => {
  const segments = getFormulaSegments(
    "=IF(x > 0, x * FACTORIAL(x - 1), coste)",
    ["x"],
    ["coste"],
  );
  expect(segments).toEqual([
    { text: "=IF(", kind: "text" },
    { text: "x", kind: "argument" },
    { text: " > 0, ", kind: "text" },
    { text: "x", kind: "argument" },
    { text: " * FACTORIAL(", kind: "text" },
    { text: "x", kind: "argument" },
    { text: " - 1), ", kind: "text" },
    { text: "coste", kind: "unknown" },
    { text: ")", kind: "text" },
  ]);
});

test("matching is case-sensitive and strings are left alone", () => {
  const segments = getFormulaSegments('Revenue & "revenue"', ["revenue"]);
  expect(segments).toEqual([{ text: 'Revenue & "revenue"', kind: "text" }]);
});

test("an empty formula has no segments", () => {
  expect(getFormulaSegments("", ["x"])).toEqual([]);
  expect(getFormulaSegments("=", ["x"])).toEqual([{ text: "=", kind: "text" }]);
});

test("first free name appends a counter", () => {
  expect(firstFreeName("NEW_FUNCTION", [])).toBe("NEW_FUNCTION");
  expect(firstFreeName("NEW_FUNCTION", ["new_function"])).toBe(
    "NEW_FUNCTION_1",
  );
  expect(firstFreeName("HOLA", ["HOLA", "HOLA_1", "Other"])).toBe("HOLA_2");
});
