export type ToWorker =
  | { type: "open"; bytes: Uint8Array; language: string; evaluate: boolean }
  | { type: "diffs"; diffs: Uint8Array; seq: number }
  | { type: "language"; language: string }
  | { type: "serialize"; id: number };

export type FromWorker =
  | { type: "opened" }
  | { type: "results"; seq: number; results: Uint8Array; evaluationMs: number }
  | { type: "serialized"; id: number; bytes: Uint8Array }
  | { type: "error"; message: string };
