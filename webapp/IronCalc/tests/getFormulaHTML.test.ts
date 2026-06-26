import { readFile } from "node:fs/promises";
import { initSync, Model } from "@ironcalc/wasm";
import type { ReactElement } from "react";
import { beforeAll, expect, test } from "vitest";
import getFormulaHTML, { getColor } from "../src/components/Editor/util";

let model: Model;

beforeAll(async () => {
  const buffer = await readFile("node_modules/@ironcalc/wasm/wasm_bg.wasm");
  initSync({ module: buffer });
  model = new Model("workbook", "en", "UTC", "en");
});

// Each entry of `html` is a React element (a plain object). The text is in
// `props.children` and the (optional) reference color in `props.style.color`.
const text = (el: ReactElement): string =>
  (el.props as { children: string }).children;
const color = (el: ReactElement): string | undefined =>
  (el.props as { style?: { color?: string } }).style?.color;

test("plain text (no leading '=') is wrapped in a single span", () => {
  const { html, activeRanges } = getFormulaHTML(model, "hello");
  expect(html).toHaveLength(1);
  expect(text(html[0])).toBe("hello");
  expect(activeRanges).toEqual([]);
});

test("a formula always starts with an '=' span", () => {
  const { html } = getFormulaHTML(model, "=1+1");
  expect(text(html[0])).toBe("=");
  // "1+1" is a single non-reference run.
  expect(html.map(text).join("")).toBe("=1+1");
});

test("a reference produces a colored span and an active range", () => {
  const { html, activeRanges } = getFormulaHTML(model, "=A1+1");
  expect(html.map(text).join("")).toBe("=A1+1");

  const a1 = html.find((el) => text(el) === "A1");
  expect(a1).toBeDefined();
  expect(color(a1 as ReactElement)).toBeTruthy();

  expect(activeRanges).toHaveLength(1);
  expect(activeRanges[0]).toMatchObject({
    sheet: 0,
    rowStart: 1,
    columnStart: 1,
    rowEnd: 1,
    columnEnd: 1,
  });
});

test("the same reference reuses the same color", () => {
  const { activeRanges } = getFormulaHTML(model, "=A1+A1");
  expect(activeRanges).toHaveLength(2);
  expect(activeRanges[0].color).toBe(activeRanges[1].color);
});

test("distinct references get distinct colors", () => {
  const { activeRanges } = getFormulaHTML(model, "=A1+B2");
  expect(activeRanges).toHaveLength(2);
  expect(activeRanges[0].color).not.toBe(activeRanges[1].color);
});

test("a range reference yields a single normalized active range", () => {
  const { html, activeRanges } = getFormulaHTML(model, "=SUM(A1:B3)");
  expect(html.map(text).join("")).toBe("=SUM(A1:B3)");
  expect(activeRanges).toHaveLength(1);
  expect(activeRanges[0]).toMatchObject({
    sheet: 0,
    rowStart: 1,
    columnStart: 1,
    rowEnd: 3,
    columnEnd: 2,
  });
});

test("distinct ranges get consecutive palette colors", () => {
  const { activeRanges } = getFormulaHTML(model, "=SUM(A2:A5)+SUM(B2:B5)");
  expect(activeRanges).toHaveLength(2);
  // First range takes the first palette color...
  expect(activeRanges[0].color).toBe(getColor(0));
  // ...so the second distinct range should take the second one, not skip ahead.
  expect(activeRanges[1].color).toBe(getColor(1));
});

test("a range does not skip a palette color for a following reference", () => {
  const { activeRanges } = getFormulaHTML(model, "=SUM(A2:A5)+B7");
  expect(activeRanges).toHaveLength(2);
  expect(activeRanges[0].color).toBe(getColor(0));
  expect(activeRanges[1].color).toBe(getColor(1));
});

// The reference-insertion hint is identified by its stable key/className.
const isHint = (el: ReactElement): boolean =>
  el.key === "reference-hint" ||
  (el.props as { className?: string }).className === "insert-reference-hint";

test("no cursor argument never renders the reference hint", () => {
  const { html } = getFormulaHTML(model, "=SUM(");
  expect(html.some(isHint)).toBe(false);
});

test("a caret in reference mode renders a hint after the caret", () => {
  // `=SUM(|` — the grammar expects a reference here and none follows.
  const { html } = getFormulaHTML(model, "=SUM(", 5);
  const hints = html.filter(isHint);
  expect(hints).toHaveLength(1);
  // It is appended at the very end (the caret is past every token).
  expect(isHint(html[html.length - 1])).toBe(true);
  // The hint carries no formula text, so the reconstructed formula is unchanged.
  expect(
    html
      .filter((el) => !isHint(el))
      .map(text)
      .join(""),
  ).toBe("=SUM(");
});

test("the hint is inserted before the token following the caret", () => {
  // `=SUM(|)` — caret between `(` and `)`; expects a reference, `)` is not one.
  const { html } = getFormulaHTML(model, "=SUM()", 5);
  const hintIndex = html.findIndex(isHint);
  expect(hintIndex).toBeGreaterThanOrEqual(0);
  // The closing paren span comes right after the hint.
  expect(text(html[hintIndex + 1])).toBe(")");
});

test("no hint when a reference already follows the caret", () => {
  // `=SUM(|A1)` — a reference sits immediately after the caret.
  const { html } = getFormulaHTML(model, "=SUM(A1)", 5);
  expect(html.some(isHint)).toBe(false);
});

test("no hint when the caret is not in reference mode", () => {
  // `=1+1|` — the grammar does not expect a reference at the end here.
  const { html } = getFormulaHTML(model, "=1+1", 4);
  expect(html.some(isHint)).toBe(false);
});

test("a hint appears after a trailing operator", () => {
  // `=A1+|` — after the `+` a reference is expected and none follows.
  const { html } = getFormulaHTML(model, "=A1+", 4);
  const hints = html.filter(isHint);
  expect(hints).toHaveLength(1);
  expect(isHint(html[html.length - 1])).toBe(true);
});

test("a trailing newline adds a trailing span", () => {
  const { html } = getFormulaHTML(model, "=A1\n");
  expect(text(html[html.length - 1])).toBe("\n");
});

test("a spill reference (A1#) covers the whole spilled range", () => {
  // Make A1 spill a 1x3 dynamic array down A1:A3.
  model.setUserInput(0, 1, 1, "={1;2;3}");
  expect(model.getCellArrayStructure(0, 1, 1)).toEqual({
    DynamicAnchor: [1, 3],
  });

  const { html, activeRanges } = getFormulaHTML(model, "=A1#+1");
  // The reference and its spill operator are kept together in one span.
  const anchor = html.find((el) => text(el) === "A1#");
  expect(anchor).toBeDefined();
  expect(color(anchor as ReactElement)).toBeTruthy();
  expect(html.map(text).join("")).toBe("=A1#+1");

  expect(activeRanges).toHaveLength(1);
  expect(activeRanges[0]).toMatchObject({
    sheet: 0,
    rowStart: 1,
    columnStart: 1,
    rowEnd: 3,
    columnEnd: 1,
  });
});

test("non-spilling A1# still reconstructs the formula text", () => {
  // B5 is empty, so it does not spill (structure is "SingleCell").
  expect(model.getCellArrayStructure(0, 5, 2)).toBe("SingleCell");

  const { html, activeRanges } = getFormulaHTML(model, "=B5#+1");
  expect(activeRanges).toHaveLength(0);
  expect(html.map(text).join("")).toBe("=B5#+1");
});

test("a spill reference does not skip a palette color for the next reference", () => {
  // A1 spills A1:A3; B2 is a distinct, non-spilling reference.
  model.setUserInput(0, 1, 1, "={1;2;3}");
  expect(model.getCellArrayStructure(0, 1, 1)).toEqual({
    DynamicAnchor: [1, 3],
  });

  const { activeRanges } = getFormulaHTML(model, "=A1#+B2");
  expect(activeRanges).toHaveLength(2);
  // First reference takes the first palette color...
  expect(activeRanges[0].color).toBe(getColor(0));
  // ...so the second distinct reference should take the second one.
  expect(activeRanges[1].color).toBe(getColor(1));
});
