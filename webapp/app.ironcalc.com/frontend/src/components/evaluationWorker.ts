/// <reference lib="webworker" />
// Evaluates the workbook off the main thread.
//
// The worker holds a second model of the open workbook. The main thread sends
// it the diffs of every change (`flushSendQueue`); the worker applies them,
// evaluates, and sends back what the evaluation changed (`EvaluationTracker`),
// which the main thread applies to its own, paused, model. The worker also
// serializes the workbook for the autosave, so that the main thread never has
// to (see workerEngine.ts).
import init, { EvaluationTracker, Model } from "@ironcalc/wasm";
import type { FromWorker, ToWorker } from "./workerProtocol";

declare const self: DedicatedWorkerGlobalScope;

const ready = init();

let model: Model | null = null;
// Diffs received and not yet applied, and the sequence number of the last one
let pending: Uint8Array[] = [];
let lastSeq = 0;
let scheduled = false;

function post(message: FromWorker, transfer: Transferable[] = []) {
  self.postMessage(message, transfer);
}

// Evaluates the model and sends what changed. The model must hold what the
// main thread holds: what the evaluation changes is measured from there (see
// `EvaluationTracker` in the engine).
function evaluateAndSend(model: Model) {
  const tracker = new EvaluationTracker(model);
  try {
    model.evaluate();
    const results = tracker.takeResults(model);
    post({ type: "results", seq: lastSeq, results }, [results.buffer]);
  } finally {
    tracker.free();
  }
}

// Applies every pending diff and evaluates once.
function evaluatePending() {
  scheduled = false;
  if (!model || pending.length === 0) {
    return;
  }
  // Applied without evaluating, so that the model holds what the main thread
  // holds
  model.pauseEvaluation();
  try {
    for (const diffs of pending) {
      model.applyExternalDiffs(diffs);
    }
  } finally {
    pending = [];
    model.resumeEvaluation();
  }
  evaluateAndSend(model);
}

function schedule() {
  if (!scheduled) {
    scheduled = true;
    // Lets the diffs already queued for the worker arrive first
    setTimeout(() => {
      try {
        evaluatePending();
      } catch (error) {
        post({ type: "error", message: String(error) });
      }
    }, 0);
  }
}

async function handle(message: ToWorker) {
  await ready;
  switch (message.type) {
    case "open": {
      model?.free();
      model = Model.fromBytes(message.bytes, message.language);
      pending = [];
      lastSeq = 0;
      post({ type: "opened" });
      // The values in a file are those of its last evaluation, which may be
      // missing changes that were saved before their results came back
      evaluateAndSend(model);
      break;
    }
    case "diffs": {
      pending.push(message.diffs);
      lastSeq = message.seq;
      schedule();
      break;
    }
    case "language": {
      // Not a diff: cell inputs in the diffs are parsed in this language
      evaluatePending();
      model?.setLanguage(message.language);
      break;
    }
    case "serialize": {
      evaluatePending();
      if (!model) {
        post({ type: "error", id: message.id, message: "No workbook open" });
        return;
      }
      const bytes = model.toBytes();
      post({ type: "serialized", id: message.id, bytes }, [bytes.buffer]);
      break;
    }
  }
}

// Messages are handled one at a time, in order, even across the `await` above
let queue: Promise<void> = Promise.resolve();
self.onmessage = (event: MessageEvent<ToWorker>) => {
  const message = event.data;
  queue = queue
    .then(() => handle(message))
    .catch((error: unknown) => {
      const id = message.type === "serialize" ? message.id : undefined;
      post({ type: "error", id, message: String(error) });
    });
};
