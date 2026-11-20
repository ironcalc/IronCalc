import type { Model } from "@ironcalc/workbook";
import type { FromWorker, ToWorker } from "./workerProtocol";

const TICK_INTERVAL_MS = 25;

// Longer than this is a long task: https://w3c.github.io/longtasks/
const SLOW_EVALUATION_MS = 50;

const engines = new WeakMap<Model, WorkerEngine>();

/**
 * Resolves once `model.toBytes()` includes the results of every change made so
 * far.
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
 * Evaluates the workbook in a Web Worker while evaluating it would block the
 * main thread, and serializes it there too.
 *
 * The workbook is evaluated in the worker as it opens, to time it. One that
 * evaluates quickly goes back to evaluating on the main thread, until an
 * evaluation there is slow. One that is slow stays in the worker.
 *
 * While in the worker, the model shown has its evaluation paused and sends its
 * changes to the worker one batch at a time (see `EvaluationTracker`). Until
 * the results of a batch arrive, the cells that depend on it show their
 * previous values.
 */
export class WorkerEngine {
  private readonly model: Model;
  private readonly onResults: () => void;
  private worker: Worker | null = null;
  private timingFirstEvaluation = false;
  private workerFailed = false;
  private timer: ReturnType<typeof setInterval> | null = null;
  private readonly emptyQueue: Uint8Array;
  private changeCount = 0;
  private savedChangeCount = 0;
  private waitingForResults = false;
  private nextRequestId = 1;
  private readonly requests = new Map<
    number,
    (bytes: Uint8Array | null) => void
  >();
  private idleWaiters: (() => void)[] = [];
  private workerLanguage: string;

  constructor(model: Model, onResults: () => void) {
    this.model = model;
    this.onResults = onResults;
    this.workerLanguage = model.getLanguage();
    model.flushSendQueue();
    this.emptyQueue = model.flushSendQueue();
    engines.set(model, this);
    if (typeof Worker !== "undefined" && this.startWorker({ evaluate: true })) {
      this.timer = setInterval(() => this.tick(), TICK_INTERVAL_MS);
    }
  }

  idle(): Promise<void> {
    this.sync();
    if (!this.worker || !this.waitingForResults) {
      return Promise.resolve();
    }
    return new Promise((resolve) => this.idleWaiters.push(resolve));
  }

  hasUnsavedChanges(): boolean {
    if (!this.worker) {
      this.flushQueue();
    } else {
      this.sync();
    }
    return this.changeCount !== this.savedChangeCount;
  }

  /**
   * The bytes of the workbook with every change made so far, or `null` if the
   * engine is disposed first.
   */
  serialize(): Promise<Uint8Array | null> {
    const changeCount = this.changeCount;
    const markSaved = (bytes: Uint8Array | null) => {
      if (bytes) {
        this.savedChangeCount = Math.max(this.savedChangeCount, changeCount);
      }
      return bytes;
    };
    if (!this.worker) {
      return Promise.resolve(markSaved(this.model.toBytes()));
    }
    const id = this.nextRequestId++;
    return new Promise<Uint8Array | null>((resolve) => {
      this.requests.set(id, resolve);
      this.post({ type: "serialize", id });
    }).then(markSaved);
  }

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
    for (const resolve of this.requests.values()) {
      resolve(null);
    }
    this.requests.clear();
    this.wakeIdleWaiters();
  }

  private startWorker({ evaluate }: { evaluate: boolean }): boolean {
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
    // Already in the bytes the worker opens
    this.flushQueue();
    this.model.pauseEvaluation();
    this.workerLanguage = this.model.getLanguage();
    const bytes = this.model.toBytes();
    this.post(
      { type: "open", bytes, language: this.workerLanguage, evaluate },
      [bytes.buffer],
    );
    this.timingFirstEvaluation = evaluate;
    this.waitingForResults = evaluate;
    return true;
  }

  private stopWorker() {
    this.worker?.terminate();
    this.worker = null;
    this.timingFirstEvaluation = false;
    this.waitingForResults = false;
  }

  private leaveWorker({ missingResults }: { missingResults: boolean }) {
    this.stopWorker();
    this.model.resumeEvaluation();
    if (this.flushQueue() || missingResults) {
      this.model.evaluate();
    }
    if (this.requests.size > 0) {
      const bytes = this.model.toBytes();
      for (const resolve of this.requests.values()) {
        resolve(bytes.slice());
      }
      this.requests.clear();
    }
    this.wakeIdleWaiters();
  }

  private flushQueue(): boolean {
    const queued = !bytesEqual(this.model.flushSendQueue(), this.emptyQueue);
    if (queued) {
      this.changeCount += 1;
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
    const duration = this.model.getLastEvaluationDuration() ?? 0;
    if (duration > SLOW_EVALUATION_MS && !this.workerFailed) {
      this.startWorker({ evaluate: false });
    }
  }

  private sync() {
    if (!this.worker || this.waitingForResults) {
      return;
    }
    const diffs = this.model.flushSendQueue();
    if (!bytesEqual(diffs, this.emptyQueue)) {
      this.changeCount += 1;
      this.waitingForResults = true;
      this.post({ type: "diffs", diffs, seq: this.changeCount }, [
        diffs.buffer,
      ]);
    }
    // Not in the diffs: a batch with changes from both before and after a
    // switch is applied in the previous language
    const language = this.model.getLanguage();
    if (language !== this.workerLanguage) {
      this.workerLanguage = language;
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
        if (this.timingFirstEvaluation) {
          this.timingFirstEvaluation = false;
          if (message.evaluationMs <= SLOW_EVALUATION_MS) {
            this.leaveWorker({ missingResults: false });
            this.onResults();
            return;
          }
        }
        this.onResults();
        this.sync();
        if (!this.waitingForResults) {
          this.wakeIdleWaiters();
        }
        break;
      }
      case "serialized": {
        this.requests.get(message.id)?.(message.bytes);
        this.requests.delete(message.id);
        break;
      }
      case "error": {
        this.fallBack(message.message);
        break;
      }
    }
  }

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
    // Saves the main thread's model too, in case the worker's last save is wrong
    this.changeCount += 1;
    this.leaveWorker({ missingResults: true });
    this.onResults();
  }
}
