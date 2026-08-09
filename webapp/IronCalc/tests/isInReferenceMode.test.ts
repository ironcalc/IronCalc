import { readFile } from "node:fs/promises";
import { initSync, Model } from "@ironcalc/wasm";
import { beforeAll, expect, test } from "vitest";
import { isInReferenceMode } from "../src/components/Editor/util";

let model: Model;

beforeAll(async () => {
  const buffer = await readFile("node_modules/@ironcalc/wasm/wasm_bg.wasm");
  initSync({ module: buffer });
  model = new Model("workbook", "en", "UTC", "en");
});

// `cursor` defaults to the end of the text (a UTF-16 offset, as the editor
// gives us).
const ref = (text: string, cursor = text.length): boolean =>
  isInReferenceMode(model, text, cursor);

// ---------------------------------------------------------------------------
// Positions that admit a reference/range
// ---------------------------------------------------------------------------

test("just '=' is reference mode", () => {
  expect(ref("=")).toBe(true);
});

test("right after an open function call", () => {
  expect(ref("=SUM(")).toBe(true);
});

test("right after a dangling binary operator", () => {
  expect(ref("=A1+")).toBe(true);
  expect(ref("=1+2*")).toBe(true);
});

test("right after an argument separator", () => {
  expect(ref("=SUM(A1,")).toBe(true);
});

// ---------------------------------------------------------------------------
// Positions that do NOT admit a reference
// ---------------------------------------------------------------------------

test("plain text (no leading '=') is never reference mode", () => {
  expect(ref("hello")).toBe(false);
  expect(ref("")).toBe(false);
});

test("a complete reference is not reference mode", () => {
  expect(ref("=A1")).toBe(false);
});

test("a complete function call is not reference mode", () => {
  expect(ref("=SUM(A1)")).toBe(false);
});

test("while typing a function name it is not reference mode", () => {
  expect(ref("=A1+SU")).toBe(false);
});

test('inside a string is not reference mode (fixes the old "=1+ false positive)', () => {
  // The old heuristic looked at the last character (`+`) and wrongly reported
  // reference mode; the cursor is actually inside the string literal "1+".
  expect(ref('="1+')).toBe(false);
});

test("inside a string literal", () => {
  expect(ref('=SUM("hello')).toBe(false);
});

// ---------------------------------------------------------------------------
// UTF-16 vs Unicode scalar offsets
// ---------------------------------------------------------------------------

test("cursor offset is treated as UTF-16, not scalar (astral characters)", () => {
  // `😀` is a single scalar but two UTF-16 code units, so the cursor index the
  // editor reports is larger than the scalar offset the engine expects.
  // Cursor sits right after the `&`, where a reference is admitted.
  const text = '="😀"&A1';
  const cursor = text.indexOf("&") + 1; // UTF-16 index (6), scalar offset is 5
  expect(ref(text, cursor)).toBe(true);

  // Sanity: with the same astral char, a position inside the string is not
  // reference mode.
  expect(ref('="😀+', '="😀+'.length)).toBe(false);
});
