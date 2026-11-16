import type { Model } from "@ironcalc/workbook";
import type { FromWorker, ToWorker } from "./workerProtocol";

// How often the model is checked for changes to send to the worker, or for a
// slow evaluation. Both checks cost next to nothing.
const TICK_INTERVAL_MS = 25;

// An evaluation that takes longer than this blocks the UI noticeably: it is a
// long task (https://w3c.github.io/longtasks/). Workbooks that evaluate faster
// stay on the main thread, where they need no second copy of the workbook.
const SLOW_EVALUATION_MS = 50;

// The engine of each model shown, so that code that serializes a model can
// wait for its evaluation (see `whenEvaluated`).
const engines = new WeakMap<Model, WorkerEngine>();

/**
 * Resolves once every change made to `model` has been evaluated and its
 * results applied, so that `model.toBytes()` holds every computed value.
 * Resolves at once for a model that evaluates on the main thread.
 */
export function whenEvaluated(model: Model): Promise<void> {
  return engines.get(model)?.idle() ?? Promise.resolve();
}

function bytesEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) {
    return false;
  }
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) {
      return false;
    }
  }
  return true;
}

/**
 * Evaluates the workbook in a Web Worker when evaluating it would block the
 * main thread for too long (or serializing it, to save it).
 *
 * A workbook that evaluates quickly evaluates on the main thread, as it always
 * did. The first evaluation of a workbook runs in the worker, where it cannot
 * block the UI, to find out which kind it is. A workbook that evaluates slowly
 * stays in the worker; a workbook that grows slow while on the main thread
 * moves to the worker after its first slow evaluation.
 *
 * In the worker, the model the UI shows has its evaluation paused. Every
 * change made to it is sent to a second model in the worker, which evaluates
 * and sends back what changed (see `EvaluationTracker` in the engine). There
 * is at most one set of changes waiting for its results: changes made in the
 * meantime wait in the model's send queue and are sent when the results
 * arrive. Until then cells that depend on a change show their previous value.
 *
 * If workers are not available, or the worker fails, the model evaluates on
 * the main thread.
 */
export class WorkerEngine {
  private readonly model: Model;
  private readonly onResults: () => void;
  private worker: Worker | null = null;
  // Whether the worker is measuring the first evaluation of the workbook
  private probing = false;
  // Whether a worker failed to start or to follow the workbook
  private workerFailed = false;
  private timer: ReturnType<typeof setInterval> | null = null;
  // What an empty send queue encodes to
  private readonly emptyQueue: Uint8Array;
  // Sequence number of the last diffs sent, and whether their results are due
  private sent = 0;
  private waitingForResults = false;
  // The last diffs included in a successful save
  private saved = 0;
  private nextRequestId = 1;
  private readonly requests = new Map<
    number,
    {
      resolve: (bytes: Uint8Array | null) => void;
      reject: (error: Error) => void;
    }
  >();
  private idleWaiters: (() => void)[] = [];
  // The language the worker parses cell inputs in
  private language: string;

  constructor(model: Model, onResults: () => void) {
    this.model = model;
    this.onResults = onResults;
    this.language = model.getLanguage();
    // Whatever is queued is already in the model
    model.flushSendQueue();
    this.emptyQueue = model.flushSendQueue();
    engines.set(model, this);
    if (typeof Worker === "undefined") {
      return;
    }
    // The values in a file are those of its last evaluation, which may be
    // missing changes that were saved before their results came back: the
    // workbook is evaluated as it opens, and that evaluation is measured.
    if (this.startWorker(true)) {
      this.timer = setInterval(() => this.tick(), TICK_INTERVAL_MS);
    }
  }

  /** Resolves when every change made so far has its results applied. */
  idle(): Promise<void> {
    this.sync();
    if (!this.worker || !this.waitingForResults) {
      return Promise.resolve();
    }
    return new Promise((resolve) => this.idleWaiters.push(resolve));
  }

  /** Whether there are changes that were not saved yet. */
  hasUnsavedChanges(): boolean {
    if (!this.worker) {
      // Same check the app always did: anything in the queue is a change
      this.flushQueue();
    } else {
      this.sync();
    }
    return this.sent !== this.saved;
  }

  /**
   * The bytes of the workbook, to save it, with every change made so far. In
   * the worker they are made there. `null` if the engine is disposed before
   * they are ready.
   */
  serialize(): Promise<Uint8Array | null> {
    const seq = this.sent;
    const markSaved = (bytes: Uint8Array | null) => {
      if (bytes) {
        this.saved = Math.max(this.saved, seq);
      }
      return bytes;
    };
    if (!this.worker) {
      return Promise.resolve(markSaved(this.model.toBytes()));
    }
    const id = this.nextRequestId++;
    return new Promise<Uint8Array | null>((resolve, reject) => {
      this.requests.set(id, { resolve, reject });
      this.post({ type: "serialize", id });
    }).then(markSaved);
  }

  /** Stops the engine. The model evaluates again as it is edited. */
  dispose() {
    if (this.worker) {
      this.model.resumeEvaluation();
    }
    this.stopWorker();
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }
    if (engines.get(this.model) === this) {
      engines.delete(this.model);
    }
    // The workbook is closing: there is nothing left to save
    for (const request of this.requests.values()) {
      request.resolve(null);
    }
    this.requests.clear();
    this.wakeIdleWaiters();
  }

  // Opens the workbook in a new worker and evaluates there from now on.
  // `evaluate` asks the worker to evaluate the workbook as it opens and to
  // measure it.
  private startWorker(evaluate: boolean): boolean {
    let worker: Worker;
    try {
      worker = new Worker(new URL("./evaluationWorker.ts", import.meta.url), {
        type: "module",
      });
    } catch (error) {
      console.warn("Evaluating on the main thread:", error);
      this.workerFailed = true;
      return false;
    }
    worker.onmessage = (event: MessageEvent<FromWorker>) =>
      this.handle(event.data);
    worker.onerror = (event) => this.fallBack(event.message);
    this.worker = worker;
    // Whatever is queued is already in the model the worker opens
    this.flushQueue();
    this.model.pauseEvaluation();
    this.language = this.model.getLanguage();
    const bytes = this.model.toBytes();
    this.post({ type: "open", bytes, language: this.language, evaluate }, [
      bytes.buffer,
    ]);
    this.probing = evaluate;
    this.waitingForResults = evaluate;
    return true;
  }

  private stopWorker() {
    this.worker?.terminate();
    this.worker = null;
    this.probing = false;
    this.waitingForResults = false;
  }

  // Evaluates on the main thread from now on. The model holds the results of
  // every diff sent to the worker, unless `evaluateAll`.
  private leaveWorker(evaluateAll: boolean) {
    this.stopWorker();
    this.model.resumeEvaluation();
    if (this.flushQueue() || evaluateAll) {
      this.model.evaluate();
    }
    if (this.requests.size > 0) {
      const bytes = this.model.toBytes();
      for (const request of this.requests.values()) {
        request.resolve(bytes.slice());
      }
      this.requests.clear();
    }
    this.wakeIdleWaiters();
  }

  // Drops what is in the send queue, counting it as a change. Whether there
  // was anything.
  private flushQueue(): boolean {
    const queued = !bytesEqual(this.model.flushSendQueue(), this.emptyQueue);
    if (queued) {
      this.sent += 1;
    }
    return queued;
  }

  private wakeIdleWaiters() {
    const waiters = this.idleWaiters;
    this.idleWaiters = [];
    for (const wake of waiters) {
      wake();
    }
  }

  private post(message: ToWorker, transfer: Transferable[] = []) {
    this.worker?.postMessage(message, transfer);
  }

  private tick() {
    if (this.worker) {
      this.sync();
      return;
    }
    // On the main thread, every change is evaluated as it is made
    const duration = this.model.getLastEvaluationDuration() ?? 0;
    if (duration > SLOW_EVALUATION_MS && !this.workerFailed) {
      // The model is evaluated: the worker need not evaluate it again
      this.startWorker(false);
    }
  }

  // Sends the changes made since the last time, unless results are due
  private sync() {
    if (!this.worker || this.waitingForResults) {
      return;
    }
    const diffs = this.model.flushSendQueue();
    if (!bytesEqual(diffs, this.emptyQueue)) {
      this.sent += 1;
      this.waitingForResults = true;
      this.post({ type: "diffs", diffs, seq: this.sent }, [diffs.buffer]);
    }
    // The language is not part of the diffs, but the cell inputs in them are
    // written in it. The worker switches after applying the diffs above, which
    // were (almost always) made before the switch: it is checked often.
    const language = this.model.getLanguage();
    if (language !== this.language) {
      this.language = language;
      this.post({ type: "language", language });
    }
  }

  private handle(message: FromWorker) {
    switch (message.type) {
      case "opened":
        break;
      case "results": {
        try {
          this.model.applyEvaluationResults(message.results);
        } catch (error) {
          this.fallBack(String(error));
          return;
        }
        this.waitingForResults = false;
        if (this.probing) {
          this.probing = false;
          if (message.evaluationMs <= SLOW_EVALUATION_MS) {
            // Quick enough for the main thread
            this.leaveWorker(false);
            this.onResults();
            return;
          }
        }
        this.onResults();
        // Changes made while the worker was evaluating
        this.sync();
        if (!this.waitingForResults) {
          this.wakeIdleWaiters();
        }
        break;
      }
      case "serialized": {
        this.requests.get(message.id)?.resolve(message.bytes);
        this.requests.delete(message.id);
        break;
      }
      case "error": {
        if (message.id !== undefined) {
          this.requests.get(message.id)?.reject(new Error(message.message));
          this.requests.delete(message.id);
        } else {
          this.fallBack(message.message);
        }
        break;
      }
    }
  }

  // The worker can no longer follow the workbook: evaluate here from now on
  private fallBack(reason: string) {
    if (!this.worker) {
      return;
    }
    console.error(
      "Evaluation worker failed, evaluating on the main thread:",
      reason,
    );
    this.workerFailed = true;
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }
    // The worker may not have seen the last changes: they are saved from here
    this.sent += 1;
    this.leaveWorker(true);
    this.onResults();
  }
}
