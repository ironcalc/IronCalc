//! Abstraction over a partially ordered log (PO-Log) of operations.
//!
//! We do not own the log. It is supplied by the hosting framework, which is responsible for commit
//! identity, causality (the parent DAG), deduplication, persistence and transport. Our side is a
//! deterministic state function that the framework drives through [`Consumer`].
//!
//! # Contract
//!
//! The framework guarantees:
//!
//! - each [`CommitId`] is delivered to [`Consumer::apply`] **at most once** — deduplication is the
//!   log's responsibility, not ours,
//! - commits arrive in **causal order**: every parent is applied before its children,
//! - commits are **append-only and immutable**; a delivered commit is never retracted or rewritten
//!   (an undo is a new commit carrying the inverse patches),
//! - [`Commit::hlc`] is the stamp its author minted for it (the HLC send rule), and every replica
//!   folds it back into its own clock on apply (the receive rule), so a commit's stamp always sorts
//!   above those of its ancestors.
//!
//! In return [`Consumer::apply`] is:
//!
//! - **deterministic**: the same sequence of commits always produces the same state,
//! - **commutative across concurrent commits**: two causally unrelated commits may arrive in either
//!   order and yield the same state. This is what removes the need to ever rewind and replay,
//! - **total**: a well-formed commit is never rejected. A patch addressing a deleted row is a no-op,
//!   not an error; `Error` is reserved for genuine corruption.

use crate::collab::hlc::Hlc;
use crate::collab::patch::Patch;
use bitcode::__private::{Buffer, Decoder, Encoder, View};
use bitcode::{Decode, Encode};
use smallvec::SmallVec;
use std::num::NonZeroUsize;
use std::ops::Deref;

/// Identifies a replica.
///
/// The hosting framework derives this from whatever it uses for session identity — a device id, a
/// public key — but it must derive it **deterministically**, so that every replica arrives at the
/// same `SessionId` for the same peer. A locally assigned number would make [`Timestamp`]
/// comparisons resolve differently on different replicas, which is divergence.
///
/// It doubles as the session suffix of a
/// [`FractionalKey`](crate::collab::fractional_index::FractionalKey), so one identity orders both
/// concurrent register writes and concurrent inserts.
pub type SessionId = u32;

/// Identifies a commit. Opaque to us — we compare and store it, never interpret it. Short ids stay
/// inline, so carrying one costs no allocation.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct CommitId(SmallVec<[u8; 8]>);

impl From<&[u8]> for CommitId {
    fn from(bytes: &[u8]) -> Self {
        CommitId(SmallVec::from_slice(bytes))
    }
}

impl Deref for CommitId {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0
    }
}

/// bitcode coders delegating to the byte-slice ones — hand-rolled for the same reason as
/// [`FractionalKeyEncoder`](crate::collab::fractional_key::FractionalKeyEncoder).
#[derive(Default)]
pub struct CommitIdEncoder(<[u8] as Encode>::Encoder);

impl Buffer for CommitIdEncoder {
    fn collect_into(&mut self, out: &mut Vec<u8>) {
        self.0.collect_into(out);
    }

    fn reserve(&mut self, additional: NonZeroUsize) {
        self.0.reserve(additional);
    }
}

impl Encoder<CommitId> for CommitIdEncoder {
    #[inline]
    fn encode(&mut self, t: &CommitId) {
        Encoder::<[u8]>::encode(&mut self.0, t);
    }
}

impl Encode for CommitId {
    type Encoder = CommitIdEncoder;
}

#[derive(Default)]
pub struct CommitIdDecoder<'a>(<Vec<u8> as Decode<'a>>::Decoder);

impl<'a> View<'a> for CommitIdDecoder<'a> {
    fn populate(&mut self, input: &mut &'a [u8], length: usize) -> bitcode::__private::Result<()> {
        self.0.populate(input, length)
    }
}

impl<'a> Decoder<'a, CommitId> for CommitIdDecoder<'a> {
    #[inline]
    fn decode(&mut self) -> CommitId {
        let bytes: Vec<u8> = self.0.decode();
        CommitId::from(bytes.as_slice())
    }
}

impl<'a> Decode<'a> for CommitId {
    type Decoder = CommitIdDecoder<'a>;
}

/// Arbitrates between writes to the same register.
///
/// `hlc` is the stamp the originating commit was minted with, so a causally later write always has a
/// strictly greater one than its ancestors and wins. Concurrent writes can land on equal stamps, and
/// `session` breaks the tie deterministically — arbitrarily, but identically on every replica. The
/// [`Hlc`] carries no node component, so it is the only tiebreak there is.
///
/// Field order is significant: the derived [`Ord`] compares `hlc` first.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Encode, Decode)]
pub struct Timestamp {
    pub hlc: Hlc,
    pub session: SessionId,
}

impl Timestamp {
    pub fn new(hlc: Hlc, session: SessionId) -> Self {
        Timestamp { hlc, session }
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Timestamp::new(Hlc::new(0), 0)
    }
}

/// A last-write-wins register: a value tagged with the [`Timestamp`] of the write that produced it.
#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct Lww<T> {
    pub timestamp: Timestamp,
    pub value: T,
}

impl<T> Lww<T> {
    pub fn new(value: T, timestamp: Timestamp) -> Self {
        Lww { timestamp, value }
    }

    /// Merge an incoming write, keeping it if it beats the one already stored. Returns whether the
    /// stored value changed, so callers can drive recalculation and repaint.
    ///
    /// The comparison is `>=` rather than `>` so that two patches of the *same* commit — which share
    /// a timestamp — resolve to the last one applied. This stays idempotent on redelivery: replaying
    /// a commit writes the same values again.
    pub fn merge(&mut self, value: T, timestamp: &Timestamp) -> bool {
        if *timestamp >= self.timestamp {
            self.timestamp = *timestamp;
            self.value = value;
            true
        } else {
            false
        }
    }
}

/// A unit of work in the log: one or more [`Patch`]es committed together, applied atomically.
pub struct Commit<'a> {
    /// Unique identifier of this commit.
    pub id: &'a CommitId,
    /// Replica that authored it.
    pub session: &'a SessionId,
    /// The stamp its author minted for it, above every stamp that author had seen.
    pub hlc: Hlc,
    /// Patches to apply, in order.
    pub patches: &'a [Patch],
}

impl Commit<'_> {
    /// The [`Timestamp`] every register written by this commit is tagged with.
    pub fn timestamp(&self) -> Timestamp {
        Timestamp::new(self.hlc, *self.session)
    }
}

/// State driven by a partially ordered log. Implemented by us, called by the hosting framework.
pub trait Consumer {
    type Error;

    /// Apply a single commit. See the module documentation for the guarantees this must uphold.
    fn apply(&mut self, commit: Commit<'_>) -> Result<(), Self::Error>;
}

/// Implemented by consumers whose materialized state can be persisted, so that a framework can
/// resume from it instead of replaying the log from the beginning. The framework tracks its own
/// resume position.
pub trait Snapshot: Consumer + Sized {
    fn encode(&self) -> Vec<u8>;

    fn decode(bytes: &[u8], session: SessionId) -> Result<Self, Self::Error>;
}

/// Implemented by consumers that can be returned to their empty state, for frameworks that
/// re-linearize history and need to replay it from scratch.
pub trait Resettable: Consumer {
    fn reset(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts(hlc: u64, session: SessionId) -> Timestamp {
        Timestamp::new(Hlc::new(hlc), session)
    }

    #[test]
    fn causally_later_write_wins_regardless_of_session() {
        // A high session id must not let an older write survive a causally later one.
        let mut reg = Lww::new("foo", ts(1, 999));
        assert!(reg.merge("bar", &ts(2, 1)));
        assert_eq!(reg.value, "bar");
    }

    #[test]
    fn concurrent_writes_break_the_tie_on_session() {
        let mut reg = Lww::new("foo", ts(7, 2));
        assert!(!reg.merge("bar", &ts(7, 1)));
        assert_eq!(reg.value, "foo");
        assert!(reg.merge("baz", &ts(7, 3)));
        assert_eq!(reg.value, "baz");
    }

    #[test]
    fn merge_is_commutative_for_concurrent_writes() {
        let mut a = Lww::new("init", ts(0, 0));
        a.merge("x", &ts(3, 1));
        a.merge("y", &ts(3, 2));

        let mut b = Lww::new("init", ts(0, 0));
        b.merge("y", &ts(3, 2));
        b.merge("x", &ts(3, 1));

        assert_eq!(a.value, b.value);
    }

    #[test]
    fn merge_is_idempotent_on_redelivery() {
        let mut reg = Lww::new("foo", ts(1, 1));
        reg.merge("bar", &ts(2, 2));
        let once = reg.value;
        reg.merge("bar", &ts(2, 2));
        assert_eq!(reg.value, once);
    }

    #[test]
    fn patches_of_the_same_commit_resolve_to_the_last_one() {
        let t = ts(4, 1);
        let mut reg = Lww::new("first", t);
        assert!(reg.merge("second", &t));
        assert_eq!(reg.value, "second");
    }
}
