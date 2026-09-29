//! Websocket relay for IronCalc collaboration (design doc §11, phase 9).
//!
//! One room per URL path. The server participates in the y-sync protocol
//! with a per-room [yrs] document: it answers handshakes, applies incoming
//! updates and fans the integrated changes out to every connection in the
//! room. It never depends on the spreadsheet engine and never inspects cell
//! content — it is a fan-out, late-joiner and (phase 9.3) persistence
//! convenience, not an ordering authority.
//!
//! Wire format: binary websocket messages, each carrying one or more y-sync
//! protocol messages (`yrs::sync::Message`, lib0 v1) — the same framing
//! `base/src/crdt/sync.rs` speaks, compatible with y-websocket — except that
//! large messages are gzip-wrapped in a custom y-sync message (see
//! [`compress`]), which clients must unwrap before handing the frame to
//! their peer.

pub mod compress;
pub mod protocol;
pub mod room;
pub mod server;
pub mod storage;
