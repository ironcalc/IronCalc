use bitcode::__private::{Buffer, Decoder, Encoder, View};
use bitcode::{Decode, Encode};
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::Bound;
use std::fmt;
use std::num::NonZeroUsize;
use std::ops::RangeBounds;

use crate::collab::codec::{decode_entries, encode_entries, CodecError, FORMAT_VERSION};
pub use crate::collab::fractional_key::{
    FractionalKey, KeyBuf, INLINE_CAP, MAX_KEY_LEN, SESSION_SUFFIX_LEN,
};
use crate::collab::hlc::Hlc;

/// Session suffix reserved for [`virtual_key`]. No replica may use it — see
/// [`CollaborativeWorkbook::new`](crate::collab::CollaborativeWorkbook::new).
pub const VIRTUAL_SESSION: [u8; SESSION_SUFFIX_LEN] = [0; SESSION_SUFFIX_LEN];

/// The key of the `ordinal`-th (1-based) row or column that nobody ever explicitly inserted.
///
/// It is session-free and deterministic, so two replicas materializing "row 5" independently mint the
/// same key and their edits meet in the same cell. Positions are spaced two apart so that a key can
/// always be minted between two consecutive virtual ones.
pub fn virtual_key(ordinal: u32) -> FractionalKey {
    let position = (2 * ordinal).to_be_bytes();
    let mut buf = KeyBuf::from(&position[1..]);
    buf.extend_from_slice(&VIRTUAL_SESSION);
    FractionalKey::try_from_bytes(&buf).expect("virtual key is 7 bytes")
}

/// Inverse of [`virtual_key`]: `None` for anything a session actually minted.
pub fn virtual_ordinal(key: &FractionalKey) -> Option<u32> {
    let (position, session) = key.split();
    if session != VIRTUAL_SESSION || position.len() != 3 {
        return None;
    }
    let n = u32::from_be_bytes([0, position[0], position[1], position[2]]);
    if n == 0 || n % 2 != 0 {
        return None;
    }
    Some(n / 2)
}

/// A single element of a [FractionalIndex]: where it sits, and the key it answers to.
///
/// The two are the same key until the element is moved. A move mints a fresh key for the new
/// position and keeps the original as the element's [identity](Entry::identity), so peers that only
/// know the element by the key it was minted as can still find it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub key: FractionalKey,
    /// Hybrid logical clock stamp of the last modification.
    pub modified_at: Hlc,
    /// If this [Entry] was generates as a move destination, this field will hold the source key
    /// that was moved. Otherwise, it will be [FractionalKey::NULL].
    ///
    /// For moved keys it's reverse: it will point to the destination. It can also point to
    /// [FractionalKey::NULL], which means that this entry has been removed.
    pub moved: FractionalKey,
}

impl Entry {
    pub fn moved(&self) -> Option<&FractionalKey> {
        if self.moved.is_empty() {
            None
        } else {
            Some(&self.moved)
        }
    }

    pub fn identity(&self) -> &FractionalKey {
        self.moved().unwrap_or(&self.key)
    }

    pub fn merge(&mut self, other: &Entry) -> bool {
        if self.key == other.key && self.modified_at < other.modified_at {
            self.modified_at = other.modified_at;
            self.moved = other.moved.clone();
            return true;
        }
        false
    }
}

/// A collection of [FractionalKey]s that enables producing them in a way that matches their desired
/// order.
#[derive(Debug, Clone, Default)]
pub struct FractionalIndex {
    /// Fractional keys, sorted by the position each currently occupies ([Entry::key]).
    /// This space contains un-moved elements, or destinations of moved elements.
    active: Vec<Entry>,
    /// Fractional keys, sorted by the position each currently occupies ([Entry::key]).
    /// This space contains moved and tombstoned elements.
    moved: Vec<Entry>,

    /// Suffix this replica mints new keys with. Never serialized — see [Self::decode].
    pub suffix: [u8; Self::SESSION_HASH_SIZE],

    /// Highest ordinal any [virtual_key] in this index has ever carried, cached rather than stored:
    /// it is exactly `max(virtual_ordinal(k))` over `active ∪ moved`, and it is derivable only
    /// because tombstones are permanent, so a key that was ever here is still in one of the two
    /// spaces. Nothing serializes it — every constructor recomputes it.
    watermark: u32,
}

/// Equality is over document state (`active`, `moved`). `suffix` is replica identity —
/// deliberately never serialized — and `watermark` is derived from the two spaces, so two
/// replicas holding the same document compare equal whatever session they mint with.
impl PartialEq for FractionalIndex {
    fn eq(&self, other: &Self) -> bool {
        self.active == other.active && self.moved == other.moved
    }
}
impl Eq for FractionalIndex {}

impl FractionalIndex {
    const SESSION_HASH_SIZE: usize = SESSION_SUFFIX_LEN;

    /// The longest position a [FractionalKey] can carry once its session suffix is appended.
    const MAX_POSITION_LEN: usize = crate::collab::codec::MAX_POSITION_LEN;

    pub fn new(
        active: Vec<Entry>,
        moved: Vec<Entry>,
        suffix: [u8; Self::SESSION_HASH_SIZE],
    ) -> Self {
        let watermark = Self::max_virtual_ordinal(&active).max(Self::max_virtual_ordinal(&moved));
        FractionalIndex {
            active,
            moved,
            suffix,
            watermark,
        }
    }

    fn max_virtual_ordinal(entries: &[Entry]) -> u32 {
        entries
            .iter()
            .filter_map(|e| virtual_ordinal(&e.key))
            .max()
            .unwrap_or(0)
    }

    /// Highest [virtual_key] ordinal this index has ever handed out. Never decreases.
    pub fn watermark(&self) -> u32 {
        self.watermark
    }

    /// The [virtual_key]s that have to be inserted before position `through` (1-based) is
    /// addressable, in order. Empty when the index already reaches that far.
    ///
    /// Ordinals continue past the watermark rather than filling gaps, so a virtual row that was
    /// deleted is never named again — its tombstone would swallow the insert. Reads only: the caller
    /// emits the keys as a patch and applies that.
    pub fn plan_virtual(&self, through: usize) -> Vec<FractionalKey> {
        let missing = through.saturating_sub(self.len());
        (1..=missing as u32)
            .map(|i| virtual_key(self.watermark + i))
            .collect()
    }

    /// The identity of every element, in the order they sit in.
    pub fn view(&self) -> impl Iterator<Item = &FractionalKey> {
        self.active.iter().filter_map(|e| match e.moved() {
            None => Some(&e.key),
            source => source,
        })
    }

    pub fn len(&self) -> usize {
        self.active.len()
    }

    pub fn get(&self, index: usize) -> Option<&Entry> {
        self.active.get(index)
    }

    pub fn key(&self, index: usize) -> Option<&FractionalKey> {
        let e = self.active.get(index)?;
        e.moved().or(Some(&e.key))
    }

    pub fn position_of(&self, identity: &FractionalKey) -> Option<usize> {
        match self.active.binary_search_by_key(&identity, |e| &e.key) {
            Ok(index) => Some(index),
            Err(_) => {
                let moved_index = self
                    .moved
                    .binary_search_by_key(&identity, |e| &e.key)
                    .ok()?;
                let e = &self.moved[moved_index];
                let dest = e.moved()?;
                self.position_of(dest)
            }
        }
    }

    pub fn move_to<R: RangeBounds<usize>>(&mut self, source: R, dest: usize) {
        let start = match source.start_bound() {
            Bound::Included(&i) => i,
            Bound::Excluded(&i) => i + 1,
            Bound::Unbounded => 0,
        };
        if start == dest {
            return; // no op
        }
        let to_move: Vec<_> = self.active.drain(source).collect();
        let len = to_move.len();
        // if start < dest, we need to shift by the number of drained entries
        let mut dest = if start < dest { dest - len } else { dest };

        let modified_at = Hlc::now();
        let mut key_gen = self.create_keys(dest, len);
        for mut entry in to_move {
            let dest_key = key_gen
                .next()
                .expect("destination is too crowded to create a fractional key for");
            // if the entry is transitive (shows a move destination), we want to update the source
            // key instead
            let moved_key = entry.moved().unwrap_or(&entry.key).clone();
            // insert new marker key into index
            self.active.insert(
                dest,
                Entry {
                    key: dest_key.clone(),
                    modified_at,
                    moved: moved_key.clone(),
                },
            );
            dest += 1;

            // move drained entry into moved space
            match entry.moved() {
                None => {
                    // this is the original key to be moved, and the record it leaves behind is the
                    // same whether or not one was already filed under it
                    entry.modified_at = modified_at;
                    entry.moved = dest_key;
                    self.park(entry);
                }
                Some(moved) => {
                    // this is a transitive entry, we need to tombstone it and update its source
                    if let Ok(i) = self.moved.binary_search_by_key(&moved, |e| &e.key) {
                        let e = &mut self.moved[i];
                        e.modified_at = modified_at;
                        e.moved = dest_key;
                    }
                    entry.modified_at = modified_at;
                    entry.moved = FractionalKey::NULL;
                    self.park(entry);
                }
            }
        }
    }

    pub fn remove_key(&mut self, key: &FractionalKey) -> Option<FractionalKey> {
        // found in index space: move to moved space
        if let Some(source) = self.tombstone(key, Some(Hlc::now())) {
            return if source.is_empty() {
                // the key was never moved, so it is the element's own identity
                Some(key.clone())
            } else {
                // it sat at a move destination, so the record it came from goes too — and that
                // record is what the caller knows the element by
                self.remove_key(&source)
            };
        }
        // not in index space, but possibly in moved space?
        match self.moved.binary_search_by_key(&key, |e| &e.key) {
            Ok(i) => {
                let removed_at = Hlc::now();
                let e = &mut self.moved[i];
                e.modified_at = removed_at;
                let moved = std::mem::replace(&mut e.moved, FractionalKey::NULL);
                if moved == FractionalKey::NULL {
                    // already removed
                    None
                } else {
                    // we removed the element that has been moved, we also need to remove
                    // its move destination
                    let removed_source = e.key.clone();
                    self.remove_key(&moved);
                    Some(removed_source)
                }
            }
            Err(_) => {
                None // we cannot remove key before it appeared
            }
        }
    }

    /// The *positions* of the stored keys bracketing index `i`, session suffix stripped. An empty
    /// slice means "unbounded on that side".
    ///
    /// Stripping is what [FractionalKey]'s two-part order asks for: positions are compared on their
    /// own, so a position generated strictly between these two slices sorts strictly between the two
    /// stored keys whatever session minted any of them. Comparing against the full stored keys does
    /// not work — the session bytes would join the comparison at a depth where they mean nothing.
    ///
    /// Generated positions are never empty, so an empty slice is unambiguous.
    fn neighbours(index: &[Entry], i: usize) -> Option<(&[u8], &[u8])> {
        if i > index.len() {
            return None;
        }
        let left = if i == 0 {
            [].as_ref()
        } else {
            index[i - 1].key.position()
        };
        let right = index.get(i).map(|e| e.key.position()).unwrap_or(&[]);
        Some((left, right))
    }

    /// Generate a new [FractionalKey] that matches a given index. Return that key and an alias to it.
    ///
    /// Returns `None` if `index` is out of range, or if the generated key would exceed
    /// [MAX_KEY_LEN] — the position is too crowded to name.
    pub fn create_key(&mut self, index: usize) -> Option<&FractionalKey> {
        let (left, right) = Self::neighbours(&self.active, index)?;
        let mut buf = Self::create_fractional_key(left, right, InsertStrategy::Middle);
        buf.extend_from_slice(self.suffix.as_ref());
        let key = FractionalKey::try_from_bytes(&buf).ok()?;
        // A fresh key is both the element's position and its identity, so it carries no origin.
        let modified_at = Hlc::now();
        self.active.insert(
            index,
            Entry {
                key,
                modified_at,
                moved: FractionalKey::NULL,
            },
        );
        Some(&self.active[index].key)
    }

    /// Insert an externally minted `key` at its sorted position.
    ///
    /// Returns `None` if the key was *ever* seen — active or parked — which is what makes the index a
    /// 2P-set: a tombstone is final, so a delete always wins over a concurrent insert of the same key
    /// and two peers minting the same key converge on one element.
    ///
    /// `modified_at` is stamped from the local clock. It is advisory (nothing here resolves
    /// conflicts by it), so peers disagreeing on it is not divergence.
    pub fn insert_key(&mut self, key: FractionalKey) -> Option<usize> {
        // Before the ever-seen check: a rejected key was still handed out, and its ordinal must not
        // be minted again.
        if let Some(ordinal) = virtual_ordinal(&key) {
            self.watermark = self.watermark.max(ordinal);
        }
        if self.active.binary_search_by_key(&&key, |e| &e.key).is_ok()
            || self.moved.binary_search_by_key(&&key, |e| &e.key).is_ok()
        {
            return None;
        }
        let index = self.lower_bound(&key);
        self.active.insert(
            index,
            Entry {
                key,
                modified_at: Hlc::now(),
                moved: FractionalKey::NULL,
            },
        );
        Some(index)
    }

    /// Where `key` would sit among the active entries, whether or not it is one of them.
    pub fn lower_bound(&self, key: &FractionalKey) -> usize {
        match self.active.binary_search_by_key(&key, |e| &e.key) {
            Ok(i) | Err(i) => i,
        }
    }

    /// Returns an iterator yielding a run of `count` new [FractionalKey]s for the gap at `start`,
    /// one per [Iterator::next] call. It reads the index but does not touch it — the caller inserts
    /// whatever it takes.
    ///
    /// The run is planned against `count` up front, so the keys come out as short as the gap allows
    /// (see [CreateKeys]). It yields fewer than `count` keys only when the gap cannot hold them.
    pub fn create_keys(&self, start: usize, count: usize) -> CreateKeys {
        CreateKeys::new(self, start, count)
    }

    /// Creates a new key payload which is lexically higher than `lo` and lower than `hi`. Returned
    /// as a [KeyBuf] because callers still have the session suffix to append before sealing it into
    /// a [FractionalKey].
    fn create_fractional_key(lo: &[u8], hi: &[u8], strategy: InsertStrategy) -> KeyBuf {
        let mut key = KeyBuf::new();
        // Once we place a digit strictly below `hi`, the rest of `hi` no longer constrains us and
        // the upper bound effectively becomes +infinity (digit 256).
        let mut hi_unbounded = hi.is_empty();
        let mut i = 0;
        loop {
            // `lo` shorter than the key we build is padded with zeroes (its smallest continuation).
            let lo_digit = lo.get(i).copied().unwrap_or(0) as u16;
            let hi_digit = if hi_unbounded { 256 } else { hi[i] as u16 };

            if hi_digit - lo_digit >= 2 {
                // There is room for a digit strictly between the two bounds: pick the midpoint.
                key.push(strategy.next_byte(lo_digit, hi_digit));
                return key;
            }

            // The digits are equal (shared prefix) or adjacent (no room here): keep the `lo` digit
            // and descend into the next position.
            key.push(lo_digit as u8);
            if hi_digit != lo_digit {
                // We just placed a digit below `hi`, so the upper bound stops constraining us.
                hi_unbounded = true;
            }
            i += 1;
        }
    }

    /// Every element as an [IterEntry]: the identity it answers to, and where it has been moved to
    /// if it no longer sits there. This is the payload a peer replays through [Self::merge].
    pub fn iter(&self) -> impl Iterator<Item = &Entry> + '_ {
        self.active.iter()
    }

    pub fn merge(&mut self, other: &Self) -> bool {
        let mut changed = false;
        let mut o1 = 0;
        let mut o2 = 0;
        for e in other.active.iter() {
            let index = &self.active[o1..];
            match index.binary_search_by_key(&&e.key, |e| &e.key) {
                Ok(i) => {
                    // key found in current index space
                    o1 += i;
                    let e2 = &mut self.active[o1];
                    if e2.modified_at <= e.modified_at {
                        e2.modified_at = e.modified_at;
                        // if two entries have the same modified_at value, we'll resolve the result
                        // by higher moved key
                        e2.moved = (&e2.moved).max(&e.moved).clone();
                        changed = true;
                    }
                }
                Err(i) => {
                    // key not found in current index space, search moved space first
                    let moved = &self.moved[o2..];
                    match moved.binary_search_by_key(&&e.key, |e| &e.key) {
                        Ok(j) => {
                            o2 += j;
                            // `e` existed in `other.index` and on `self.moved`: since we key can
                            // be (re)moved only AFTER it appeared, it means that other has outdated update
                        }
                        Err(j) => {
                            o1 += i;
                            o2 += j;
                            // key didn't exist in moved space either, we can insert it into index
                            self.active.insert(o1, e.clone());
                            changed = true;
                        }
                    }
                }
            }
        }

        let mut o1 = 0;
        let mut o2 = 0;
        for e in other.moved.iter() {
            let index = &self.active[o1..];
            match index.binary_search_by_key(&&e.key, |e| &e.key) {
                Ok(i) => {
                    // moved key found in current index space, it has to be moved
                    o1 += i;
                    let removed = self.active.remove(o1);
                    if let Some(moved) = removed.moved() {
                        // this key is transitive - it's move destination of another key
                    }

                    let moved = &self.moved[o2..];
                    match moved.binary_search_by_key(&&e.key, |e| &e.key) {
                        Ok(i) => {
                            // technically this shouldn't happen (the same key should never
                            // be present in both index and moved spaces)
                            o2 += i;
                            let e2 = &mut self.moved[o2];
                            // Same pairwise last-writer-wins as below, for the same reason.
                            if (e2.modified_at, &e2.moved) < (e.modified_at, &e.moved) {
                                e2.modified_at = e.modified_at;
                                e2.moved = e.moved.clone();
                                changed = true;
                            }
                        }
                        Err(i) => {
                            o2 += i;
                            self.moved.insert(o2, e.clone());
                        }
                    }
                }
                Err(_) => {
                    // not found in regular index space, try moved space
                    let moved = &self.moved[o2..];
                    match moved.binary_search_by_key(&&e.key, |e| &e.key) {
                        Ok(i) => {
                            o2 += i;
                            let e2 = &mut self.moved[o2];
                            // Last writer wins, the destination key breaking a tie. The two have to
                            // be compared as one pair: taking the newer timestamp but then the
                            // *higher* key lets a newer-but-lower record win here and lose on the
                            // peer that sent it, and the two never converge.
                            let loser = if (e2.modified_at, &e2.moved) < (e.modified_at, &e.moved) {
                                e2.modified_at = e.modified_at;
                                changed = true;
                                std::mem::replace(&mut e2.moved, e.moved.clone())
                            } else {
                                e.moved.clone()
                            };
                            let winner = e2.moved.clone();
                            // Two peers moving one element mint a destination each, and neither
                            // payload mentions the other's — so the loop above read the incoming one
                            // as a position nobody had heard of and inserted it alongside ours. Both
                            // now claim this identity and the element shows up twice in `view`.
                            if loser != winner
                                && !loser.is_empty()
                                && self.tombstone(&loser, None).is_some()
                            {
                                // The offsets only narrow the searches, and an entry leaving `index`
                                // invalidates them. Conflicts are rare enough that widening back to
                                // the whole vector costs nothing measurable.
                                o1 = 0;
                                o2 = 0;
                                changed = true;
                            }
                        }
                        Err(i) => {
                            o2 += i;
                            self.moved.insert(o2, e.clone());
                        }
                    }
                }
            }
        }
        // A merge brings in keys nobody here minted, so the watermark is re-derived from the two
        // spaces it is defined over. It can only ever grow.
        self.watermark = Self::max_virtual_ordinal(&self.active)
            .max(Self::max_virtual_ordinal(&self.moved))
            .max(self.watermark);
        changed
    }

    /// Drops `key` from the active index and parks it in `moved` under [FractionalKey::NULL] — the
    /// same tombstone [Self::move_to] leaves for a destination it supersedes.
    ///
    /// Returns the origin the entry carried, [FractionalKey::NULL] for one that was never moved, or
    /// `None` if `key` held no position at all. The two are worth telling apart: an origin names a
    /// record in `moved` space that still points here and has to be dealt with in turn, where `None`
    /// means there was nothing to drop in the first place.
    ///
    /// `at` is what the parked record gets stamped with. A local edit passes a fresh [`Hlc`]; a merge
    /// passes `None` to keep the timestamp the entry already carried, since a merge that read the
    /// clock would land on different state on every peer.
    fn tombstone(&mut self, key: &FractionalKey, at: Option<Hlc>) -> Option<FractionalKey> {
        let Ok(i) = self.active.binary_search_by_key(&key, |e| &e.key) else {
            return None;
        };
        let mut e = self.active.remove(i);
        if let Some(at) = at {
            e.modified_at = at;
        }
        // Whatever identity it carried belongs to whatever superseded this position.
        let origin = std::mem::replace(&mut e.moved, FractionalKey::NULL);
        self.park(e);
        Some(origin)
    }

    /// Files `e` in `moved` space, replacing whatever was already filed under its key.
    ///
    /// `moved` is kept sorted by key — every lookup in here binary-searches it — so a record goes
    /// where the search says it goes, not on the end.
    fn park(&mut self, e: Entry) {
        match self.moved.binary_search_by_key(&&e.key, |o| &o.key) {
            Ok(i) => self.moved[i] = e,
            Err(i) => self.moved.insert(i, e),
        }
    }

    /// The whole index as bytes: a version byte, then each space as a column table.
    ///
    /// The two spaces are written back to back and read back the same way, so the split between them
    /// costs nothing beyond each table's own entry count. See [codec](crate::collab::codec) for the
    /// layout and for why an index is worth encoding by column rather than by entry.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        let mut out = Vec::new();
        out.push(FORMAT_VERSION);
        encode_entries(&self.active, &mut out)?;
        encode_entries(&self.moved, &mut out)?;
        Ok(out)
    }

    /// Reads back what [Self::encode] wrote.
    ///
    /// `suffix` is the **local** session's, not the one that wrote the bytes: it is what this replica
    /// will mint new keys with, and it has to differ from every peer's or two replicas generate
    /// colliding keys. Nothing in the payload names it, deliberately — a snapshot is restored by
    /// whichever replica loads it, and it is that replica's identity that matters.
    pub fn decode(bytes: &[u8], suffix: [u8; Self::SESSION_HASH_SIZE]) -> Result<Self, CodecError> {
        let (&version, mut input) = bytes.split_first().ok_or(CodecError::UnexpectedEof)?;
        if version != FORMAT_VERSION {
            return Err(CodecError::UnsupportedVersion(version));
        }
        let active = decode_entries(&mut input)?;
        let moved = decode_entries(&mut input)?;
        if !input.is_empty() {
            return Err(CodecError::TrailingBytes(input.len()));
        }
        // Through `new`, so the watermark is re-derived rather than carried in the payload.
        Ok(FractionalIndex::new(active, moved, suffix))
    }
}

/// The columnar encoding is the representation in *every* serde format, human-readable ones
/// included: an index is bulk machine state, and there is no readable rendering of a few dozen bytes
/// standing in for ten thousand keys that would be worth a second code path to maintain.
///
/// [Entry] keeps its own derive, so a legible dump of an index's contents is still a `{:?}` or a
/// `serde_json::to_string` over [FractionalIndex::iter] away.
impl Serialize for FractionalIndex {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let bytes = self.encode().map_err(serde::ser::Error::custom)?;
        serializer.serialize_bytes(&bytes)
    }
}

impl<'de> Deserialize<'de> for FractionalIndex {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_bytes(FractionalIndexVisitor)
    }
}

/// Accepts every shape a format may hand the payload over in: borrowed bytes, an owned buffer, or a
/// sequence of `u8`. Which one it is depends on the format — bitcode calls `visit_bytes`, others hand
/// over an owned buffer, and a format without native byte support (serde_json among them) renders
/// `serialize_bytes` as a sequence.
///
/// The session suffix is not in the payload, so a deserialized index comes back with a zeroed one,
/// exactly as it did under `#[serde(skip)]`. A replica that means to *mint* keys through the index
/// has to come in through [FractionalIndex::decode] with its own session instead.
struct FractionalIndexVisitor;

impl<'de> Visitor<'de> for FractionalIndexVisitor {
    type Value = FractionalIndex;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "an encoded fractional index, as a byte sequence")
    }

    fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        FractionalIndex::decode(v, Default::default()).map_err(E::custom)
    }

    fn visit_byte_buf<E: serde::de::Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
        self.visit_bytes(&v)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut bytes = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(b) = seq.next_element::<u8>()? {
            bytes.push(b);
        }
        self.visit_bytes(&bytes)
    }
}

/// bitcode coders delegating to [FractionalIndex::encode]/[FractionalIndex::decode], the same
/// columnar payload the serde impls carry — written by hand for the same reason as
/// [`FractionalKeyEncoder`](crate::collab::fractional_key::FractionalKeyEncoder), and coming back
/// with a zeroed session suffix for the same reason too.
#[derive(Default)]
pub struct FractionalIndexEncoder(<[u8] as Encode>::Encoder);

impl Buffer for FractionalIndexEncoder {
    fn collect_into(&mut self, out: &mut Vec<u8>) {
        self.0.collect_into(out);
    }

    fn reserve(&mut self, additional: NonZeroUsize) {
        self.0.reserve(additional);
    }
}

impl Encoder<FractionalIndex> for FractionalIndexEncoder {
    #[inline]
    fn encode(&mut self, t: &FractionalIndex) {
        let bytes = t.encode().expect("fractional index cannot be encoded");
        Encoder::<[u8]>::encode(&mut self.0, &bytes);
    }
}

impl Encode for FractionalIndex {
    type Encoder = FractionalIndexEncoder;
}

#[derive(Default)]
pub struct FractionalIndexDecoder<'a>(<Vec<u8> as Decode<'a>>::Decoder);

impl<'a> View<'a> for FractionalIndexDecoder<'a> {
    fn populate(&mut self, input: &mut &'a [u8], length: usize) -> bitcode::__private::Result<()> {
        self.0.populate(input, length)
    }
}

impl<'a> Decoder<'a, FractionalIndex> for FractionalIndexDecoder<'a> {
    /// # Panics
    ///
    /// On a malformed payload: bitcode wants all validation in [`View::populate`], which cannot see
    /// the entry tables. Untrusted input should come in through `serde`, which reports the error.
    #[inline]
    fn decode(&mut self) -> FractionalIndex {
        let bytes: Vec<u8> = self.0.decode();
        FractionalIndex::decode(&bytes, Default::default()).expect("malformed bitcode payload")
    }
}

impl<'a> Decode<'a> for FractionalIndex {
    type Decoder = FractionalIndexDecoder<'a>;
}

enum InsertStrategy {
    Middle,
}

impl InsertStrategy {
    #[inline]
    pub fn next_byte(&self, lo: u16, hi: u16) -> u8 {
        match self {
            InsertStrategy::Middle => (lo + (hi - lo) / 2) as u8,
        }
    }
}

/// How many positions of exactly `width` digits sort strictly between `lo` and `hi`, an empty `hi`
/// meaning unbounded. Saturates once the answer is far past any run length.
///
/// A run reads its gap as a range of integers: `lo` and `hi` padded (or truncated) to `width` digits
/// and read base-256 as `L` and `H`, with `H = 256^width` when unbounded. Every width-`width` `X`
/// with `L < X < H` sorts strictly between `lo` and `hi`, whichever bound is longer or shorter than
/// the width. Strictness is load-bearing at the bottom end: it drops `X = lo ++ [0x00…]`, and since
/// nothing sorts between `p` and `p ++ [0x00]`, emitting that would seal off the gap above it.
///
/// Capacity never shrinks as `width` grows, which is what makes "the narrowest width that fits" well
/// defined: with `D = H - L`, each further digit gives `D' = 256·D + (hi_digit - lo_digit)`, so a
/// positive `D` stays positive and a negative one stays negative.
fn gap_capacity(lo: &[u8], hi: &[u8], width: usize) -> u128 {
    let unbounded = hi.is_empty();
    let mut diff: i128 = 0;
    for i in 0..width {
        // `256^width - 1` is `[0xff; width]`, so an unbounded `hi` is all-`0xff` digits with the
        // missing `+ 1` folded into the result below.
        let hi_digit = if unbounded {
            0xff
        } else {
            hi.get(i).copied().unwrap_or(0) as i128
        };
        let lo_digit = lo.get(i).copied().unwrap_or(0) as i128;
        diff = (diff * 256 + hi_digit - lo_digit).min(u64::MAX as i128);
        if diff < 0 {
            return 0; // `lo` sorts above `hi`: not a gap, at this width or any wider one
        }
    }
    // A bounded `hi` is a position itself and is excluded; an unbounded one is `[0xff; width] + 1`,
    // so its topmost position is already counted.
    (if unbounded { diff } else { diff - 1 }).max(0) as u128
}

/// Adds `delta` to a big-endian position in place. The width is picked to hold the whole run, so a
/// carry running off the front is a bug rather than something to handle.
fn add_assign(position: &mut [u8], delta: u8) {
    let mut carry = delta as u16;
    for digit in position.iter_mut().rev() {
        if carry == 0 {
            return;
        }
        let sum = *digit as u16 + carry;
        *digit = sum as u8;
        carry = sum >> 8;
    }
}

/// Iterator produced by [FractionalIndex::create_keys]: a run of new [FractionalKey]s for one gap,
/// minted lazily but planned eagerly.
///
/// Knowing the run length up front is what keeps the keys short. The width that holds the whole run
/// is picked once, and the run is then a fixed-width counter over it — so every key in a run is the
/// same length, and whether the run stays inline is settled before the first [Iterator::next]
/// instead of being discovered part-way through it.
pub struct CreateKeys {
    suffix: [u8; FractionalIndex::SESSION_HASH_SIZE],
    /// The position to yield next, as many digits as the run was planned for.
    next: KeyBuf,
    /// Distance between consecutive positions of the run.
    stride: u8,
    remaining: usize,
}

impl CreateKeys {
    /// Width [CreateKeys] pads its positions out to.
    ///
    /// A position only counts up to `0xff` per byte before it has to grow, so a narrow one runs out of
    /// room fast — but padding is free up to this point, since a key stays a single word until the
    /// position plus its session suffix outgrows [INLINE_CAP]. Three bytes hold ~16.7M positions in one
    /// gap; one byte holds 255.
    const RUN_WIDTH: usize = INLINE_CAP - SESSION_SUFFIX_LEN;

    fn new(index: &FractionalIndex, i: usize, count: usize) -> Self {
        let (lo, hi) = FractionalIndex::neighbours(&index.active, i)
            .expect("cannot mint fractional keys past the end of the index");
        // A tombstone sitting in the gap raises its floor. The index is a 2P-set, so a key it has
        // ever seen can never be inserted again — re-minting one would yield a key that silently
        // does nothing, which is exactly what undoing a delete must not produce.
        let lo = index.moved.iter().fold(lo, |floor, e| {
            let position = e.key.position();
            if position > floor && (hi.is_empty() || position < hi) {
                position
            } else {
                floor
            }
        });
        // A gap without room for the run leaves `remaining` at zero, so the iterator ends where the
        // room does rather than minting keys that do not fit in it.
        let (next, stride, remaining) = match Self::plan(lo, hi, count) {
            Some((next, stride)) => (next, stride, count),
            None => (KeyBuf::new(), 1, 0),
        };
        CreateKeys {
            suffix: index.suffix,
            next,
            stride,
            remaining,
        }
    }

    /// The first position and stride of a run of `count` keys between `lo` and `hi`, or `None` if no
    /// width up to [MAX_POSITION_LEN] has room for it.
    fn plan(lo: &[u8], hi: &[u8], count: usize) -> Option<(KeyBuf, u8)> {
        // Capacity grows with width, so the first width that fits is the narrowest one. Widening
        // past it to [RUN_WIDTH] is free — the key is one word either way — and pays for the stride.
        let width = (1..=FractionalIndex::MAX_POSITION_LEN)
            .find(|&width| gap_capacity(lo, hi, width) >= count as u128)?
            .max(Self::RUN_WIDTH);
        // A stride of 2 leaves a free position between consecutive keys, so a later insert between
        // two of them lands at this same width — which at [RUN_WIDTH] is the difference between an
        // inline key and a heap allocation.
        let stride = if gap_capacity(lo, hi, width) >= 2 * count as u128 + 1 {
            2
        } else {
            1
        };
        // The run starts one stride above the gap's floor and ends `count` strides later, which is
        // exactly what the width was chosen to accommodate.
        let mut next = KeyBuf::from(&lo[..lo.len().min(width)]);
        next.resize(width, 0);
        add_assign(&mut next, stride);
        Some((next, stride))
    }
}

impl Iterator for CreateKeys {
    type Item = FractionalKey;

    /// Yields `None` once the planned run is used up — immediately, for a gap with no room for it.
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        let mut buf = self.next.clone();
        buf.extend_from_slice(self.suffix.as_ref());

        add_assign(&mut self.next, self.stride);

        // Infallible: the width is capped so that the suffix still fits within [MAX_KEY_LEN].
        FractionalKey::try_from_bytes(&buf).ok()
    }
}

#[cfg(test)]
mod test {
    use super::{
        gap_capacity, virtual_key, virtual_ordinal, Entry, FractionalIndex, FractionalKey, Hlc,
    };
    use std::collections::HashSet;

    /// The identity of every entry, in stored order. An identity is stable per element across a
    /// `move_to`, so this tracks element movement *within* a single [FractionalIndex].
    fn identity_order(fi: &FractionalIndex) -> Vec<FractionalKey> {
        fi.view().cloned().collect()
    }

    #[test]
    fn fractional_index() {
        let mut fi = FractionalIndex::default();

        let k2 = fi.create_key(0).unwrap().clone(); // [.]
        let k1 = fi.create_key(0).unwrap().clone(); // [. k2]
        let k4 = fi.create_key(2).unwrap().clone(); // [k1 k2 .]
        let k3 = fi.create_key(2).unwrap().clone(); // [k1 k2 . k4]

        let expected = vec![k1, k2, k3, k4];
        let mut actual: Vec<_> = fi.active.iter().map(|e| e.key.clone()).collect();
        assert_eq!(actual, expected);

        // entries should be already sorted
        actual.sort();
        assert_eq!(actual, expected);

        // every key is distinct => no dedups
        let keys: HashSet<_> = fi.active.iter().map(|e| e.key.clone()).collect();
        assert_eq!(keys.len(), fi.len());
    }

    #[test]
    fn fractional_indexes() {
        let mut fi = FractionalIndex::default();

        let k1 = fi.create_key(0).unwrap().clone(); // [.]
        let k2 = fi.create_key(1).unwrap().clone(); // [k1 .]

        let keys: Vec<_> = fi.create_keys(1, 2_000_000).collect();

        let mut last = &k1;
        for k in keys.iter() {
            assert!(k > last, "{k:?} should be higher than previous one");
            assert!(k < &k2, "{k:?} should be lower than the upper boundary");
            assert!(k.is_inline(), "{k:?} should be inlined");
            last = k;
        }
    }

    /// The integer model a run is planned against: how many positions of a given width sort strictly
    /// between two bounds.
    #[test]
    fn gap_capacity_counts_positions_at_each_width() {
        // A bounded gap holds 0x81..=0xbf at one digit, and 256x as many per digit added.
        assert_eq!(gap_capacity(&[0x80], &[0xc0], 1), 63);
        assert_eq!(gap_capacity(&[0x80], &[0xc0], 2), 16383);
        assert_eq!(gap_capacity(&[0x80], &[0xc0], 3), 4194303);
        // An unbounded one runs to the top of the width instead.
        assert_eq!(gap_capacity(&[0xe0], &[], 1), 31);
        assert_eq!(gap_capacity(&[0xe0], &[], 3), 0xff_ffff - 0xe0_0000);
        // Adjacent digits leave no room until a digit has been spent on the shared prefix.
        assert_eq!(gap_capacity(&[0x05, 0x03], &[0x05, 0x04], 2), 0);
        assert_eq!(gap_capacity(&[0x05, 0x03], &[0x05, 0x04], 3), 255);
        // A `hi` shorter than the width still bounds it, and `lo` extended by zeroes is excluded —
        // hence 0 rather than 1 at width 2.
        assert_eq!(gap_capacity(&[], &[0x00, 0x01], 2), 0);
        assert_eq!(gap_capacity(&[], &[0x00, 0x01], 3), 255);
        // Equal bounds (two sessions, one position) never open up, however wide the position gets.
        assert_eq!(
            gap_capacity(&[0x05], &[0x05], FractionalIndex::MAX_POSITION_LEN),
            0
        );
        // Nor do inverted ones.
        assert_eq!(gap_capacity(&[0x07], &[0x05], 4), 0);
    }

    /// A `move_to` relocates an element: it must appear at its new position and no longer be visible
    /// at its old one, and it must not be duplicated.
    #[test]
    fn move_to_relocates_element() {
        let mut fi = FractionalIndex::default();
        let a = fi.create_key(0).unwrap().clone(); // [A]
        let b = fi.create_key(1).unwrap().clone(); // [A B]
        let c = fi.create_key(2).unwrap().clone(); // [A B C]

        // Move B (at index 1) to the end.
        fi.move_to(1..2, 3);

        // New order is [A C B]: B is at the end...
        assert_eq!(identity_order(&fi), vec![a, c.clone(), b.clone()]);
        // ...its old position (index 1) is now C, so B is gone from there...
        assert_eq!(fi.get(1).unwrap().identity(), &c);
        // ...and B exists exactly once (no duplication) with the count unchanged.
        assert_eq!(fi.len(), 3);
        assert_eq!(fi.active.iter().filter(|e| e.identity() == &b).count(), 1);
        // It answers to the key it was minted as, not to the one the move gave it.
        assert_eq!(fi.position_of(&b), Some(2));
        assert_ne!(fi.get(2).unwrap().key, b);
    }

    /// A second move must not overwrite the identity the first one preserved.
    #[test]
    fn identity_survives_repeated_moves() {
        let mut fi = FractionalIndex::default();
        let a = fi.create_key(0).unwrap().clone(); // [A]
        let b = fi.create_key(1).unwrap().clone(); // [A B]
        let c = fi.create_key(2).unwrap().clone(); // [A B C]

        fi.move_to(1..2, 3); // [A C B]
        fi.move_to(2..3, 0); // [B A C]

        assert_eq!(identity_order(&fi), vec![b.clone(), a, c]);
        assert_eq!(fi.get(0).unwrap().identity(), &b);
        assert_ne!(fi.get(0).unwrap().key, b);
    }

    #[test]
    fn remove_drops_element() {
        let mut fi = FractionalIndex::default();
        let a = fi.create_key(0).unwrap().clone(); // [A]
        let b = fi.create_key(1).unwrap().clone(); // [A B]
        let c = fi.create_key(2).unwrap().clone(); // [A B C]

        // Remove an unmoved element from the middle: it answers to its own key.
        assert_eq!(fi.remove_key(&b), Some(b.clone()));
        assert_eq!(fi.len(), 2);
        assert_eq!(identity_order(&fi), vec![a.clone(), c.clone()]);
        // It is gone, and cannot be removed twice.
        assert_eq!(fi.position_of(&b), None);
        assert_eq!(fi.remove_key(&b), None);

        // Move C to the front, then remove it by its *identity* key (not its new position key).
        fi.move_to(1..2, 0); // view: [C A]
        assert_eq!(identity_order(&fi), vec![c.clone(), a.clone()]);
        assert_eq!(fi.remove_key(&c), Some(c.clone()));
        assert_eq!(fi.len(), 1);
        assert_eq!(identity_order(&fi), vec![a]);
        // Its move left nothing behind.
        assert_eq!(fi.position_of(&c), None);
    }

    #[test]
    fn concurrent_move_of_same_element_converges() {
        // Distinct session suffixes => the two peers never generate colliding keys.
        let mut a = FractionalIndex::new(vec![], vec![], [b'a', 0, 0, 0]);
        let mut b = FractionalIndex::new(vec![], vec![], [b'b', 0, 0, 0]);

        // Shared initial list [0 x1 x2]: A creates it, B replicates via the merge payload.
        let k0 = a.create_key(0).unwrap().clone();
        let k1 = a.create_key(1).unwrap().clone();
        let k2 = a.create_key(2).unwrap().clone();
        b.merge(&a);
        let view_a: Vec<_> = a.view().collect();
        let view_b: Vec<_> = b.view().collect();
        assert_eq!(view_a, view_b, "peers should start from an identical view");
        assert_eq!(view_a, vec![&k0, &k1, &k2]);

        // Both concurrently move the same element (born as k1) to different destinations.
        a.move_to(1..2, 3); // A moves it to the end
        b.move_to(1..2, 0); // B moves it to the front

        // Exchange full payloads (snapshot both before applying, so order of application is moot).
        let pa = a.clone();
        let pb = b.clone();
        a.merge(&pb);
        b.merge(&pa);

        // Consistency: both peers resolve to the exact same view.
        let view_a: Vec<_> = a.view().collect();
        let view_b: Vec<_> = b.view().collect();
        assert_eq!(view_a, view_b, "peers must converge to the same view");
        // No duplication: element count is unchanged and k1 appears exactly once.
        assert_eq!(a.len(), 3, "moved element must not be duplicated on peer A");
        assert_eq!(b.len(), 3, "moved element must not be duplicated on peer B");
        assert_eq!(a.view().filter(|k| **k == k1).count(), 1);
    }

    #[test]
    fn encode_roundtrips_an_index_with_both_spaces_populated() {
        let suffix = [b'z', 1, 2, 3];
        let mut fi = FractionalIndex::new(vec![], vec![], suffix);
        let a = fi.create_key(0).unwrap().clone();
        let b = fi.create_key(1).unwrap().clone();
        let c = fi.create_key(2).unwrap().clone();
        let d = fi.create_key(3).unwrap().clone();

        fi.move_to(1..3, 4); // parks b and c, minting a destination for each
        fi.remove_key(&d); // tombstones d
        assert!(
            !fi.moved.is_empty(),
            "this test is only meaningful with a populated moved space"
        );

        let bytes = fi.encode().unwrap();
        let decoded = FractionalIndex::decode(&bytes, suffix).unwrap();
        assert_eq!(decoded, fi);
        // Canonical: re-encoding what we just read reproduces the same bytes.
        assert_eq!(decoded.encode().unwrap(), bytes);
        // Functionally identical, not just structurally equal.
        assert_eq!(
            decoded.view().collect::<Vec<_>>(),
            fi.view().collect::<Vec<_>>()
        );
        for key in [&a, &b, &c, &d] {
            assert_eq!(decoded.position_of(key), fi.position_of(key));
        }
    }

    /// The bitcode derive path, which is how an index reaches storage inside a `Worksheet`. The
    /// session suffix is not in the payload, so a decoded index comes back with a zeroed one.
    #[test]
    fn bitcode_derive_roundtrips() {
        let mut fi = FractionalIndex::new(vec![], vec![], [b'z', 1, 2, 3]);
        let a = fi.create_key(0).unwrap().clone();
        fi.create_key(1);
        fi.move_to(0..1, 2);

        let decoded: FractionalIndex = bitcode::decode(&bitcode::encode(&fi)).unwrap();
        assert_eq!(decoded.suffix, [0; 4]);
        assert_eq!(
            decoded.view().collect::<Vec<_>>(),
            fi.view().collect::<Vec<_>>()
        );
        assert_eq!(decoded.position_of(&a), fi.position_of(&a));
    }

    /// What the column layout is for: a sheet's worth of rows costs bytes, not kilobytes.
    #[test]
    fn a_sheet_sized_index_encodes_compactly() {
        let mut fi = FractionalIndex::new(vec![], vec![], [b'a', 0, 0, 0]);
        // A single bulk run, as an import or a large paste produces.
        let keys: Vec<_> = fi.create_keys(0, 10_000).collect();
        assert_eq!(keys.len(), 10_000);
        for (i, key) in keys.into_iter().enumerate() {
            fi.active.insert(
                i,
                Entry {
                    key,
                    modified_at: Hlc::new(1_700_000_000_000 << 16),
                    moved: FractionalKey::NULL,
                },
            );
        }

        let bytes = fi.encode().unwrap();
        assert!(
            bytes.len() < 64,
            "10k rows should encode in a few dozen bytes, got {}",
            bytes.len()
        );
        assert_eq!(FractionalIndex::decode(&bytes, fi.suffix).unwrap(), fi);
        // bitcode adds only its own framing on top.
        assert!(bitcode::serialize(&fi).unwrap().len() < 128);
    }

    #[test]
    fn insert_key_places_sorted_and_never_resurrects() {
        let mut fi = FractionalIndex::new(vec![], vec![], [b'a', 0, 0, 0]);
        let (v1, v2, v3) = (virtual_key(1), virtual_key(2), virtual_key(3));

        // Sorted placement, whatever order they arrive in.
        assert_eq!(fi.insert_key(v3.clone()), Some(0));
        assert_eq!(fi.insert_key(v1.clone()), Some(0));
        assert_eq!(fi.insert_key(v2.clone()), Some(1));
        assert_eq!(
            fi.view().cloned().collect::<Vec<_>>(),
            vec![v1, v2.clone(), v3]
        );

        // Idempotent: a key already active is not inserted twice.
        assert_eq!(fi.insert_key(v2.clone()), None);
        assert_eq!(fi.len(), 3);

        // A tombstone is final: the key can never come back.
        assert_eq!(fi.remove_key(&v2), Some(v2.clone()));
        assert_eq!(fi.insert_key(v2.clone()), None);
        assert_eq!(fi.len(), 2);
    }

    #[test]
    fn virtual_keys_roundtrip_and_bracket_minted_ones() {
        for n in [1u32, 2, 7, 1_000, 1_048_576] {
            assert_eq!(virtual_ordinal(&virtual_key(n)), Some(n));
        }
        // Session-minted keys are not virtual.
        let mut fi = FractionalIndex::new(vec![], vec![], [b'a', 0, 0, 0]);
        fi.insert_key(virtual_key(1));
        fi.insert_key(virtual_key(2));
        let k = fi.create_keys(1, 1).next().unwrap();
        assert_eq!(virtual_ordinal(&k), None);
        // ...and they sort strictly inside the gap they were minted for.
        assert!(virtual_key(1) < k && k < virtual_key(2));
    }

    /// The watermark is derived state, so it has to come back with the bytes rather than in them.
    #[test]
    fn watermark_survives_a_roundtrip_and_a_tombstone() {
        let suffix = [b'a', 0, 0, 0];
        let mut fi = FractionalIndex::new(vec![], vec![], suffix);
        assert_eq!(fi.watermark(), 0);
        for n in 1..=3 {
            fi.insert_key(virtual_key(n));
        }
        assert_eq!(fi.watermark(), 3);

        let decoded = FractionalIndex::decode(&fi.encode().unwrap(), suffix).unwrap();
        assert_eq!(decoded.watermark(), 3);
        assert_eq!(decoded, fi);

        // Deleting a virtual row hands nothing back: its ordinal stays spent, so the next plan
        // starts above it and cannot run into the tombstone.
        fi.remove_key(&virtual_key(2));
        assert_eq!(fi.watermark(), 3);
        assert_eq!(fi.len(), 2);
        assert_eq!(fi.plan_virtual(3), vec![virtual_key(4)]);
        // The tombstone is still in `moved`, so the watermark is re-derivable from the two spaces.
        assert_eq!(
            FractionalIndex::decode(&fi.encode().unwrap(), suffix)
                .unwrap()
                .watermark(),
            3
        );
    }

    #[test]
    fn plan_virtual_extends_only_as_far_as_needed() {
        let mut fi = FractionalIndex::new(vec![], vec![], [b'a', 0, 0, 0]);
        assert_eq!(
            fi.plan_virtual(3),
            (1..=3).map(virtual_key).collect::<Vec<_>>()
        );
        for key in fi.plan_virtual(3) {
            fi.insert_key(key);
        }
        // Already long enough: nothing to mint.
        assert!(fi.plan_virtual(3).is_empty());
        assert!(fi.plan_virtual(0).is_empty());
        // Partially filled: only the ordinals past the watermark.
        assert_eq!(fi.plan_virtual(5), vec![virtual_key(4), virtual_key(5)]);
    }

    #[test]
    fn lower_bound_finds_the_insertion_point() {
        let mut fi = FractionalIndex::new(vec![], vec![], [b'a', 0, 0, 0]);
        assert_eq!(fi.lower_bound(&virtual_key(1)), 0);
        fi.insert_key(virtual_key(2));
        fi.insert_key(virtual_key(4));
        assert_eq!(fi.lower_bound(&virtual_key(1)), 0);
        assert_eq!(fi.lower_bound(&virtual_key(2)), 0); // present: its own slot
        assert_eq!(fi.lower_bound(&virtual_key(3)), 1);
        assert_eq!(fi.lower_bound(&virtual_key(5)), 2); // past the tail
    }
}
