/// <reference lib="webworker" />
import init, { EvaluationTracker, Model } from "@ironcalc/wasm";
import type { FromWorker, ToWorker } from "./workerProtocol";

declare const self: DedicatedWorkerGlobalScope;

const ready = init();

let model: Model | null = null;
let pendingDiffs: Uint8Array[] = [];
let lastSeq = 0;
let evaluationScheduled = false;

function post(message: FromWorker, transfer: Transferable[] = []) {
  self.postMessage(message, transfer);
}

function fail(error: unknown) {
  post({ type: "error", message: String(error) });
}

function evaluateAndSend(model: Model) {
  const tracker = new EvaluationTracker(model);
  model.evaluate();
  const results = tracker.takeResults(model);
  const evaluationMs = model.getLastEvaluationDuration() ?? 0;
  post({ type: "results", seq: lastSeq, results, evaluationMs }, [
    results.buffer,
  ]);
}

function evaluatePending() {
  evaluationScheduled = false;
  if (!model || pendingDiffs.length === 0) {
    return;
  }
  model.pauseEvaluation();
  try {
    for (const diffs of pendingDiffs) {
      model.applyExternalDiffs(diffs);
    }
  } finally {
    pendingDiffs = [];
    model.resumeEvaluation();
  }
  evaluateAndSend(model);
}

function scheduleEvaluation() {
  if (!evaluationScheduled) {
    evaluationScheduled = true;
    setTimeout(() => {
      try {
        evaluatePending();
      } catch (error) {
        fail(error);
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
      pendingDiffs = [];
      lastSeq = 0;
      post({ type: "opened" });
      if (message.evaluate) {
        evaluateAndSend(model);
      }
      break;
    }
    case "diffs": {
      pendingDiffs.push(message.diffs);
      lastSeq = message.seq;
      scheduleEvaluation();
      break;
    }
    case "language": {
      // The pending diffs were typed in the previous language
      evaluatePending();
      model?.setLanguage(message.language);
      break;
    }
    case "serialize": {
      evaluatePending();
      if (!model) {
        throw new Error("No workbook open");
      }
      const bytes = model.toBytes();
      post({ type: "serialized", id: message.id, bytes }, [bytes.buffer]);
      break;
    }
  }
}

self.onmessage = (event: MessageEvent<ToWorker>) => {
  handle(event.data).catch(fail);
};
