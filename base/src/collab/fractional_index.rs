use serde::{Deserialize, Serialize};
use std::collections::Bound;
use std::ops::RangeBounds;

pub use crate::collab::fractional_key::{
    FractionalKey, KeyBuf, INLINE_CAP, MAX_KEY_LEN, SESSION_SUFFIX_LEN,
};
use crate::get_milliseconds_since_epoch;

pub type Timestamp = i64;

/// A single element of a [FractionalIndex]: where it sits, and the key it answers to.
///
/// The two are the same key until the element is moved. A move mints a fresh key for the new
/// position and keeps the original as the element's [identity](Entry::identity), so peers that only
/// know the element by the key it was minted as can still find it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub key: FractionalKey,
    /// UNIX timestamp when the entry was modified.
    pub modified_at: Timestamp,
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FractionalIndex {
    /// Fractional keys, sorted by the position each currently occupies ([Entry::key]).
    /// This space contains un-moved elements, or destinations of moved elements.
    active: Vec<Entry>,
    /// Fractional keys, sorted by the position each currently occupies ([Entry::key]).
    /// This space contains moved and tombstoned elements.
    moved: Vec<Entry>,

    #[serde(skip)]
    pub suffix: [u8; Self::SESSION_HASH_SIZE],
}

impl FractionalIndex {
    const SESSION_HASH_SIZE: usize = SESSION_SUFFIX_LEN;

    /// The longest position a [FractionalKey] can carry once its session suffix is appended.
    const MAX_POSITION_LEN: usize = MAX_KEY_LEN - SESSION_SUFFIX_LEN;

    pub fn new(suffix: [u8; Self::SESSION_HASH_SIZE]) -> Self {
        FractionalIndex {
            active: Vec::new(),
            moved: Vec::new(),
            suffix,
        }
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

        let modified_at = get_milliseconds_since_epoch();
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
        if let Some(source) = self.tombstone(key, Some(get_milliseconds_since_epoch())) {
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
                let removed_at = get_milliseconds_since_epoch();
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
        let modified_at = get_milliseconds_since_epoch();
        Some(
            &self
                .active
                .insert_mut(
                    index,
                    Entry {
                        key,
                        modified_at,
                        moved: FractionalKey::NULL,
                    },
                )
                .key,
        )
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
    /// `at` is what the parked record gets stamped with. A local edit passes the wall clock; a merge
    /// passes `None` to keep the timestamp the entry already carried, since a merge that read the
    /// clock would land on different state on every peer.
    fn tombstone(&mut self, key: &FractionalKey, at: Option<Timestamp>) -> Option<FractionalKey> {
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
    use super::{gap_capacity, FractionalIndex, FractionalKey};
    use std::collections::HashSet;

    /// The identity of every entry, in stored order. An identity is stable per element across a
    /// `move_to`, so this tracks element movement *within* a single [FractionalIndex].
    fn identity_order(fi: &FractionalIndex) -> Vec<FractionalKey> {
        fi.view().cloned().collect()
    }

    #[test]
    fn fractional_index() {
        let mut fi = FractionalIndex::new(Default::default());

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
        let mut fi = FractionalIndex::new(Default::default());

        let k1 = fi.create_key(0).unwrap().clone(); // [.]
        let k2 = fi.create_key(1).unwrap().clone(); // [k1 .]

        let keys = fi.create_keys(1, 2_000_000);

        let mut last = k1;
        for k in keys {
            assert!(k > last, "{k:?} should be higher than previous one");
            assert!(k < k2, "{k:?} should be lower than the upper boundary");
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
        let mut fi = FractionalIndex::new(Default::default());
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
        let mut fi = FractionalIndex::new(Default::default());
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
        let mut fi = FractionalIndex::new(Default::default());
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
        let mut a = FractionalIndex::new([b'a', 0, 0, 0]);
        let mut b = FractionalIndex::new([b'b', 0, 0, 0]);

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
}
