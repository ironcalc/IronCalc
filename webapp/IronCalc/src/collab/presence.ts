import type { CollabPresence } from "@ironcalc/wasm";

// Collaborative presence (phase 10.2): the JSON each client publishes about
// itself and the decoding of other clients' states into remote cursors.
// The transport treats these as opaque strings; this module is the only
// place that knows their shape.

export interface CollabCursor {
  clientId: number;
  name: string;
  /** Index of the sheet the collaborator is on. */
  sheet: number;
  /** The active cell. */
  row: number;
  column: number;
  /** The selected range: [rowStart, columnStart, rowEnd, columnEnd]. */
  range: [number, number, number, number];
}

// A collaborator's color is derived from its client id, so every replica
// picks the same one without transmitting it.
const CURSOR_COLORS = [
  "#E91E63",
  "#7B1FA2",
  "#3F51B5",
  "#0288D1",
  "#00796B",
  "#43A047",
  "#EF6C00",
  "#6D4C41",
];

export function colorForClient(clientId: number): string {
  return CURSOR_COLORS[Math.abs(clientId) % CURSOR_COLORS.length];
}

function isValidIndex(value: unknown): value is number {
  return typeof value === "number" && Number.isInteger(value) && value >= 0;
}

/** Decodes one presence state; `null` when it is not a cursor we understand. */
export function decodeCursor(
  clientId: number,
  state: string,
): CollabCursor | null {
  let parsed: unknown;
  try {
    parsed = JSON.parse(state);
  } catch {
    return null;
  }
  if (typeof parsed !== "object" || parsed === null) {
    return null;
  }
  const { name, sheet, row, column, range } = parsed as Record<string, unknown>;
  if (typeof name !== "string") {
    return null;
  }
  if (!(isValidIndex(sheet) && isValidIndex(row) && isValidIndex(column))) {
    return null;
  }
  if (!(Array.isArray(range) && range.length === 4)) {
    return null;
  }
  if (!range.every(isValidIndex)) {
    return null;
  }
  return {
    clientId,
    name,
    sheet,
    row,
    column,
    range: range as [number, number, number, number],
  };
}

/** The other collaborators' cursors (our own client id is filtered out). */
export function decodeCursors(
  presence: CollabPresence[],
  ownClientId: number,
): CollabCursor[] {
  const cursors: CollabCursor[] = [];
  for (const entry of presence) {
    if (entry.clientId === ownClientId) {
      continue;
    }
    const cursor = decodeCursor(entry.clientId, entry.state);
    if (cursor) {
      cursors.push(cursor);
    }
  }
  return cursors;
}
