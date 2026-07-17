import { expect, test } from "vitest";
import {
  colorForClient,
  decodeCursor,
  decodeCursors,
} from "../src/collab/presence";

const state = (overrides: Record<string, unknown> = {}) =>
  JSON.stringify({
    name: "ana",
    sheet: 0,
    row: 3,
    column: 2,
    range: [3, 2, 5, 4],
    ...overrides,
  });

test("decodes a well-formed cursor", () => {
  expect(decodeCursor(7, state())).toEqual({
    clientId: 7,
    name: "ana",
    sheet: 0,
    row: 3,
    column: 2,
    range: [3, 2, 5, 4],
  });
});

test("rejects malformed states", () => {
  expect(decodeCursor(7, "not json")).toBeNull();
  expect(decodeCursor(7, '"a string"')).toBeNull();
  expect(decodeCursor(7, "null")).toBeNull();
  expect(decodeCursor(7, state({ name: 42 }))).toBeNull();
  expect(decodeCursor(7, state({ row: -1 }))).toBeNull();
  expect(decodeCursor(7, state({ row: 1.5 }))).toBeNull();
  expect(decodeCursor(7, state({ sheet: "0" }))).toBeNull();
  expect(decodeCursor(7, state({ range: [1, 2, 3] }))).toBeNull();
  expect(decodeCursor(7, state({ range: [1, 2, 3, "4"] }))).toBeNull();
});

test("decodeCursors filters own client and undecodable states", () => {
  const cursors = decodeCursors(
    [
      { clientId: 1, state: state() },
      { clientId: 2, state: "garbage" },
      { clientId: 3, state: state({ name: "bob" }) },
    ],
    1,
  );
  expect(cursors.map((c) => [c.clientId, c.name])).toEqual([[3, "bob"]]);
});

test("colors are stable per client and spread over the palette", () => {
  expect(colorForClient(5)).toBe(colorForClient(5));
  const colors = new Set(
    Array.from({ length: 8 }, (_, i) => colorForClient(i)),
  );
  expect(colors.size).toBe(8);
});
