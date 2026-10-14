//! Timing harness for the collaboration pipeline on a forward-chain workbook.
//!
//! Mimics the webapp flows:
//!   host:   click "Collaborate" = SyncPeer::attach + handshake (full state out)
//!   server: room doc applies the host's SyncStep2
//!   joiner: empty model + handshake, applies the room's full state
//!   then one remote edit round trip host -> joiner and joiner -> host.
//!
//! Usage: cargo run --release --example collab_bench -- [rows]

// A timing harness: any failure should abort loudly.
#![allow(clippy::unwrap_used)]

use std::time::Instant;

use ironcalc_base::crdt::SyncPeer;
use ironcalc_base::{Model, UserModel};
use yrs::sync::{Message, MessageReader, SyncMessage};
use yrs::updates::decoder::{Decode, DecoderV1};
use yrs::updates::encoder::Encode;
use yrs::{Doc, ReadTxn, StateVector, Transact, Update};

fn messages(frame: &[u8]) -> Vec<Message> {
    let mut decoder = DecoderV1::from(frame);
    MessageReader::new(&mut decoder)
        .collect::<Result<_, _>>()
        .unwrap()
}

fn mb(bytes: usize) -> String {
    format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
}

/// Minimal in-process relay room (same logic as collab-server/src/room.rs).
struct Room {
    doc: Doc,
}

impl Room {
    fn hello(&self) -> Vec<u8> {
        let sv = self.doc.transact().state_vector();
        Message::Sync(SyncMessage::SyncStep1(sv)).encode_v1()
    }

    /// Returns (direct replies, broadcast frames).
    fn handle(&self, frame: &[u8]) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
        let mut replies = Vec::new();
        let mut broadcast = Vec::new();
        for message in messages(frame) {
            match message {
                Message::Sync(SyncMessage::SyncStep1(sv)) => {
                    let diff = self.doc.transact().encode_state_as_update_v1(&sv);
                    replies.push(Message::Sync(SyncMessage::SyncStep2(diff)).encode_v1());
                }
                Message::Sync(SyncMessage::SyncStep2(update))
                | Message::Sync(SyncMessage::Update(update)) => {
                    if update == [0, 0] {
                        continue;
                    }
                    let t = Instant::now();
                    let decoded = Update::decode_v1(&update).unwrap();
                    let t_decode = t.elapsed();
                    let t = Instant::now();
                    self.doc.transact_mut().apply_update(decoded).unwrap();
                    println!(
                        "    server: decode {:?}, apply {:?} ({})",
                        t_decode,
                        t.elapsed(),
                        mb(update.len())
                    );
                    broadcast.push(Message::Sync(SyncMessage::Update(update)).encode_v1());
                }
                _ => {}
            }
        }
        (replies, broadcast)
    }
}

fn timed<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let t = Instant::now();
    let r = f();
    println!("  {label}: {:?}", t.elapsed());
    r
}

fn main() {
    let rows: i32 = std::env::args()
        .nth(1)
        .map(|v| v.parse().unwrap())
        .unwrap_or(100_000);
    println!("rows = {rows}");

    // ---- host builds the workbook (like the xlsx import) ----
    let mut host = timed("host: build + evaluate", || {
        let mut model = Model::new_empty("forward-chain.xlsx", "en", "UTC", "en").unwrap();
        model.set_user_input(0, 1, 1, "1".to_string()).unwrap();
        for row in 2..=rows {
            model
                .set_user_input(0, row, 1, format!("=A{}+1", row - 1))
                .unwrap();
        }
        model.evaluate();
        UserModel::from_model(model)
    });

    if let Ok(path) = std::env::var("COLLAB_BENCH_DUMP_MODEL") {
        std::fs::write(&path, host.to_bytes()).unwrap();
        println!("  host model written to {path}");
    }
    // ---- host clicks "Collaborate" ----
    let mut host_peer = timed("host: SyncPeer::attach", || {
        SyncPeer::attach(&mut host, 1).unwrap()
    });
    let host_start = timed("host: start_sync", || host_peer.start_sync());

    let room = Room { doc: Doc::new() };
    let hello = room.hello();
    // host handles the server hello (SyncStep1 with empty sv) -> full state
    let outcome = timed("host: handle hello (handshake_diff of full state)", || {
        host_peer.handle_frame(&mut host, &hello).unwrap()
    });
    let mut host_broadcasts = Vec::new();
    for frame in host_start.iter().chain(outcome.replies.iter()) {
        println!("  host -> server frame {}", mb(frame.len()));
        let (replies, broadcast) = room.handle(frame);
        host_broadcasts.extend(broadcast);
        for reply in replies {
            timed("host: handle server reply", || {
                host_peer.handle_frame(&mut host, &reply).unwrap()
            });
        }
    }
    // The server fans the host's own update back to the host.
    for frame in &host_broadcasts {
        timed("host: handle echo of own full state", || {
            host_peer.handle_frame(&mut host, frame).unwrap()
        });
    }
    let full_state = room
        .doc
        .transact()
        .encode_state_as_update_v1(&StateVector::default());
    println!("  room doc full state = {}", mb(full_state.len()));
    // COLLAB_BENCH_DUMP=<path> writes the full state (e.g. to measure how
    // well it compresses).
    if let Ok(path) = std::env::var("COLLAB_BENCH_DUMP") {
        std::fs::write(&path, &full_state).unwrap();
        println!("  full state written to {path}");
    }

    // ---- joiner opens the URL ----
    let mut joiner = UserModel::new_empty("", "en", "UTC", "en").unwrap();
    let mut joiner_peer = timed("joiner: attach (empty)", || {
        SyncPeer::attach(&mut joiner, 2).unwrap()
    });
    let hello = room.hello();
    let outcome = timed("joiner: handle hello", || {
        joiner_peer.handle_frame(&mut joiner, &hello).unwrap()
    });
    let mut to_server: Vec<Vec<u8>> = joiner_peer.start_sync();
    to_server.extend(outcome.replies);
    let mut joiner_broadcasts = Vec::new();
    for frame in to_server {
        let (replies, broadcast) = room.handle(&frame);
        joiner_broadcasts.extend(broadcast);
        for reply in replies {
            println!("  server -> joiner frame {}", mb(reply.len()));
            timed(
                "joiner: handle SyncStep2 (apply full state + reconcile)",
                || joiner_peer.handle_frame(&mut joiner, &reply).unwrap(),
            );
        }
    }
    println!(
        "  joiner A{rows} = {}",
        joiner.get_formatted_cell_value(0, rows, 1).unwrap()
    );
    for frame in &joiner_broadcasts {
        timed("host: handle joiner's bootstrap update", || {
            host_peer.handle_frame(&mut host, frame).unwrap()
        });
    }

    // ---- steady state: one edit each way ----
    host.set_user_input(0, 1, 2, "hello").unwrap();
    let frame = timed("host: flush_local (one edit)", || {
        host_peer.flush_local(&mut host).unwrap().unwrap()
    });
    let (_, broadcast) = room.handle(&frame);
    for frame in &broadcast {
        timed("joiner: handle one remote edit", || {
            joiner_peer.handle_frame(&mut joiner, frame).unwrap()
        });
        timed("host: handle echo of one edit", || {
            host_peer.handle_frame(&mut host, frame).unwrap()
        });
    }
    joiner.set_user_input(0, 2, 2, "world").unwrap();
    let frame = timed("joiner: flush_local (one edit)", || {
        joiner_peer.flush_local(&mut joiner).unwrap().unwrap()
    });
    let (_, broadcast) = room.handle(&frame);
    for frame in &broadcast {
        timed("host: handle one remote edit", || {
            host_peer.handle_frame(&mut host, frame).unwrap()
        });
    }
    println!(
        "  host B2 = {}",
        host.get_formatted_cell_value(0, 2, 2).unwrap()
    );
}
