// Shared logic for the Formula Helper: the function catalogue (functions.json)
// plus a single `getCompletion` entry point that asks the engine what the
// grammar expects at the cursor and normalises it into a small, UI-friendly
// shape. Both the popup (FormulaHelper.tsx) and the editor's keyboard handling
// consume the same `Completion` value.

import type { DefinedName, Model, NamedFunction } from "@ironcalc/wasm";
import rawFunctions from "./functions.json";

// One argument: [name, type, description]. A trailing "*" on the name marks the
// argument as optional. The literal name "..." is the repeating-args marker.
export type FunctionArg = [name: string, type: string, description: string];

export interface FunctionInfo {
  tier: number;
  category: number;
  tags: string[];
  args: FunctionArg[];
  description: string;
  examples: string[];
}

const functions = rawFunctions as unknown as Record<string, FunctionInfo>;

// Upper-cased function names, sorted, computed once.
const functionNames = Object.keys(functions)
  .map((name) => name.toUpperCase())
  .sort();

export function lookup(name: string): FunctionInfo | undefined {
  return functions[name.toLowerCase()];
}

export function firstSentence(text: string): string {
  const match = text.match(/^.*?[.!?](\s|$)/);
  return (match ? match[0] : text).trim();
}

// Maps the numeric `category` in functions.json to the docs site folder (which
// mirrors docs/src/functions/<folder>/<name>.md). Categories without a folder
// here (e.g. legacy functions) have no dedicated docs page.
const CATEGORY_FOLDERS: Record<number, string> = {
  1: "database",
  2: "date_and_time",
  3: "engineering",
  4: "financial",
  5: "information",
  6: "logical",
  7: "lookup_and_reference",
  8: "math_and_trigonometry",
  9: "statistical",
  10: "text",
};

// The documentation URL for a function, or null if it has no docs page. The
// page file keeps the name's dots (e.g. "t.test.html"); the heading anchor
// slugifies dots/spaces to dashes and is only appended when it differs from the
// file name. So ACCRINT -> ".../financial/accrint.html" and T.TEST ->
// ".../statistical/t.test.html#t-test".
export function docsUrl(name: string, category: number): string | null {
  const folder = CATEGORY_FOLDERS[category];
  if (!folder) {
    return null;
  }
  const file = name.toLowerCase();
  const anchor = file.replace(/[ .]/g, "-");
  const url = `https://docs.ironcalc.com/functions/${folder}/${file}.html`;
  return anchor === file ? url : `${url}#${anchor}`;
}

// How an argument is shown in the signature: "..." stays as is, optional args
// (trailing "*") are wrapped in brackets Excel-style, required args are plain.
export function displayArgName(argName: string): string {
  if (argName === "...") {
    return "…";
  }
  return argName.endsWith("*") ? `[${argName.slice(0, -1)}]` : argName;
}

// One row of the list. Besides the built-in functions of the catalogue there
// are the things the user defined in the workbook:
//
//   * "builtin":  a built-in function (SUM, VLOOKUP…)
//   * "function": a named function, with the description the user gave it
//   * "name":     a defined name (named range), with its comment
//
// `name` is what gets inserted: upper case for built-ins, as the user wrote it
// for the other two. `description` is empty when there is none.
export type CompletionItemKind = "builtin" | "function" | "name";

export interface CompletionItem {
  kind: CompletionItemKind;
  name: string;
  description: string;
}

// Listing names (e.g. "=SU"): the typed `prefix`, the `matches`, and the
// scalar offset where the partial name starts (so it can be replaced when a
// completion is accepted).
export interface ListCompletion {
  kind: "list";
  prefix: string;
  matches: CompletionItem[];
  replaceFrom: number;
}

// Filling in a function's arguments (e.g. "=SUMIF("): the function `name` and
// the 0-based index of the argument under the cursor. `namedFunction` is set
// when the function is one the user defined, rather than a built-in one.
export interface DetailCompletion {
  kind: "detail";
  name: string;
  activeIndex: number;
  namedFunction?: NamedFunction;
}

export type Completion = ListCompletion | DetailCompletion | null;

// The entries of `entries` that can be used from `sheet` and are meant to be
// seen: the workbook ones and those local to the sheet, minus the hidden ones.
// A local entry shadows a workbook one with the same name, as it does in a
// formula. Names are case-insensitive.
function visibleFrom<
  T extends { name: string; scope?: number | null; hidden: boolean },
>(entries: T[], sheet: number): T[] {
  const byName = new Map<string, T>();
  for (const entry of entries) {
    const isLocal = entry.scope !== undefined && entry.scope !== null;
    if (entry.hidden || (isLocal && entry.scope !== sheet)) {
      continue;
    }
    const key = entry.name.toUpperCase();
    if (isLocal || !byName.has(key)) {
      byName.set(key, entry);
    }
  }
  return [...byName.values()];
}

function namedFunctions(model: Model, sheet: number): NamedFunction[] {
  return visibleFrom(model.getNamedFunctionList(), sheet);
}

function definedNames(model: Model, sheet: number): DefinedName[] {
  return visibleFrom(model.getDefinedNameList(), sheet);
}

// Ask the engine what is expected at `cursor` (a UTF-16 offset into `text`) and
// turn it into a `Completion`. Returns null when there is nothing to suggest.
export function getCompletion(
  model: Model,
  text: string,
  cursor: number,
): Completion {
  let context: ReturnType<Model["getFormulaCompletion"]>;
  let sheet: number;
  try {
    const [selectedSheet, row, column] = model.getSelectedCell();
    sheet = selectedSheet;
    // The engine wants a Unicode scalar offset, not a UTF-16 one.
    const scalarCursor = Array.from(text.slice(0, cursor)).length;
    context = model.getFormulaCompletion(
      sheet,
      row,
      column,
      text,
      scalarCursor,
    );
  } catch (error) {
    console.error("getCompletion: getFormulaCompletion failed", error);
    return null;
  }

  // Filling in a function's arguments takes priority over the name list.
  const argHint = context.expecting.find(
    (token): token is { Argument: [string, number] } =>
      typeof token === "object" && "Argument" in token,
  );
  if (argHint) {
    const [name, oneBasedIndex] = argHint.Argument;
    const activeIndex = oneBasedIndex - 1;
    const upperName = name.toUpperCase();
    const namedFunction = namedFunctions(model, sheet).find(
      (entry) => entry.name.toUpperCase() === upperName,
    );
    if (namedFunction) {
      return {
        kind: "detail",
        name: namedFunction.name,
        activeIndex,
        namedFunction,
      };
    }
    if (!lookup(name)) {
      return null;
    }
    return { kind: "detail", name, activeIndex };
  }

  const nameHint = context.expecting.find(
    (token): token is { FunctionName: string } =>
      typeof token === "object" && "FunctionName" in token,
  );
  if (nameHint) {
    const prefix = nameHint.FunctionName.toUpperCase();
    const startsWithPrefix = (name: string) =>
      name.toUpperCase().startsWith(prefix);
    const matches: CompletionItem[] = [
      ...functionNames.filter(startsWithPrefix).map(
        (name): CompletionItem => ({
          kind: "builtin",
          name,
          description: firstSentence(lookup(name)?.description ?? ""),
        }),
      ),
      ...namedFunctions(model, sheet)
        .filter((entry) => startsWithPrefix(entry.name))
        .map(
          (entry): CompletionItem => ({
            kind: "function",
            name: entry.name,
            description: entry.description,
          }),
        ),
      ...definedNames(model, sheet)
        .filter((entry) => startsWithPrefix(entry.name))
        .map(
          (entry): CompletionItem => ({
            kind: "name",
            name: entry.name,
            description: entry.comment,
          }),
        ),
    ].sort((a, b) => {
      const nameA = a.name.toUpperCase();
      const nameB = b.name.toUpperCase();
      return nameA < nameB ? -1 : nameA > nameB ? 1 : 0;
    });
    if (matches.length === 0) {
      return null;
    }
    return { kind: "list", prefix, matches, replaceFrom: context.replace_from };
  }

  return null;
}

// Accept a row of a list completion: replace the partial name with `NAME(` for
// a function, or just `NAME` for a defined name, and report where the caret
// should land (right after what was inserted).
//
// `text` is the full cell input including any leading "=" and `cursor` is a
// UTF-16 offset into it. `completion.replaceFrom` is a Unicode *scalar* offset
// into the formula body (the engine strips a leading "=" before parsing), so we
// add the "=" back and convert to a UTF-16 index before splicing.
export function applyListCompletion(
  text: string,
  cursor: number,
  completion: ListCompletion,
  selected: number,
): { text: string; cursor: number } {
  const index = Math.min(Math.max(selected, 0), completion.matches.length - 1);
  const item = completion.matches[index];
  const insert = item.kind === "name" ? item.name : `${item.name}(`;

  const equalsOffset = text.startsWith("=") ? 1 : 0;
  const startScalar = completion.replaceFrom + equalsOffset;
  const startUtf16 = Array.from(text).slice(0, startScalar).join("").length;

  const newText = text.slice(0, startUtf16) + insert + text.slice(cursor);
  return { text: newText, cursor: startUtf16 + insert.length };
}
