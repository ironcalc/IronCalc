import { readFile } from "node:fs/promises";
import { initSync, Model } from "@ironcalc/wasm";
import { beforeAll, describe, expect, test } from "vitest";
import {
  addRun,
  compositionReading,
  initialRuns,
  isKana,
  type PhoneticState,
  savePhoneticRuns,
  shiftRuns,
  type TrackedRun,
  toKatakana,
  validRuns,
} from "../src/components/Editor/phonetic";

beforeAll(async () => {
  const buffer = await readFile("node_modules/@ironcalc/wasm/wasm_bg.wasm");
  initSync({ module: buffer });
});

function run(start: number, end: number, text: string, base: string) {
  return { start, end, text, base };
}

// Simulates typing: the composition of `reading` is converted to `committed` at `start`
function compose(
  state: PhoneticState,
  start: number,
  reading: string,
  committed: string,
) {
  const before = state.text;
  const after = before.slice(0, start) + committed + before.slice(start);
  state.runs = shiftRuns(state.runs, state.text, after);
  state.text = after;
  const katakana = compositionReading(committed, reading, committed);
  if (katakana) {
    addRun(state, start, committed, katakana);
  }
}

// Simulates an edit that is not a composition (typing ASCII, deleting, pasting)
function edit(state: PhoneticState, after: string) {
  state.runs = shiftRuns(state.runs, state.text, after);
  state.text = after;
}

describe("kana", () => {
  test("isKana", () => {
    expect(isKana("やまだ")).toBe(true);
    expect(isKana("コーヒー")).toBe(true);
    expect(isKana("山田")).toBe(false);
    expect(isKana("やまだ太郎")).toBe(false);
    expect(isKana("")).toBe(false);
  });
  test("toKatakana", () => {
    expect(toKatakana("おみおつけ")).toBe("オミオツケ");
    expect(toKatakana("きょうと")).toBe("キョウト");
    expect(toKatakana("ゔぁ")).toBe("ヴァ");
  });
});

describe("compositionReading", () => {
  test("a converted composition is read as its kana", () => {
    expect(compositionReading("御御御付", "おみおつけ", "御御御付")).toBe(
      "オミオツケ",
    );
  });
  test("kana that are not converted need no reading", () => {
    expect(compositionReading("すし", "すし", "すし")).toBeNull();
    expect(compositionReading("スシ", "すし", "スシ")).toBeNull();
  });
  test("a partly committed composition is not recorded", () => {
    // The IME committed 山田 while 太郎 was still being composed
    expect(compositionReading("山田", "やまだたろう", "山田太郎")).toBeNull();
  });
  test("no kana seen (pasted with the IME, a symbol) is not recorded", () => {
    expect(compositionReading("山田", "", "山田")).toBeNull();
  });
});

describe("editing keeps the readings right", () => {
  test("two conversions", () => {
    const state: PhoneticState = { text: "", runs: [] };
    compose(state, 0, "やまだ", "山田");
    compose(state, 2, "たろう", "太郎");
    expect(validRuns(state.runs, state.text)).toEqual([
      { start: 0, end: 2, text: "ヤマダ" },
      { start: 2, end: 4, text: "タロウ" },
    ]);
  });

  test("typing before a run moves it", () => {
    const state: PhoneticState = { text: "", runs: [] };
    compose(state, 0, "やまだ", "山田");
    edit(state, "Mr 山田");
    expect(validRuns(state.runs, state.text)).toEqual([
      { start: 3, end: 5, text: "ヤマダ" },
    ]);
  });

  test("typing after a run keeps it", () => {
    const state: PhoneticState = { text: "", runs: [] };
    compose(state, 0, "やまだ", "山田");
    edit(state, "山田さん");
    expect(validRuns(state.runs, state.text)).toEqual([
      { start: 0, end: 2, text: "ヤマダ" },
    ]);
  });

  test("changing a character of a run drops its reading", () => {
    const state: PhoneticState = { text: "", runs: [] };
    compose(state, 0, "やまだ", "山田");
    compose(state, 2, "たろう", "太郎");
    // 田 is deleted and 中 is typed: 山中太郎
    edit(state, "山太郎");
    compose(state, 1, "なか", "中");
    expect(validRuns(state.runs, state.text)).toEqual([
      { start: 1, end: 2, text: "ナカ" },
      { start: 2, end: 4, text: "タロウ" },
    ]);
  });

  test("deleting a run drops it", () => {
    const state: PhoneticState = { text: "", runs: [] };
    compose(state, 0, "やまだ", "山田");
    compose(state, 2, "たろう", "太郎");
    edit(state, "太郎");
    expect(validRuns(state.runs, state.text)).toEqual([
      { start: 0, end: 2, text: "タロウ" },
    ]);
  });

  test("a run that ends up on other characters is dropped when saving", () => {
    // A diff of repeated characters can be ambiguous: the check against the
    // characters the run was recorded for catches it
    const runs: TrackedRun[] = [run(0, 2, "ヤマダ", "山田")];
    expect(validRuns(runs, "山中")).toEqual([]);
  });

  test("converting again over a run replaces it", () => {
    const state: PhoneticState = { text: "", runs: [] };
    compose(state, 0, "やまだ", "山田");
    // The IME reconverts the selected 山田 as 八幡 (やはた): the text is replaced
    edit(state, "");
    compose(state, 0, "やはた", "八幡");
    expect(validRuns(state.runs, state.text)).toEqual([
      { start: 0, end: 2, text: "ヤハタ" },
    ]);
  });
});

describe("with the model", () => {
  test("saves the readings and PHONETIC returns them", () => {
    const model = new Model("workbook", "ja", "UTC", "ja");
    const state: PhoneticState = { text: "", runs: [] };
    compose(state, 0, "おみおつけ", "御御御付");
    model.setUserInput(0, 1, 1, state.text);
    savePhoneticRuns(model, 0, 1, 1, state, state.text);
    model.setUserInput(0, 1, 2, "=PHONETIC(A1)");
    expect(model.getFormattedCellValue(0, 1, 1)).toBe("御御御付");
    expect(model.getFormattedCellValue(0, 1, 2)).toBe("オミオツケ");

    // Editing the cell again (F2) starts with its readings
    const runs = initialRuns(model, 0, 1, 1, "御御御付");
    expect(runs).toEqual([run(0, 4, "オミオツケ", "御御御付")]);
    // Not when the editor starts with other text (typing over the cell)
    expect(initialRuns(model, 0, 1, 1, "x")).toEqual([]);
  });

  test("numbers and formulas get no furigana", () => {
    const model = new Model("workbook", "ja", "UTC", "ja");
    const state: PhoneticState = {
      text: "123",
      runs: [run(0, 3, "イチニサン", "123")],
    };
    model.setUserInput(0, 1, 1, "123");
    expect(() =>
      savePhoneticRuns(model, 0, 1, 1, state, state.text),
    ).not.toThrow();
    expect(model.getCellPhoneticRuns(0, 1, 1)).toEqual([]);
  });
});
