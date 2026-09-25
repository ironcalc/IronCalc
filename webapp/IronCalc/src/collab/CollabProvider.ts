import type { CollabPresence, Model } from "@ironcalc/wasm";

// Websocket provider for collaborative editing (design doc §11, phase 10).
//
// The wasm `Model` owns the CRDT peer (`collabAttach` & friends) and speaks
// an opaque binary frame protocol; this class owns the websocket to the
// collab relay server and shuttles frames both ways:
//
//   - on (re)open it sends the handshake (`collabStartSync`), which also
//     re-publishes our presence and heals any edits made while offline;
//   - every incoming message goes through `collabHandleFrame`, replies are
//     sent back, and `remoteUpdate` / `presenceChange` events fire so the
//     UI can repaint;
//   - local edits are shipped on a short interval via `collabFlushLocal`
//     (a no-op returning nothing when the model is unchanged).
//
// Large frames (a workbook's full state is tens of MB) travel gzip-wrapped
// in a custom y-sync message agreed with the relay
// (`collab-server/src/compress.rs`); wrapping and unwrapping happen here,
// with the platform's CompressionStream/DecompressionStream, so the wasm
// peer only ever sees plain frames.
//
// The provider never touches cell content; everything it sends is opaque.

export type CollabStatus =
  | "connecting"
  | "connected"
  /** Applying a large document frame (typically the room state on join). */
  | "syncing"
  | "disconnected";

// The subset of the browser WebSocket API the provider uses; tests inject
// in-process fakes through `createWebSocket`.
export interface CollabWebSocket {
  binaryType: string;
  readyState: number;
  onopen: (() => void) | null;
  onmessage: ((event: { data: ArrayBuffer }) => void) | null;
  onclose: (() => void) | null;
  onerror: (() => void) | null;
  send(data: Uint8Array): void;
  close(): void;
}

export interface CollabProviderOptions {
  /** How often pending local edits are shipped (default 200ms). */
  flushIntervalMs?: number;
  /** Initial reconnect delay; doubles up to `maxReconnectDelayMs`. */
  reconnectDelayMs?: number;
  maxReconnectDelayMs?: number;
  /** Override the yjs client id (must be unique in the room). */
  clientId?: number;
  /** Display name published with this client's presence. */
  userName?: string;
  /** Websocket factory, injectable for tests. */
  createWebSocket?: (url: string) => CollabWebSocket;
}

const WS_OPEN = 1;

/**
 * Frames at least this large are applied only after the UI has had a chance
 * to paint: applying is synchronous and a big workbook takes seconds.
 */
const LARGE_FRAME_BYTES = 256 * 1024;

/** Runs `callback` after the browser has painted once. */
function afterPaint(callback: () => void): void {
  if (typeof requestAnimationFrame === "function") {
    requestAnimationFrame(() => setTimeout(callback, 0));
  } else {
    setTimeout(callback, 0);
  }
}

// ---- gzip-wrapped frames (must match collab-server/src/compress.rs) ----

/** Custom y-sync message tag of a gzip-wrapped frame. */
const MSG_GZIP = 0x10;
/** Outgoing frames at least this large are gzip-wrapped. */
const COMPRESS_THRESHOLD = 64 * 1024;

function canCompress(): boolean {
  return typeof CompressionStream === "function";
}

function canDecompress(): boolean {
  return typeof DecompressionStream === "function";
}

/** lib0 unsigned varint: 7 bits per byte, least significant first. */
function readVarUint(
  bytes: Uint8Array,
  offset: number,
): { value: number; next: number } | null {
  let value = 0;
  let scale = 1;
  let position = offset;
  while (position < bytes.length) {
    const byte = bytes[position++];
    value += (byte & 0x7f) * scale;
    if ((byte & 0x80) === 0) {
      return { value, next: position };
    }
    scale *= 128;
    if (scale > Number.MAX_SAFE_INTEGER) {
      return null;
    }
  }
  return null;
}

function writeVarUint(value: number): number[] {
  const out: number[] = [];
  let rest = value;
  while (rest >= 0x80) {
    out.push((rest % 128) | 0x80);
    rest = Math.floor(rest / 128);
  }
  out.push(rest);
  return out;
}

/** The gzip payload of a wrapped frame, or null for a plain frame. */
function gzipPayload(frame: Uint8Array): Uint8Array | null {
  if (frame.length === 0 || frame[0] !== MSG_GZIP) {
    return null;
  }
  const header = readVarUint(frame, 1);
  if (!header || header.next + header.value !== frame.length) {
    return null;
  }
  return frame.subarray(header.next);
}

function wrapGzip(compressed: Uint8Array): Uint8Array {
  const header = [MSG_GZIP, ...writeVarUint(compressed.length)];
  const out = new Uint8Array(header.length + compressed.length);
  out.set(header, 0);
  out.set(compressed, header.length);
  return out;
}

async function pipeThrough(
  data: Uint8Array,
  transform: ReadableWritablePair<Uint8Array, BufferSource>,
): Promise<Uint8Array> {
  const source = new ReadableStream<BufferSource>({
    start(controller) {
      // `slice` yields an ArrayBuffer-backed copy (the streams API rejects
      // views over other buffer kinds).
      controller.enqueue(data.slice());
      controller.close();
    },
  });
  const stream = source.pipeThrough(transform);
  return new Uint8Array(await new Response(stream).arrayBuffer());
}

function deflate(data: Uint8Array): Promise<Uint8Array> {
  return pipeThrough(data, new CompressionStream("gzip"));
}

function inflate(data: Uint8Array): Promise<Uint8Array> {
  return pipeThrough(data, new DecompressionStream("gzip"));
}

function randomClientId(): number {
  const buffer = new Uint32Array(1);
  crypto.getRandomValues(buffer);
  return buffer[0];
}

const RANDOM_USER_NAMES = [
  "Amber",
  "Crimson",
  "Coral",
  "Golden",
  "Indigo",
  "Ivory",
  "Jade",
  "Marble",
  "Onyx",
  "Opal",
  "Saffron",
  "Scarlet",
  "Silver",
  "Teal",
  "Velvet",
  "Willow",
];

const RANDOM_USER_LASTNAMES = [
  "Fox",
  "Ibis",
  "Heron",
  "Lynx",
  "Otter",
  "Falcon",
  "Swallow",
  "Wren",
  "Swift",
  "Finch",
  "Crane",
  "Tanager",
  "Marten",
  "Kingfisher",
  "Sparrow",
  "Egret",
];

function randomUserName(): string {
  const firstName =
    RANDOM_USER_NAMES[Math.floor(Math.random() * RANDOM_USER_NAMES.length)];
  const lastName =
    RANDOM_USER_LASTNAMES[
      Math.floor(Math.random() * RANDOM_USER_LASTNAMES.length)
    ];
  return `${firstName} ${lastName}`;
}

export class CollabProvider {
  readonly clientId: number;
  readonly userName: string;

  private model: Model;
  private url: string;
  private socket: CollabWebSocket | null = null;
  private currentStatus: CollabStatus = "disconnected";
  private destroyed = false;

  private flushIntervalMs: number;
  private reconnectDelayMs: number;
  private maxReconnectDelayMs: number;
  private nextReconnectDelayMs: number;
  private createWebSocket: (url: string) => CollabWebSocket;

  private flushTimer: ReturnType<typeof setInterval> | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;

  /** Received frames not yet applied (applied strictly in arrival order). */
  private inbox: Uint8Array[] = [];
  /** A paint or an inflate is pending before the head frame is applied. */
  private inboxWaiting = false;
  /** Outgoing frames are sent in call order even when some are compressed
   *  asynchronously: `outbound` is the tail of that chain. */
  private outbound: Promise<void> = Promise.resolve();
  private outboundPending = 0;

  private remoteUpdateHandlers = new Set<() => void>();
  private presenceChangeHandlers = new Set<() => void>();
  private statusChangeHandlers = new Set<(status: CollabStatus) => void>();

  constructor(model: Model, url: string, options: CollabProviderOptions = {}) {
    this.model = model;
    this.url = url;
    this.flushIntervalMs = options.flushIntervalMs ?? 200;
    this.reconnectDelayMs = options.reconnectDelayMs ?? 500;
    this.maxReconnectDelayMs = options.maxReconnectDelayMs ?? 10_000;
    this.nextReconnectDelayMs = this.reconnectDelayMs;
    this.createWebSocket =
      options.createWebSocket ??
      ((wsUrl: string) => new WebSocket(wsUrl) as unknown as CollabWebSocket);
    this.clientId = options.clientId ?? randomClientId();
    this.userName = options.userName ?? randomUserName();
    if (!model.collabIsAttached()) {
      model.collabAttach(this.clientId);
    }
  }

  get status(): CollabStatus {
    return this.currentStatus;
  }

  /** Opens the websocket and starts the flush loop. */
  connect(): void {
    if (this.destroyed || this.socket) {
      return;
    }
    this.setStatus("connecting");
    let socket: CollabWebSocket;
    try {
      socket = this.createWebSocket(this.url);
    } catch (error) {
      console.warn("collab: cannot open websocket", error);
      this.scheduleReconnect();
      return;
    }
    socket.binaryType = "arraybuffer";
    socket.onopen = () => {
      this.nextReconnectDelayMs = this.reconnectDelayMs;
      this.setStatus("connected");
      // Handshake: sync steps, presence query and our own presence. It
      // also carries every local edit the other side has not seen, so
      // offline edits heal here.
      this.send(this.model.collabStartSync());
    };
    socket.onmessage = (event) => {
      this.inbox.push(new Uint8Array(event.data));
      this.drainInbox();
    };
    socket.onclose = () => {
      this.socket = null;
      this.setStatus("disconnected");
      this.scheduleReconnect();
    };
    socket.onerror = () => {
      socket.close();
    };
    this.socket = socket;
    if (this.flushTimer === null) {
      this.flushTimer = setInterval(
        () => this.flushNow(),
        this.flushIntervalMs,
      );
    }
  }

  /** Ships pending local edits immediately (also runs on the interval). */
  flushNow(): void {
    if (this.destroyed) {
      return;
    }
    let frame: Uint8Array | undefined;
    try {
      // Called even while disconnected: it folds model edits into the CRDT
      // doc, so the reconnect handshake carries them.
      frame = this.model.collabFlushLocal();
    } catch (error) {
      console.warn("collab: flush failed", error);
      return;
    }
    if (frame !== undefined) {
      this.send(frame);
    }
  }

  /**
   * Publishes this client's presence (user name, selection, …). The value
   * is serialized to JSON and treated as opaque by the server.
   */
  setPresence(state: unknown): void {
    this.send(this.model.collabSetPresence(JSON.stringify(state)));
  }

  clearPresence(): void {
    this.send(this.model.collabClearPresence());
  }

  /** The current presence map, including this client (when published). */
  presence(): CollabPresence[] {
    return this.model.collabPresence();
  }

  onRemoteUpdate(handler: () => void): () => void {
    this.remoteUpdateHandlers.add(handler);
    return () => this.remoteUpdateHandlers.delete(handler);
  }

  onPresenceChange(handler: () => void): () => void {
    this.presenceChangeHandlers.add(handler);
    return () => this.presenceChangeHandlers.delete(handler);
  }

  onStatusChange(handler: (status: CollabStatus) => void): () => void {
    this.statusChangeHandlers.add(handler);
    return () => this.statusChangeHandlers.delete(handler);
  }

  /** Withdraws presence, closes the websocket and stops all timers. */
  destroy(): void {
    if (this.destroyed) {
      return;
    }
    // Flush what we can and say goodbye before closing.
    this.flushNow();
    this.destroyed = true;
    try {
      this.send(this.model.collabClearPresence());
    } catch {
      // The model may already be gone; closing is all that is left.
    }
    if (this.flushTimer !== null) {
      clearInterval(this.flushTimer);
      this.flushTimer = null;
    }
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.socket) {
      const socket = this.socket;
      this.socket = null;
      socket.onclose = null;
      socket.close();
    }
    this.inbox = [];
    this.setStatus("disconnected");
    this.remoteUpdateHandlers.clear();
    this.presenceChangeHandlers.clear();
    this.statusChangeHandlers.clear();
  }

  /**
   * Applies queued frames in order. Before a large frame the status flips
   * to "syncing" and the apply is deferred past the next paint, so the UI
   * can show that the workbook is loading instead of freezing silently.
   */
  private drainInbox(): void {
    if (this.inboxWaiting || this.destroyed) {
      return;
    }
    while (this.inbox.length > 0) {
      const frame = this.inbox[0];
      const payload = gzipPayload(frame);
      if (payload) {
        if (!canDecompress()) {
          console.warn("collab: dropping compressed frame (no gzip support)");
          this.inbox.shift();
          continue;
        }
        // Wrapped frames are large by construction: show the syncing state
        // now, inflate, then apply after a paint (see below).
        if (this.currentStatus === "connected") {
          this.setStatus("syncing");
        }
        this.inboxWaiting = true;
        inflate(payload).then(
          (inflated) => {
            if (this.inbox[0] === frame) {
              this.inbox[0] = inflated;
            }
            afterPaint(() => {
              this.inboxWaiting = false;
              this.drainInbox();
            });
          },
          (error) => {
            console.warn(
              "collab: dropping undecodable compressed frame",
              error,
            );
            if (this.inbox[0] === frame) {
              this.inbox.shift();
            }
            this.inboxWaiting = false;
            this.drainInbox();
          },
        );
        return;
      }
      if (
        frame.length >= LARGE_FRAME_BYTES &&
        this.currentStatus === "connected"
      ) {
        this.setStatus("syncing");
        this.inboxWaiting = true;
        afterPaint(() => {
          this.inboxWaiting = false;
          this.drainInbox();
        });
        return;
      }
      this.inbox.shift();
      this.handleFrame(frame);
      if (this.currentStatus === "syncing") {
        const open = this.socket?.readyState === WS_OPEN;
        this.setStatus(open ? "connected" : "disconnected");
      }
    }
  }

  private handleFrame(data: Uint8Array): void {
    let outcome: ReturnType<Model["collabHandleFrame"]>;
    try {
      outcome = this.model.collabHandleFrame(data);
    } catch (error) {
      console.warn("collab: dropping malformed frame", error);
      return;
    }
    const replies = outcome.replies;
    if (replies.length > 0) {
      this.send(replies);
    }
    if (outcome.appliedUpdate) {
      for (const handler of this.remoteUpdateHandlers) {
        handler();
      }
    }
    if (outcome.presenceChanged) {
      for (const handler of this.presenceChangeHandlers) {
        handler();
      }
    }
  }

  private send(frame: Uint8Array): void {
    if (frame.length === 0) {
      return;
    }
    const compress = frame.length >= COMPRESS_THRESHOLD && canCompress();
    if (!compress && this.outboundPending === 0) {
      this.sendNow(frame);
      return;
    }
    // Compression is asynchronous; later frames queue behind it so the
    // relay sees them in order (out-of-order updates would only cost a
    // resync round trip, but there is no reason to pay it).
    this.outboundPending += 1;
    this.outbound = this.outbound
      .then(async () => {
        let data = frame;
        if (compress) {
          try {
            data = wrapGzip(await deflate(frame));
          } catch (error) {
            console.warn("collab: sending frame uncompressed", error);
          }
        }
        this.sendNow(data);
      })
      .finally(() => {
        this.outboundPending -= 1;
      });
  }

  private sendNow(frame: Uint8Array): void {
    const socket = this.socket;
    if (socket && socket.readyState === WS_OPEN) {
      // Frames dropped while closed are recovered by the next handshake.
      socket.send(frame);
    }
  }

  private scheduleReconnect(): void {
    if (this.destroyed || this.reconnectTimer !== null) {
      return;
    }
    const delay = this.nextReconnectDelayMs;
    this.nextReconnectDelayMs = Math.min(delay * 2, this.maxReconnectDelayMs);
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, delay);
  }

  private setStatus(status: CollabStatus): void {
    if (status === this.currentStatus) {
      return;
    }
    this.currentStatus = status;
    for (const handler of this.statusChangeHandlers) {
      handler(status);
    }
  }
}
