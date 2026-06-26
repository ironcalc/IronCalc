import { readFile } from "node:fs/promises";
import { initSync, Model } from "@ironcalc/wasm";
import { beforeAll, expect, test } from "vitest";
import {
  applyListCompletion,
  getCompletion,
  type ListCompletion,
} from "../src/components/FormulaHelper/formulaCompletion";

let model: Model;

beforeAll(async () => {
  const buffer = await readFile("node_modules/@ironcalc/wasm/wasm_bg.wasm");
  initSync({ module: buffer });
  model = new Model("workbook", "en", "UTC", "en");
});

// Resolve the list completion for `text`/`cursor` and accept the given function
// name, returning the resulting cell input. The engine drives `replaceFrom`, so
// this exercises the real "=" stripping behaviour, not a hand-built fixture.
function acceptFunction(
  text: string,
  cursor: number,
  functionName: string,
): { text: string; cursor: number } {
  const completion = getCompletion(model, text, cursor);
  expect(completion?.kind).toBe("list");
  const list = completion as ListCompletion;
  const selected = list.matches.indexOf(functionName);
  expect(selected).toBeGreaterThanOrEqual(0);
  return applyListCompletion(text, cursor, list, selected);
}

test("accepting a function keeps the leading '=' (regression)", () => {
  const result = acceptFunction("=SU", 3, "SUM");
  expect(result.text).toBe("=SUM(");
  // Caret sits just inside the parenthesis.
  expect(result.cursor).toBe(5);
});

test("accepting a function mid-formula only replaces the partial name", () => {
  const result = acceptFunction("=1+SU", 5, "SUM");
  expect(result.text).toBe("=1+SUM(");
  expect(result.cursor).toBe(7);
});

test("text after the cursor is preserved", () => {
  // Cursor sits right after "SU", before "+1".
  const result = acceptFunction("=SU+1", 3, "SUM");
  expect(result.text).toBe("=SUM(+1");
  expect(result.cursor).toBe(5);
});

test("the partial name is matched case-insensitively", () => {
  const result = acceptFunction("=su", 3, "SUM");
  expect(result.text).toBe("=SUM(");
});
