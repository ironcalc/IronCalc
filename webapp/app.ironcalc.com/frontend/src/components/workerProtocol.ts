// Messages between the main thread (workerEngine.ts) and the evaluation worker
// (evaluationWorker.ts).

export type ToWorker =
  // Opens a workbook: the bytes of the model the main thread shows
  | { type: "open"; bytes: Uint8Array; language: string }
  // Changes made in the main thread, in order (`flushSendQueue`)
  | { type: "diffs"; diffs: Uint8Array; seq: number }
  // The language cell inputs are written in
  | { type: "language"; language: string }
  // Asks for the bytes of the workbook, with every diff sent so far applied
  | { type: "serialize"; id: number };

export type FromWorker =
  | { type: "opened" }
  // What evaluating every diff up to `seq` changed (`EvaluationTracker`)
  | { type: "results"; seq: number; results: Uint8Array }
  | { type: "serialized"; id: number; bytes: Uint8Array }
  // `id` is set when the error answers a `serialize` request. Any other error
  // means the worker can no longer be trusted to follow the workbook.
  | { type: "error"; id?: number; message: string };
