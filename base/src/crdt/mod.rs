//! CRDT-based collaboration for IronCalc.
//!
//! This module mirrors the shared state of a workbook into a [yrs] document
//! (the convergence point) and translates between the `user_model` diff stream
//! and that document. See `CRDTs/CRDT-design.md` in the repository root for the
//! full design: stable ids (`ids`), fractional ordering (`order`), the flat doc
//! schema and projection (`projection`), and the session driving the
//! translation in both directions (`session`).

mod formula;
mod ids;
mod materialize;
mod order;
mod projection;
mod session;
mod sync;

pub use ids::EntityId;
pub use materialize::materialize_doc_update;
pub use session::CollabSession;
pub use sync::{FrameOutcome, SyncPeer};
