// Furigana (readings) recorded from the IME while a cell is edited, as Japanese Excel does.
//
// When a text is converted with the IME (やまだ => 山田) we remember the reading of the
// converted characters as a run: characters start..end of the text read as ヤマダ. Runs use
// UTF-16 positions, like JavaScript string indexes and xlsx files.
//
// The text can be edited character by character afterwards, so after every change we work
// out which part of the text changed: runs before it stay, runs after it move, and runs
// that touch it are dropped. A missing reading is better than a wrong one (PHONETIC then
// returns the text itself). When the cell is saved a run is only kept if its characters are
// still the ones it was recorded for.
import type { Model, PhoneticRun } from "@ironcalc/wasm";

// A run and the characters it was recorded for
export interface TrackedRun extends PhoneticRun {
  base: string;
}

// The runs of the text being edited
export interface PhoneticState {
  text: string;
  runs: TrackedRun[];
}

// Hiragana, katakana and the prolonged sound mark
const KANA = /^[ぁ-ゖゝゞァ-ヺー-ヾ]+$/;

export function isKana(text: string): boolean {
  return KANA.test(text);
}

// Furigana are shown in full-width katakana, the default in Excel
export function toKatakana(text: string): string {
  return text.replace(/[ぁ-ゖゝゞ]/g, (c) =>
    String.fromCharCode(c.charCodeAt(0) + 0x60),
  );
}

// Moves the runs after `before` is edited into `after`. The edit is the part between the
// common start and the common end of both texts.
export function shiftRuns(
  runs: TrackedRun[],
  before: string,
  after: string,
): TrackedRun[] {
  if (before === after) {
    return runs;
  }
  const maxCommon = Math.min(before.length, after.length);
  let prefix = 0;
  while (prefix < maxCommon && before[prefix] === after[prefix]) {
    prefix += 1;
  }
  let suffix = 0;
  while (
    suffix < maxCommon - prefix &&
    before[before.length - 1 - suffix] === after[after.length - 1 - suffix]
  ) {
    suffix += 1;
  }
  const editEnd = before.length - suffix;
  const delta = after.length - before.length;
  const result: TrackedRun[] = [];
  for (const run of runs) {
    if (run.end <= prefix) {
      result.push(run);
    } else if (run.start >= editEnd) {
      result.push({ ...run, start: run.start + delta, end: run.end + delta });
    }
    // Otherwise the edit touches the run and its reading is no longer right
  }
  return result;
}

// The runs whose characters are still the ones they were recorded for
export function validRuns(runs: TrackedRun[], text: string): PhoneticRun[] {
  return runs
    .filter((run) => text.slice(run.start, run.end) === run.base)
    .sort((a, b) => a.start - b.start)
    .map(({ start, end, text }) => ({ start, end, text }));
}

// The reading of an IME composition, if it can be recorded:
// * `committed` is the text the composition ended with (山田)
// * `kana` is the last kana-only text seen while composing (やまだ)
// * `last` is the last text seen while composing
// If the IME committed only part of the composition (the first clause), `last` is not the
// committed text and we cannot tell which part of the kana belongs to it.
export function compositionReading(
  committed: string,
  kana: string,
  last: string,
): string | null {
  if (committed === "" || kana === "" || isKana(committed)) {
    return null;
  }
  if (last !== committed) {
    return null;
  }
  return toKatakana(kana);
}

// Adds the run of a composition that ended at `start` with `committed` read as `reading`
export function addRun(
  state: PhoneticState,
  start: number,
  committed: string,
  reading: string,
): void {
  const end = start + committed.length;
  if (state.text.slice(start, end) !== committed) {
    return;
  }
  const runs = state.runs.filter((run) => run.end <= start || run.start >= end);
  runs.push({ start, end, text: reading, base: committed });
  runs.sort((a, b) => a.start - b.start);
  state.runs = runs;
}

// The runs of the text in a cell when an edit starts, if the editor starts with that text
export function initialRuns(
  model: Model,
  sheet: number,
  row: number,
  column: number,
  text: string,
): TrackedRun[] {
  if (text === "" || model.getCellContent(sheet, row, column) !== text) {
    return [];
  }
  return model
    .getCellPhoneticRuns(sheet, row, column)
    .map((run) => ({ ...run, base: text.slice(run.start, run.end) }));
}

function sameRuns(a: PhoneticRun[], b: PhoneticRun[]): boolean {
  return (
    a.length === b.length &&
    a.every(
      (run, i) =>
        run.start === b[i].start &&
        run.end === b[i].end &&
        run.text === b[i].text,
    )
  );
}

// Saves the furigana of a cell after its input was set
export function savePhoneticRuns(
  model: Model,
  sheet: number,
  row: number,
  column: number,
  state: PhoneticState | undefined,
  text: string,
): void {
  if (!state || text.startsWith("=")) {
    return;
  }
  const runs = validRuns(state.runs, text);
  try {
    if (!sameRuns(runs, model.getCellPhoneticRuns(sheet, row, column))) {
      model.setCellPhoneticRuns(sheet, row, column, runs);
    }
  } catch {
    // The cell is not a text (a number, a date, a boolean): it has no furigana
  }
}
