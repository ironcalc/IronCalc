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
  const selected = list.matches.findIndex((item) => item.name === functionName);
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

// ---- Named functions and defined names ----

function workbookWithNames(): Model {
  const m = new Model("workbook", "en", "UTC", "en");
  m.newSheet();
  m.newNamedFunction({
    name: "Margin",
    formula: "(revenue-cost)/revenue",
    description: "Share of the revenue that is profit.",
    parameters: [
      { name: "revenue", description: "Total revenue" },
      { name: "cost", optional: true },
    ],
  });
  m.newNamedFunction({ name: "MaSecret", formula: "1", hidden: true });
  m.newNamedFunction({ name: "MaOther", formula: "2", scope: 1 });
  m.newDefinedName({
    name: "MaxRate",
    formula: "Sheet1!$A$1",
    comment: "Highest rate allowed",
  });
  // Adding a sheet selects it: go back to the first one.
  m.setSelectedSheet(0);
  return m;
}

test("the list mixes built-ins, named functions and defined names", () => {
  const m = workbookWithNames();
  const completion = getCompletion(m, "=ma", 3) as ListCompletion;
  expect(completion.kind).toBe("list");
  const byName = new Map(completion.matches.map((item) => [item.name, item]));
  expect(byName.get("MATCH")?.kind).toBe("builtin");
  expect(byName.get("Margin")).toEqual({
    kind: "function",
    name: "Margin",
    description: "Share of the revenue that is profit.",
  });
  expect(byName.get("MaxRate")).toEqual({
    kind: "name",
    name: "MaxRate",
    description: "Highest rate allowed",
  });
  // Hidden entries and those local to another sheet are left out.
  expect(byName.has("MaSecret")).toBe(false);
  expect(byName.has("MaOther")).toBe(false);
  // A local function is offered in its own sheet.
  m.setSelectedSheet(1);
  const local = getCompletion(m, "=ma", 3) as ListCompletion;
  expect(local.matches.some((item) => item.name === "MaOther")).toBe(true);
  // Sorted by name, whatever the kind.
  const names = completion.matches.map((item) => item.name.toUpperCase());
  expect(names).toEqual([...names].sort());
});

test("a named function is accepted with its own casing and a parenthesis", () => {
  const m = workbookWithNames();
  const completion = getCompletion(m, "=1+marg", 7) as ListCompletion;
  const selected = completion.matches.findIndex((i) => i.name === "Margin");
  const result = applyListCompletion("=1+marg", 7, completion, selected);
  expect(result).toEqual({ text: "=1+Margin(", cursor: 10 });
});

test("a defined name is accepted without a parenthesis", () => {
  const m = workbookWithNames();
  const completion = getCompletion(m, "=maxr", 5) as ListCompletion;
  const selected = completion.matches.findIndex((i) => i.name === "MaxRate");
  const result = applyListCompletion("=maxr", 5, completion, selected);
  expect(result).toEqual({ text: "=MaxRate", cursor: 8 });
});

test("the arguments of a named function carry its definition", () => {
  const m = workbookWithNames();
  const completion = getCompletion(m, "=margin(A1,", 11);
  expect(completion?.kind).toBe("detail");
  if (completion?.kind !== "detail") {
    return;
  }
  expect(completion.name).toBe("Margin");
  expect(completion.activeIndex).toBe(1);
  expect(completion.namedFunction?.parameters).toEqual([
    { name: "revenue", description: "Total revenue", optional: false },
    { name: "cost", description: "", optional: true },
  ]);
});
