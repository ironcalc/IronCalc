import {
  afterAll,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from "vitest";
import type { FromWorker, ToWorker } from "./workerProtocol";

const wasm = vi.hoisted(() => {
  const state = { failEvaluation: false };
  class FakeModel {
    pauseEvaluation() {}
    resumeEvaluation() {}
    applyExternalDiffs() {}
    evaluate() {
      if (state.failEvaluation) {
        throw new Error("evaluation failed");
      }
    }
    getLastEvaluationDuration() {
      return 0;
    }
    setLanguage() {}
    toBytes() {
      return new Uint8Array([7]);
    }
    free() {}
  }
  class FakeTracker {
    takeResults() {
      return new Uint8Array([1]);
    }
  }
  return {
    state,
    module: {
      default: () => Promise.resolve(),
      Model: { fromBytes: () => new FakeModel() },
      EvaluationTracker: FakeTracker,
    },
  };
});

vi.mock("@ironcalc/wasm", () => wasm.module);

const posted: FromWorker[] = [];
const scope = {
  onmessage: null as ((event: MessageEvent<ToWorker>) => void) | null,
  postMessage: (message: FromWorker) => posted.push(message),
};

function send(message: ToWorker) {
  scope.onmessage?.({ data: message } as MessageEvent<ToWorker>);
}

function settle(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

function open() {
  send({
    type: "open",
    bytes: new Uint8Array([0]),
    language: "en",
    evaluate: false,
  });
}

beforeAll(async () => {
  vi.stubGlobal("self", scope);
  await import("./evaluationWorker");
});

afterAll(() => {
  vi.unstubAllGlobals();
});

beforeEach(() => {
  posted.length = 0;
  wasm.state.failEvaluation = false;
});

describe("evaluationWorker", () => {
  it("evaluates the pending diffs before serializing", async () => {
    open();
    send({ type: "diffs", diffs: new Uint8Array([1]), seq: 1 });
    send({ type: "serialize", id: 2 });
    await settle();
    expect(posted).toEqual([
      { type: "opened" },
      {
        type: "results",
        seq: 1,
        results: new Uint8Array([1]),
        evaluationMs: 0,
      },
      { type: "serialized", id: 2, bytes: new Uint8Array([7]) },
    ]);
  });

  it("fails as a whole when evaluating for a save fails", async () => {
    open();
    send({ type: "diffs", diffs: new Uint8Array([1]), seq: 1 });
    wasm.state.failEvaluation = true;
    send({ type: "serialize", id: 2 });
    await settle();
    expect(posted).toEqual([
      { type: "opened" },
      { type: "error", message: "Error: evaluation failed" },
    ]);
  });
});
