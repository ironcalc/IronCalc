import type { Model } from "@ironcalc/workbook";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { WorkerEngine, whenEvaluated } from "./workerEngine";
import type { FromWorker, ToWorker } from "./workerProtocol";

const EMPTY_QUEUE = new Uint8Array([0]);

// The parts of a model the engine uses. Changes are counted, not made: the
// send queue holds how many there are since the last flush.
class FakeModel {
  queued = 0;
  paused = false;
  evaluations = 0;
  lastEvaluationDuration: number | undefined = undefined;
  appliedResults: Uint8Array[] = [];
  language = "en";

  edit() {
    this.queued += 1;
    if (!this.paused) {
      this.evaluate();
    }
  }
  flushSendQueue(): Uint8Array {
    const queue = this.queued === 0 ? EMPTY_QUEUE : new Uint8Array([1, 2]);
    this.queued = 0;
    return queue;
  }
  pauseEvaluation() {
    this.paused = true;
  }
  resumeEvaluation() {
    this.paused = false;
  }
  evaluate() {
    this.evaluations += 1;
  }
  getLastEvaluationDuration(): number | undefined {
    return this.lastEvaluationDuration;
  }
  getLanguage(): string {
    return this.language;
  }
  toBytes(): Uint8Array {
    return new Uint8Array([7, 7, 7]);
  }
  applyEvaluationResults(results: Uint8Array) {
    this.appliedResults.push(results);
  }
}

// Records what the engine posts and lets the test answer as the worker would
class FakeWorker {
  static instances: FakeWorker[] = [];
  posted: ToWorker[] = [];
  terminated = false;
  onmessage: ((event: MessageEvent<FromWorker>) => void) | null = null;
  onerror: ((event: ErrorEvent) => void) | null = null;

  constructor() {
    FakeWorker.instances.push(this);
  }
  postMessage(message: ToWorker) {
    this.posted.push(message);
  }
  terminate() {
    this.terminated = true;
  }
  reply(message: FromWorker) {
    this.onmessage?.({ data: message } as MessageEvent<FromWorker>);
  }
  results(seq: number, evaluationMs: number) {
    this.reply({
      type: "results",
      seq,
      results: new Uint8Array([seq]),
      evaluationMs,
    });
  }
}

function lastWorker(): FakeWorker {
  const worker = FakeWorker.instances.at(-1);
  if (!worker) {
    throw new Error("No worker was started");
  }
  return worker;
}

function setUp() {
  const model = new FakeModel();
  const onResults = vi.fn();
  const engine = new WorkerEngine(model as unknown as Model, onResults);
  return { model, engine, onResults };
}

// Lets the engine's 25 ms tick run
function tick() {
  vi.advanceTimersByTime(25);
}

beforeEach(() => {
  vi.useFakeTimers();
  FakeWorker.instances = [];
  vi.stubGlobal("Worker", FakeWorker);
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

describe("WorkerEngine", () => {
  it("measures the first evaluation in the worker", () => {
    const { model, engine } = setUp();
    const worker = lastWorker();
    expect(model.paused).toBe(true);
    expect(worker.posted).toEqual([
      { type: "open", bytes: model.toBytes(), language: "en", evaluate: true },
    ]);
    engine.dispose();
  });

  it("evaluates a quick workbook on the main thread", async () => {
    const { model, engine, onResults } = setUp();
    const worker = lastWorker();
    worker.results(0, 10);
    expect(worker.terminated).toBe(true);
    expect(model.paused).toBe(false);
    expect(model.appliedResults).toHaveLength(1);
    expect(onResults).toHaveBeenCalledTimes(1);
    // Nothing was waiting: nothing to evaluate here
    expect(model.evaluations).toBe(0);
    await expect(whenEvaluated(model as unknown as Model)).resolves.toBe(
      undefined,
    );
    // Edits evaluate as they are made, and are saved from here
    model.edit();
    expect(model.evaluations).toBe(1);
    expect(engine.hasUnsavedChanges()).toBe(true);
    await expect(engine.serialize()).resolves.toEqual(model.toBytes());
    expect(engine.hasUnsavedChanges()).toBe(false);
    engine.dispose();
  });

  it("evaluates what was edited while measuring", async () => {
    const { model, engine } = setUp();
    const worker = lastWorker();
    model.edit();
    const bytes = engine.serialize();
    worker.results(0, 10);
    expect(model.evaluations).toBe(1);
    expect(model.queued).toBe(0);
    // The worker was gone before it could answer
    await expect(bytes).resolves.toEqual(model.toBytes());
    engine.dispose();
  });

  it("keeps a slow workbook in the worker", async () => {
    const { model, engine } = setUp();
    const worker = lastWorker();
    const evaluated = whenEvaluated(model as unknown as Model);
    worker.results(0, 200);
    await evaluated;
    expect(worker.terminated).toBe(false);
    expect(model.paused).toBe(true);

    model.edit();
    expect(model.evaluations).toBe(0);
    tick();
    expect(worker.posted.at(-1)).toMatchObject({ type: "diffs", seq: 1 });
    // A fast evaluation later on does not move it back
    worker.results(1, 1);
    expect(worker.terminated).toBe(false);
    expect(model.paused).toBe(true);
    engine.dispose();
    expect(worker.terminated).toBe(true);
    expect(model.paused).toBe(false);
  });

  it("moves to the worker after a slow evaluation on the main thread", () => {
    const { model, engine } = setUp();
    lastWorker().results(0, 10);
    model.edit();
    model.lastEvaluationDuration = 30;
    tick();
    expect(FakeWorker.instances).toHaveLength(1);

    model.lastEvaluationDuration = 120;
    model.edit();
    tick();
    expect(FakeWorker.instances).toHaveLength(2);
    const worker = lastWorker();
    expect(model.paused).toBe(true);
    // The model is evaluated already
    expect(worker.posted).toEqual([
      { type: "open", bytes: model.toBytes(), language: "en", evaluate: false },
    ]);
    // The edits before the move were not saved yet
    expect(engine.hasUnsavedChanges()).toBe(true);

    model.edit();
    tick();
    expect(worker.posted.at(-1)).toMatchObject({ type: "diffs" });
    engine.dispose();
  });

  it("stays on the main thread after the worker fails", () => {
    const { model, engine, onResults } = setUp();
    const worker = lastWorker();
    worker.results(0, 200);
    model.edit();
    worker.reply({ type: "error", message: "boom" });
    expect(worker.terminated).toBe(true);
    expect(model.paused).toBe(false);
    expect(model.evaluations).toBe(1);
    expect(onResults).toHaveBeenCalledTimes(2);

    model.lastEvaluationDuration = 500;
    tick();
    expect(FakeWorker.instances).toHaveLength(1);
    engine.dispose();
  });

  it("evaluates on the main thread without workers", async () => {
    vi.stubGlobal("Worker", undefined);
    const { model, engine } = setUp();
    expect(model.paused).toBe(false);
    model.lastEvaluationDuration = 500;
    model.edit();
    tick();
    expect(model.paused).toBe(false);
    expect(engine.hasUnsavedChanges()).toBe(true);
    await expect(engine.serialize()).resolves.toEqual(model.toBytes());
    engine.dispose();
  });

  it("gives up pending saves when disposed", async () => {
    const { model, engine } = setUp();
    lastWorker().results(0, 200);
    model.edit();
    const bytes = engine.serialize();
    engine.dispose();
    await expect(bytes).resolves.toBeNull();
  });
});
