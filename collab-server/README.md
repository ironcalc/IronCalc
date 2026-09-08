# ironcalc_collab_server

A small websocket relay that lets several IronCalc clients edit the same
workbook at the same time.

The server knows nothing about spreadsheets. It does not depend on the
engine, never parses a cell and never evaluates a formula. All it holds per
room is a [yrs](https://github.com/y-crdt/y-crdt) document (the Rust port of
Yjs) and it speaks the standard **y-sync** protocol, the same wire format as
`y-websocket`. Everything spreadsheet-specific lives in the clients, in
`base/src/crdt/` (see `CRDTs/CRDT-design.md`, §11 in particular).

## What the server does

1. **Rooms.** The URL path of the websocket request names the room:
   `ws://host:9000/budget-2026` joins room `budget-2026`. A room is created
   on first use and lives for the lifetime of the process. An empty path maps
   to the room `default`. Names are restricted to `[A-Za-z0-9._-]`, at most
   64 characters and not starting with a dot, because they double as file
   names on disk.

2. **Late joiners.** Each room owns a yrs document that integrates every
   update it ever saw. When a client connects the server sends its state
   vector (`SyncStep1`). The client answers with the updates the server is
   missing and, symmetrically, asks for what it is missing. After that
   handshake a fresh client has the whole workbook, even if every other
   collaborator has gone home.

3. **Fan-out.** Every update that gets integrated into the room document is
   broadcast to every connection in the room, including the one it came
   from. Clients deduplicate, so the echo is harmless and doubles as an
   acknowledgement. Broadcasting goes through a tokio `broadcast` channel
   with a bounded queue per connection. A client that falls too far behind is
   disconnected and heals by reconnecting and redoing the handshake.

4. **Presence (awareness).** Cursor positions, user names and the like travel
   as y-sync awareness messages. The server keeps the current presence map so
   a late joiner learns who is already there, relays every change, and when a
   connection drops it removes that connection's presence and tells the room.
   Presence is never written to disk.

5. **Persistence (optional).** With a data directory, each room is stored as
   `<room>.snapshot` (one full-state update) plus `<room>.log`, an
   append-only, length-prefixed list of incremental updates. Every integrated
   update is appended before it is broadcast. When the log exceeds 1 MiB it
   is compacted into a new snapshot, written to a temporary file and renamed
   so a crash never leaves a half-written snapshot. A torn tail in the log
   (crash mid-append) is detected on load and truncated. On restart the room
   replays snapshot then log before accepting connections.

6. **Causal gaps.** yrs 0.27.3 mishandles updates that arrive before the
   updates they depend on (it parks them and never re-integrates them). The
   server therefore classifies every incoming update against its state vector
   (`src/protocol.rs`, a copy of the client-side rule). Contiguous updates are
   applied; gapped ones are stashed, the server asks the sender for a resync,
   and the stash is retried after every successful apply. The stash is capped
   at 64 entries; beyond that it is simply dropped because the resync
   re-delivers everything anyway.

The server is **not an ordering authority**. Convergence comes from the CRDT
itself: two clients that exchange updates directly, in any order, end up in
the same state. The relay is only a convenience for fan-out, late joining and
durability.

## Source layout

| File | Role |
| --- | --- |
| `src/main.rs` | Binary: parses `addr` and optional `data_dir`, binds, runs. |
| `src/server.rs` | Accept loop, room registry, per-connection pump (`select!` over socket input and room broadcast). |
| `src/room.rs` | One room: yrs doc + awareness + broadcast sender. Handles each y-sync message, gap stash, compaction trigger. |
| `src/protocol.rs` | `classify_update`: is an update contiguous, gapped or already known? |
| `src/storage.rs` | Snapshot + log files, torn-tail recovery, compaction. |
| `tests/integration.rs` | Real `SyncPeer` clients over live websockets: convergence, room isolation, presence pruning, restart. |

## Running it

```sh
# Ephemeral: rooms vanish when the process exits.
cargo run -p ironcalc_collab_server

# Custom address and a directory to persist rooms in.
cargo run -p ironcalc_collab_server -- 0.0.0.0:9000 ./collab-data
```

The first argument is the bind address (default `127.0.0.1:9000`), the second
the data directory (default: none, ephemeral). There is no TLS and no
authentication: put it behind a reverse proxy if you expose it.

Tests:

```sh
cargo test -p ironcalc_collab_server
```

## A minimal example

The client side is `ironcalc_base::crdt::SyncPeer`. It attaches to a
`UserModel`, translates local edits into CRDT updates and applies remote
updates back into the model. It is transport-agnostic: it hands you byte
frames and you decide how to ship them. Below, two peers in one process join
the same room and converge.

The code is checked in as `examples/two_peers.rs`; run it with a server
listening on `127.0.0.1:9000`:

```sh
cargo run -p ironcalc_collab_server            # terminal 1
cargo run -p ironcalc_collab_server --example two_peers   # terminal 2
```

```rust
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use ironcalc_base::crdt::SyncPeer;
use ironcalc_base::UserModel;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// One collaborator: a workbook, its CRDT peer and a websocket to the relay.
struct Client {
    model: UserModel<'static>,
    peer: SyncPeer,
    socket: Socket,
}

impl Client {
    async fn connect(room_url: &str, client_id: u64) -> Client {
        let mut model = UserModel::new_empty("wb", "en", "UTC", "en").unwrap();
        let peer = SyncPeer::attach(&mut model, client_id).unwrap();
        let (socket, _) = tokio_tungstenite::connect_async(room_url).await.unwrap();
        let mut client = Client { model, peer, socket };
        // Handshake: send our state vector, the relay answers with what we
        // miss and asks for what it misses. Pump until the line goes quiet.
        for frame in client.peer.start_sync() {
            client.send(frame).await;
        }
        while client.step(Duration::from_millis(300)).await {}
        client
    }

    async fn send(&mut self, frame: Vec<u8>) {
        self.socket.send(WsMessage::Binary(frame)).await.unwrap();
    }

    /// Ships pending local edits, if any.
    async fn flush(&mut self) {
        if let Some(frame) = self.peer.flush_local(&mut self.model).unwrap() {
            self.send(frame).await;
        }
    }

    /// Handles one incoming frame (answering its replies); false on timeout.
    async fn step(&mut self, wait: Duration) -> bool {
        match tokio::time::timeout(wait, self.socket.next()).await {
            Ok(Some(Ok(WsMessage::Binary(data)))) => {
                let outcome = self.peer.handle_frame(&mut self.model, &data).unwrap();
                for reply in outcome.replies {
                    self.send(reply).await;
                }
                true
            }
            Ok(Some(Ok(_))) => true, // ping/pong/text: ignore
            _ => false,
        }
    }

    fn cell(&self, row: i32, column: i32) -> String {
        self.model.get_formatted_cell_value(0, row, column).unwrap()
    }
}

#[tokio::main]
async fn main() {
    let room = "ws://127.0.0.1:9000/demo";
    let mut alice = Client::connect(room, 1).await;
    let mut bob = Client::connect(room, 2).await;

    // Alice types into A1; Bob answers with a formula over it.
    alice.model.set_user_input(0, 1, 1, "5").unwrap();
    alice.flush().await;
    while bob.cell(1, 1) != "5" {
        assert!(bob.step(Duration::from_secs(5)).await, "no update from Alice");
    }
    bob.model.set_user_input(0, 1, 2, "=A1+1").unwrap();
    bob.flush().await;
    while alice.cell(1, 2) != "6" {
        assert!(alice.step(Duration::from_secs(5)).await, "no update from Bob");
    }
    println!("Alice sees B1 = {}", alice.cell(1, 2));
}
```

The `Client` shape is the whole client contract:

- on (re)connect, send everything `start_sync()` returns and keep reading
  until the handshake settles. This matters: `attach` seeds the document
  with the initial workbook state, and that seed only reaches the relay
  through the handshake. An edit flushed before it would arrive with a
  causal gap and sit in the relay's stash until the resync completes;
- for every incoming frame call `handle_frame` and send back its `replies`;
  `applied_update` and `presence_changed` tell you whether to repaint;
- after local edits call `flush_local` and send the frame, if any;
- `set_presence(json)` / `clear_presence()` return a frame announcing your
  cursor or name.

### From the browser

The wasm bindings expose the same peer as `collabAttach`, `collabStartSync`,
`collabHandleFrame`, `collabFlushLocal` and `collabSetPresence` on `Model`,
and `@ironcalc/workbook` wraps the websocket plumbing in `CollabProvider`:

```ts
import { Model } from "@ironcalc/wasm";
import { CollabProvider, IronCalc } from "@ironcalc/workbook";

const model = new Model("wb", "en", "UTC");
const provider = new CollabProvider(model, "ws://127.0.0.1:9000/demo", {
  userName: "Ana",
});
provider.connect();
// <IronCalc model={model} collabProvider={provider} />
```

The provider runs the handshake on open, flushes local edits every 200 ms,
reconnects with backoff, and fires `remoteUpdate` / `presenceChange` events
so the UI can repaint.
