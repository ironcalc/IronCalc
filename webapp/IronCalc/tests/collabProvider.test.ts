import { readFile } from "node:fs/promises";
import { initSync, Model } from "@ironcalc/wasm";
import { afterEach, beforeAll, expect, test, vi } from "vitest";
import {
  CollabProvider,
  type CollabStatus,
  type CollabWebSocket,
} from "../src/collab/CollabProvider";

// Two providers wired directly to each other through fake sockets: the
// y-sync protocol is symmetric, so this exercises the same frames a relay
// server shuttles (see bindings/wasm/tests/test_collab.mjs).

const WS_CONNECTING = 0;
const WS_OPEN = 1;
const WS_CLOSED = 3;

class FakeSocket implements CollabWebSocket {
  binaryType = "blob";
  readyState = WS_CONNECTING;
  onopen: (() => void) | null = null;
  onmessage: ((event: { data: ArrayBuffer }) => void) | null = null;
  onclose: (() => void) | null = null;
  onerror: (() => void) | null = null;
  peer: FakeSocket | null = null;
  // Frames sent to us before we opened (the relay buffers nothing, but the
  // peers never open at the very same instant in a test).
  private inbox: ArrayBuffer[] = [];

  static pair(): [FakeSocket, FakeSocket] {
    const a = new FakeSocket();
    const b = new FakeSocket();
    a.peer = b;
    b.peer = a;
    return [a, b];
  }

  open(): void {
    this.readyState = WS_OPEN;
    this.onopen?.();
    const pending = this.inbox;
    this.inbox = [];
    for (const data of pending) {
      this.onmessage?.({ data });
    }
  }

  send(data: Uint8Array): void {
    const peer = this.peer;
    if (!peer || this.readyState !== WS_OPEN) {
      return;
    }
    const copy = data.slice().buffer as ArrayBuffer;
    if (peer.readyState === WS_OPEN) {
      peer.onmessage?.({ data: copy });
    } else if (peer.readyState === WS_CONNECTING) {
      peer.inbox.push(copy);
    }
  }

  close(): void {
    if (this.readyState === WS_CLOSED) {
      return;
    }
    this.readyState = WS_CLOSED;
    this.onclose?.();
    this.peer?.close();
  }
}

function newModel(): Model {
  return new Model("workbook", "en", "UTC", "en");
}

// Manual flushes only: a huge interval keeps timers out of the tests.
const options = { flushIntervalMs: 3_600_000 };

const cleanups: (() => void)[] = [];

beforeAll(async () => {
  const buffer = await readFile("node_modules/@ironcalc/wasm/wasm_bg.wasm");
  initSync({ module: buffer });
});

afterEach(() => {
  for (const cleanup of cleanups.splice(0)) {
    cleanup();
  }
  vi.useRealTimers();
});

function connectedPair(): {
  a: Model;
  b: Model;
  pa: CollabProvider;
  pb: CollabProvider;
  sa: FakeSocket;
  sb: FakeSocket;
} {
  const a = newModel();
  const b = newModel();
  const [sa, sb] = FakeSocket.pair();
  const pa = new CollabProvider(a, "ws://fake/room", {
    ...options,
    clientId: 1,
    createWebSocket: () => sa,
  });
  const pb = new CollabProvider(b, "ws://fake/room", {
    ...options,
    clientId: 2,
    createWebSocket: () => sb,
  });
  cleanups.push(
    () => pa.destroy(),
    () => pb.destroy(),
  );
  pa.connect();
  pb.connect();
  sa.open();
  sb.open();
  return { a, b, pa, pb, sa, sb };
}

test("edits flow both ways and formulas evaluate", () => {
  const { a, b, pa, pb } = connectedPair();
  expect(pa.status).toBe("connected");

  let remoteUpdates = 0;
  pb.onRemoteUpdate(() => {
    remoteUpdates += 1;
  });

  a.setUserInput(0, 1, 1, "21");
  pa.flushNow();
  expect(b.getFormattedCellValue(0, 1, 1)).toBe("21");
  expect(remoteUpdates).toBe(1);

  b.setUserInput(0, 1, 2, "=A1*2");
  pb.flushNow();
  expect(a.getFormattedCellValue(0, 1, 2)).toBe("42");
});

test("unsubscribing a remote-update handler stops notifications", () => {
  const { a, pa, pb } = connectedPair();
  let calls = 0;
  const unsubscribe = pb.onRemoteUpdate(() => {
    calls += 1;
  });
  a.setUserInput(0, 1, 1, "1");
  pa.flushNow();
  unsubscribe();
  a.setUserInput(0, 1, 1, "2");
  pa.flushNow();
  expect(calls).toBe(1);
});

test("presence round trip and withdrawal", () => {
  const { pa, pb } = connectedPair();
  let presenceChanges = 0;
  pb.onPresenceChange(() => {
    presenceChanges += 1;
  });

  pa.setPresence({ name: "ana", cell: "A1" });
  expect(presenceChanges).toBe(1);
  expect(pb.presence()).toEqual([
    { clientId: 1, state: '{"name":"ana","cell":"A1"}' },
  ]);

  pa.clearPresence();
  expect(pb.presence()).toEqual([]);
});

test("destroy withdraws presence and closes the socket", () => {
  const { pa, pb, sa } = connectedPair();
  pa.setPresence({ name: "ana" });
  expect(pb.presence()).toHaveLength(1);

  pa.destroy();
  expect(pb.presence()).toEqual([]);
  expect(sa.readyState).toBe(WS_CLOSED);
  expect(pa.status).toBe("disconnected");
});

test("offline edits heal through the reconnect handshake", () => {
  vi.useFakeTimers();
  const a = newModel();
  const b = newModel();
  const firstPair = FakeSocket.pair();
  const socketsA = [firstPair[0]];
  const socketsB = [firstPair[1]];
  const pa = new CollabProvider(a, "ws://fake/room", {
    ...options,
    clientId: 1,
    createWebSocket: () => {
      const socket = socketsA.shift();
      if (!socket) {
        throw new Error("no socket available");
      }
      return socket;
    },
  });
  const pb = new CollabProvider(b, "ws://fake/room", {
    ...options,
    clientId: 2,
    createWebSocket: () => {
      const socket = socketsB.shift();
      if (!socket) {
        throw new Error("no socket available");
      }
      return socket;
    },
  });
  cleanups.push(
    () => pa.destroy(),
    () => pb.destroy(),
  );
  pa.connect();
  pb.connect();
  firstPair[0].open();
  firstPair[1].open();

  const statuses: CollabStatus[] = [];
  cleanups.push(pa.onStatusChange((status) => statuses.push(status)));

  // The connection drops; edits keep accumulating locally.
  firstPair[0].close();
  expect(pa.status).toBe("disconnected");
  a.setUserInput(0, 1, 1, "7");
  pa.flushNow();
  b.setUserInput(0, 2, 1, "35");
  pb.flushNow();
  expect(b.getFormattedCellValue(0, 1, 1)).toBe("");

  // Both providers reconnect after their backoff delay; the handshake
  // carries everything the other side missed.
  const secondPair = FakeSocket.pair();
  socketsA.push(secondPair[0]);
  socketsB.push(secondPair[1]);
  vi.advanceTimersByTime(500);
  secondPair[0].open();
  secondPair[1].open();

  expect(pa.status).toBe("connected");
  expect(statuses).toEqual(["disconnected", "connecting", "connected"]);
  expect(b.getFormattedCellValue(0, 1, 1)).toBe("7");
  expect(a.getFormattedCellValue(0, 2, 1)).toBe("35");
});
