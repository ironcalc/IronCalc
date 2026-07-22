use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::{Bound, HashMap};
use std::num::NonZeroU32;
use std::ops::RangeBounds;

/// Fractional index: a sequence of bytes that when generated between two other [FractionalKey]s
/// will have it's lexical order in between them. It's always finished with [SessionId] suffix bytes.
pub type FractionalKey = SmallVec<[u8; 8]>;

/// Since [FractionalKey] is a long structure, [FractionalIndex] assigns them a short alias that can
/// be used as a point of reference.
pub type KeyAlias = NonZeroU32;

const SESSION_HASH_SIZE: usize = 4;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Origin {
    /// Another fractional key with a given alias, that has been moved to a new position shown by
    /// the associated [FractionalKey].
    Moved,
    /// Direct (unmoved) fractional key alias.
    Direct,
}

/// A single entry stored in a [FractionalIndex]: a [FractionalKey] together with its [KeyAlias] and
/// the [Origin] describing how it came to be at its current position.
#[derive(Debug, PartialEq, Eq)]
pub struct Entry {
    key: FractionalKey,
    alias: KeyAlias,
    origin: Origin,
}

/// A collection of [FractionalKey]s that enables producing them in a way that matches their desired
/// order.
pub struct FractionalIndex {
    /// Lexically sorted (by [FractionalKey]) collection of [FractionalKey]s and their aliases.
    index: Vec<Entry>,
    origins: HashMap<KeyAlias, FractionalKey>,
    suffix: [u8; SESSION_HASH_SIZE],
    seq: KeyAlias,
}

impl FractionalIndex {
    pub fn new(suffix: [u8; SESSION_HASH_SIZE]) -> Self {
        FractionalIndex {
            index: Vec::new(),
            origins: HashMap::new(),
            seq: unsafe { KeyAlias::new_unchecked(1) },
            suffix,
        }
    }

    pub fn view(&self) -> impl Iterator<Item = &FractionalKey> {
        self.index.iter().map(move |e| match e.origin {
            Origin::Direct => &e.key,
            Origin::Moved => self.origins.get(&e.alias).unwrap(),
        })
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn get(&self, index: usize) -> Option<&Entry> {
        self.index.get(index)
    }

    pub fn alias_for(&self, key: &FractionalKey) -> Option<KeyAlias> {
        let index = self.index.binary_search_by_key(&key, |e| &e.key).ok()?;
        Some(self.index[index].alias)
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
        let moved: Vec<_> = self.index.drain(source).collect();
        let len = moved.len();
        // if start < dest, we need to shift by the number of drained entries
        let mut dest = if start < dest { dest - len } else { dest };

        let mut key_gen = self.create_keys(dest);
        for entry in moved {
            let dest_key = key_gen.next().unwrap();
            self.index.insert(
                dest,
                Entry {
                    key: dest_key,
                    alias: entry.alias,
                    origin: Origin::Moved,
                },
            );
            // insert only if the value didn't exist already
            self.origins.entry(entry.alias).or_insert(entry.key);
            dest += 1;
        }
    }

    fn next_alias(&mut self) -> KeyAlias {
        let next_alias = self.seq;
        self.seq = unsafe { NonZeroU32::new_unchecked(self.seq.get() + 1) };
        next_alias
    }

    pub fn insert(&mut self, fkey: FractionalKey, origin: Option<FractionalKey>) -> KeyAlias {
        match self.index.binary_search_by_key(&&fkey, |e| &e.key) {
            Ok(existing_index) => self.index[existing_index].alias,
            Err(insert_index) => {
                let next_alias = self.next_alias();
                self.index.insert(
                    insert_index,
                    Entry {
                        key: fkey,
                        alias: next_alias,
                        origin: Origin::Direct,
                    },
                );
                next_alias
            }
        }
    }

    fn neighbours(index: &Vec<Entry>, i: usize) -> Option<(&[u8], &[u8])> {
        let left = if i == 0 {
            [].as_ref()
        } else {
            let left = index[i - 1].key.as_ref();
            &left[..left.len() - SESSION_HASH_SIZE]
        };
        let right = if i <= index.len() {
            index
                .get(i)
                .map(|e| &e.key[..e.key.len() - SESSION_HASH_SIZE])
                .unwrap_or(&[])
        } else {
            return None;
        };
        Some((left, right))
    }

    /// Generate a new [FractionalKey] that matches a given index. Return that key and an alias to it.
    pub fn create_key(&mut self, index: usize) -> Option<&Entry> {
        let (left, right) = Self::neighbours(&self.index, index)?;
        let mut fkey = Self::create_fractional_key(left, right, InsertStrategy::Middle);
        fkey.extend_from_slice(self.suffix.as_ref());
        let next_alias = self.next_alias();
        Some(&*self.index.insert_mut(
            index,
            Entry {
                key: fkey,
                alias: next_alias,
                origin: Origin::Direct,
            },
        ))
    }

    /// Returns an iterator that lazily inserts a contiguous run of new [FractionalKey]s starting at
    /// position `start`, one per [Iterator::next] call, yielding the [KeyAlias] of each inserted key.
    pub fn create_keys(&self, start: usize) -> CreateKeys {
        CreateKeys::new(self, start)
    }

    /// Creates a new [FractionalKey] which is lexically higher than `lo` and lower than `hi`.
    fn create_fractional_key(lo: &[u8], hi: &[u8], strategy: InsertStrategy) -> FractionalKey {
        let mut key = FractionalKey::new();
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

    pub fn iter(&self) -> Iter<'_> {
        Iter::new(self)
    }

    pub fn merge(&mut self, e: IterEntry) {
        match e.moved_to {
            None => match self.index.binary_search_by_key(&&e.key, |e2| &e2.key) {
                Ok(found_index) => {
                    let curr = &mut self.index[found_index];
                    if curr.origin != Origin::Direct {
                        curr.origin = Origin::Direct;
                        self.origins.remove(&curr.alias);
                    }
                }
                Err(insert_index) => {
                    let alias = self.next_alias();
                    self.index.insert(
                        insert_index,
                        Entry {
                            key: e.key,
                            alias,
                            origin: Origin::Direct,
                        },
                    );
                }
            },
            Some(dest) => {
                let origins = &self.origins;
                let existing = self.index.iter().position(|e2| match e2.origin {
                    Origin::Direct => e2.key == e.key,
                    Origin::Moved => origins[&e2.alias] == e.key,
                });
                let alias = match existing {
                    Some(i) => {
                        let curr = &self.index[i];
                        if curr.origin == Origin::Moved && dest <= curr.key {
                            return; // incoming move loses; keep the current position
                        }
                        self.index.remove(i).alias // winner reuses the element's existing alias
                    }
                    None => self.next_alias(), // element unknown here: materialise it at `dest`
                };
                let insert_index = match self.index.binary_search_by_key(&&dest, |e2| &e2.key) {
                    Ok(i) | Err(i) => i,
                };
                self.index.insert(
                    insert_index,
                    Entry {
                        key: dest,
                        alias,
                        origin: Origin::Moved,
                    },
                );
                self.origins.insert(alias, e.key);
            }
        }
    }

    pub fn merge_iter(&mut self, entries: impl Iterator<Item = IterEntry>) {
        for e in entries {
            self.merge(e);
        }
    }
}

pub struct Iter<'a> {
    inner: std::slice::Iter<'a, Entry>,
    origins: &'a HashMap<KeyAlias, FractionalKey>,
}

impl<'a> Iter<'a> {
    fn new(index: &'a FractionalIndex) -> Self {
        Iter {
            inner: index.index.iter(),
            origins: &index.origins,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = IterEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let e = self.inner.next()?;
        let mut result = IterEntry {
            key: e.key.clone(),
            moved_to: None,
        };
        if let Some(origin) = self.origins.get(&e.alias) {
            result.moved_to = Some(result.key);
            result.key = origin.clone();
        }
        Some(result)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IterEntry {
    pub key: FractionalKey,
    pub moved_to: Option<FractionalKey>,
}

enum InsertStrategy {
    Middle,
    Start,
}

impl InsertStrategy {
    #[inline]
    pub fn next_byte(&self, lo: u16, hi: u16) -> u8 {
        match self {
            InsertStrategy::Middle => (lo + (hi - lo) / 2) as u8,
            InsertStrategy::Start => (lo + 1) as u8,
        }
    }
}

/// Iterator produced by [FractionalIndex::create_keys]. Each [Iterator::next] inserts one new
/// [FractionalKey] right after the previous one (and before the originally captured right neighbour)
/// and returns its [KeyAlias].
pub struct CreateKeys {
    suffix: [u8; SESSION_HASH_SIZE],
    upper_bound: SmallVec<[u8; 8]>,
    next: FractionalKey,
    index: usize,
}

impl CreateKeys {
    fn new(index: &FractionalIndex, i: usize) -> Self {
        let (left, right) = FractionalIndex::neighbours(&index.index, i).unwrap();
        let next = FractionalIndex::create_fractional_key(left, right, InsertStrategy::Start);
        let upper_bound = right.into();
        CreateKeys {
            suffix: index.suffix,
            upper_bound,
            next,
            index: i,
        }
    }

    /// Get current `self.next` fractional key and try to advance it to next one.
    /// If last byte falls out of byte boundary, we need to push new byte level.
    /// If next fractional key would be >= upper_bound, we need to push new byte level.
    fn advance(&mut self) {
        let i = self.next.len() - 1;

        let lo_digit = &mut self.next[i];
        let hi_digit = match self.upper_bound.get(i) {
            Some(&b) => b as u16,
            None => 256,
        };
        if *lo_digit as u16 + 1 >= hi_digit {
            // no room left, progress to next
            self.next.push(0);
        } else {
            *lo_digit += 1;
        }
    }
}

impl Iterator for CreateKeys {
    type Item = FractionalKey;

    fn next(&mut self) -> Option<Self::Item> {
        let mut key = self.next.clone();
        self.advance(); // advance to next key
        key.extend_from_slice(self.suffix.as_ref());
        self.index += 1;
        Some(key)
    }
}

#[cfg(test)]
mod test {
    use super::{FractionalIndex, FractionalKey, KeyAlias};
    use std::collections::HashSet;

    /// Aliases of every entry, in stored order. Aliases are stable per element across a `move_to`,
    /// so this tracks element movement *within* a single [FractionalIndex].
    fn alias_order(fi: &FractionalIndex) -> Vec<KeyAlias> {
        fi.index.iter().map(|e| e.alias).collect()
    }

    #[test]
    fn fractional_index() {
        let mut fi = FractionalIndex::new(Default::default());

        let k2 = fi.create_key(0).unwrap().key.clone(); // [.]
        let k1 = fi.create_key(0).unwrap().key.clone(); // [. k2]
        let k4 = fi.create_key(2).unwrap().key.clone(); // [k1 k2 .]
        let k3 = fi.create_key(2).unwrap().key.clone(); // [k1 k2 . k4]

        let expected = vec![k1, k2, k3, k4];
        let mut actual: Vec<_> = fi.index.iter().map(|e| e.key.clone()).collect();
        assert_eq!(actual, expected);

        // entries should be already sorted
        actual.sort();
        assert_eq!(actual, expected);

        // all aliases are unique
        let aliases: HashSet<_> = fi.index.iter().map(|e| e.alias).collect();
        assert_eq!(aliases.len(), fi.len()); // all unique => no dedups
    }

    #[test]
    fn fractional_indexes() {
        let mut fi = FractionalIndex::new(Default::default());

        let k1 = fi.create_key(0).unwrap().key.clone(); // [.]
        let k2 = fi.create_key(1).unwrap().key.clone(); // [k1 .]

        let keys: Vec<_> = fi.create_keys(1).take(300).collect();

        let mut last = k1;
        for k in keys {
            assert!(k > last, "next key should be higher than previous one");
            assert!(k < k2, "next key should be lower than the upper boundary");
            last = k;
        }
    }

    /// A `move_to` relocates an element: it must appear at its new position and no longer be visible
    /// at its old one, and it must not be duplicated.
    #[test]
    fn move_to_relocates_element() {
        let mut fi = FractionalIndex::new(Default::default());
        let a = fi.create_key(0).unwrap().alias; // [A]
        let b = fi.create_key(1).unwrap().alias; // [A B]
        let c = fi.create_key(2).unwrap().alias; // [A B C]

        // Move B (at index 1) to the end.
        fi.move_to(1..2, 3);

        // New order is [A C B]: B is at the end...
        assert_eq!(alias_order(&fi), vec![a, c, b]);
        // ...its old position (index 1) is now C, so B is gone from there...
        assert_eq!(fi.get(1).unwrap().alias, c);
        // ...and B exists exactly once (no duplication) with the count unchanged.
        assert_eq!(fi.len(), 3);
        assert_eq!(fi.index.iter().filter(|e| e.alias == b).count(), 1);
    }

    /// Two peers concurrently move the *same* element to different positions. After exchanging their
    /// serialized payloads (`iter` -> `merge_iter`) the element must resolve to a single position
    /// (no duplication) and both peers must end up with the exact same view.
    #[test]
    fn concurrent_move_of_same_element_converges() {
        // Distinct session suffixes => the two peers never generate colliding keys.
        let mut a = FractionalIndex::new([b'a', 0, 0, 0]);
        let mut b = FractionalIndex::new([b'b', 0, 0, 0]);

        // Shared initial list [0 x1 x2]: A creates it, B replicates via the merge payload.
        let k0 = a.create_key(0).unwrap().key.clone();
        let k1 = a.create_key(1).unwrap().key.clone();
        let k2 = a.create_key(2).unwrap().key.clone();
        b.merge_iter(a.iter());
        let view_a: Vec<_> = a.view().collect();
        let view_b: Vec<_> = b.view().collect();
        assert_eq!(view_a, view_b, "peers should start from an identical view");
        assert_eq!(view_a, vec![&k0, &k1, &k2]);

        // Both concurrently move the same element (born as k1) to different destinations.
        a.move_to(1..2, 3); // A moves it to the end
        b.move_to(1..2, 0); // B moves it to the front

        // Exchange full payloads (snapshot both before applying, so order of application is moot).
        let pa: Vec<_> = a.iter().collect();
        let pb: Vec<_> = b.iter().collect();
        a.merge_iter(pb.into_iter());
        b.merge_iter(pa.into_iter());

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
