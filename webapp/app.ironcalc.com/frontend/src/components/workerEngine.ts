import type { Model } from "@ironcalc/workbook";
import type { FromWorker, ToWorker } from "./workerProtocol";

// How often the model is checked for changes to send to the worker. The check
// is a flush of an (almost always) empty queue: it costs next to nothing.
const SYNC_INTERVAL_MS = 25;

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
 * Evaluates the workbook in a Web Worker, so that the main thread is never
 * blocked by a long evaluation (or by serializing the workbook to save it).
 *
 * The model the UI shows has its evaluation paused. Every change made to it is
 * sent to a second model in the worker, which evaluates and sends back what
 * changed (see `EvaluationTracker` in the engine). There is at most one set of
 * changes waiting for its results: changes made in the meantime wait in the
 * model's send queue and are sent when the results arrive. Until then cells
 * that depend on a change show their previous value.
 *
 * If workers are not available, or the worker fails, the model evaluates on
 * the main thread as before.
 */
export class WorkerEngine {
  private readonly model: Model;
  private readonly onResults: () => void;
  private worker: Worker | null = null;
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
    // Whatever is queued is already in the model the worker opens
    model.flushSendQueue();
    this.emptyQueue = model.flushSendQueue();
    if (typeof Worker === "undefined") {
      return;
    }
    try {
      this.worker = new Worker(
        new URL("./evaluationWorker.ts", import.meta.url),
        { type: "module" },
      );
    } catch (error) {
      console.warn("Evaluating on the main thread:", error);
      return;
    }
    this.worker.onmessage = (event: MessageEvent<FromWorker>) =>
      this.handle(event.data);
    this.worker.onerror = (event) => this.fallBack(event.message);
    model.pauseEvaluation();
    const bytes = model.toBytes();
    this.post({ type: "open", bytes, language: this.language }, [bytes.buffer]);
    // The worker evaluates the workbook it opens and sends the results
    this.waitingForResults = true;
    this.timer = setInterval(() => this.sync(), SYNC_INTERVAL_MS);
    engines.set(model, this);
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
      const queue = this.model.flushSendQueue();
      if (!bytesEqual(queue, this.emptyQueue)) {
        this.sent += 1;
      }
    } else {
      this.sync();
    }
    return this.sent !== this.saved;
  }

  /**
   * The bytes of the workbook, to save it. They are made in the worker and
   * include every change sent to it so far. `null` if the engine is disposed
   * before they are ready.
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

  /** Stops evaluating in the worker. The model evaluates again as it is edited. */
  dispose() {
    if (this.worker) {
      this.model.resumeEvaluation();
    }
    this.stop();
    // The workbook is closing: there is nothing left to save
    for (const request of this.requests.values()) {
      request.resolve(null);
    }
    this.requests.clear();
    this.wakeIdleWaiters();
  }

  private stop() {
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }
    this.worker?.terminate();
    this.worker = null;
    if (engines.get(this.model) === this) {
      engines.delete(this.model);
    }
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
    this.stop();
    this.model.resumeEvaluation();
    this.model.evaluate();
    // The worker may not have seen the last changes: they are saved from here
    this.sent += 1;
    const bytes = this.model.toBytes();
    for (const request of this.requests.values()) {
      request.resolve(bytes.slice());
    }
    this.requests.clear();
    this.wakeIdleWaiters();
    this.onResults();
  }
}
