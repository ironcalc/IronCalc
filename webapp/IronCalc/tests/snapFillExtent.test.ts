import type { MergedCell } from "@ironcalc/wasm";
import { expect, test } from "vitest";
import { snapFillExtent } from "../src/components/util";

const merged = (
  row: number,
  column: number,
  width: number,
  height: number,
): MergedCell => ({ row, column, width, height });

test("a selection without merges keeps the dragged extent", () => {
  const selection = { rowStart: 1, rowEnd: 2, columnStart: 1, columnEnd: 1 };
  expect(snapFillExtent([], selection, "rowsDown", 3)).toBe(3);
  expect(snapFillExtent([], selection, "rowsUp", 1)).toBe(1);
});

test("merges outside the selection are ignored", () => {
  const selection = { rowStart: 1, rowEnd: 1, columnStart: 1, columnEnd: 1 };
  expect(snapFillExtent([merged(5, 5, 2, 2)], selection, "rowsDown", 3)).toBe(
    3,
  );
});

test("a merged selection snaps down to whole tiles", () => {
  // the selection is a single 2-tall merge A1:A2
  const selection = { rowStart: 1, rowEnd: 2, columnStart: 1, columnEnd: 1 };
  const merges = [merged(1, 1, 1, 2)];
  expect(snapFillExtent(merges, selection, "rowsDown", 4)).toBe(4);
  expect(snapFillExtent(merges, selection, "rowsDown", 3)).toBe(2);
  expect(snapFillExtent(merges, selection, "rowsDown", 1)).toBe(0);
  expect(snapFillExtent(merges, selection, "rowsUp", 3)).toBe(2);
});

test("a partial tile is kept when its merges fit whole", () => {
  // pattern: A1:A2 merged plus the plain row 3
  const selection = { rowStart: 1, rowEnd: 3, columnStart: 1, columnEnd: 1 };
  const merges = [merged(1, 1, 1, 2)];
  // filling down, a partial tile of two rows holds the whole merge...
  expect(snapFillExtent(merges, selection, "rowsDown", 5)).toBe(5);
  // ...but one of a single row would cut it
  expect(snapFillExtent(merges, selection, "rowsDown", 4)).toBe(3);
  expect(snapFillExtent(merges, selection, "rowsDown", 1)).toBe(0);
  // filling up the tiles flip: the partial tile holds the plain row first
  expect(snapFillExtent(merges, selection, "rowsUp", 4)).toBe(4);
  expect(snapFillExtent(merges, selection, "rowsUp", 2)).toBe(1);
});

test("stacked merges snap independently", () => {
  // two 2-tall merges, A1:A2 and A3:A4
  const selection = { rowStart: 1, rowEnd: 4, columnStart: 1, columnEnd: 1 };
  const merges = [merged(1, 1, 1, 2), merged(3, 1, 1, 2)];
  expect(snapFillExtent(merges, selection, "rowsDown", 6)).toBe(6);
  expect(snapFillExtent(merges, selection, "rowsDown", 5)).toBe(4);
  expect(snapFillExtent(merges, selection, "rowsDown", 3)).toBe(2);
});

test("column fills snap on the merge width", () => {
  // the selection is a single 2-wide merge B1:C1
  const selection = { rowStart: 1, rowEnd: 1, columnStart: 2, columnEnd: 3 };
  const merges = [merged(1, 2, 2, 1)];
  expect(snapFillExtent(merges, selection, "columnsRight", 4)).toBe(4);
  expect(snapFillExtent(merges, selection, "columnsRight", 3)).toBe(2);
  expect(snapFillExtent(merges, selection, "columnsLeft", 1)).toBe(0);
  expect(snapFillExtent(merges, selection, "columnsLeft", 2)).toBe(2);
});

test("a vertical merge does not restrict a column fill", () => {
  // A1:A2 is one column wide: every column extent tiles it whole
  const selection = { rowStart: 1, rowEnd: 2, columnStart: 1, columnEnd: 1 };
  const merges = [merged(1, 1, 1, 2)];
  expect(snapFillExtent(merges, selection, "columnsRight", 3)).toBe(3);
  expect(snapFillExtent(merges, selection, "columnsLeft", 1)).toBe(1);
});
