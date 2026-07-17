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
// The provider never touches cell content; everything it sends is opaque.

export type CollabStatus = "connecting" | "connected" | "disconnected";

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

function randomClientId(): number {
  const buffer = new Uint32Array(1);
  crypto.getRandomValues(buffer);
  return buffer[0];
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
    this.userName = options.userName ?? "Guest";
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
      this.handleFrame(new Uint8Array(event.data));
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
    this.setStatus("disconnected");
    this.remoteUpdateHandlers.clear();
    this.presenceChangeHandlers.clear();
    this.statusChangeHandlers.clear();
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
