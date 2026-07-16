use smallvec::SmallVec;
use std::num::NonZeroU32;

/// Fractional index: a sequence of bytes that when generated between two other [FractionalKey]s
/// will have it's lexical order in between them. It's always finished with [SessionId] suffix bytes.
pub type FractionalKey = SmallVec<[u8; 8]>;

/// Since [FractionalKey] is a long structure, [FractionalIndex] assigns them a short alias that can
/// be used as a point of reference.
pub type KeyAlias = NonZeroU32;

const SESSION_HASH_SIZE: usize = 4;

/// A collection of [FractionalKey]s that enables producing them in a way that matches their desired
/// order.
pub struct FractionalIndex {
    /// Lexically sorted (by [FractionalKey]) collection of [FractionalKey]s and their aliases.
    index: Vec<(FractionalKey, KeyAlias)>,
    suffix: [u8; SESSION_HASH_SIZE],
    seq: KeyAlias,
}

impl FractionalIndex {
    pub fn new(suffix: [u8; SESSION_HASH_SIZE]) -> Self {
        FractionalIndex {
            index: Vec::new(),
            seq: unsafe { KeyAlias::new_unchecked(1) },
            suffix,
        }
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn get(&self, index: usize) -> Option<&(FractionalKey, KeyAlias)> {
        self.index.get(index)
    }

    pub fn alias_for(&self, key: &FractionalKey) -> Option<KeyAlias> {
        let index = self.index.binary_search_by_key(&key, |(k, _)| k).ok()?;
        Some(self.index[index].1)
    }

    pub fn key_for(&self, alias: KeyAlias) -> Option<&FractionalKey> {
        let (key, _) = self.index.iter().find(|(k, a)| *a == alias)?;
        Some(key)
    }

    pub fn insert_key(&mut self, key: FractionalKey) -> KeyAlias {
        match self.index.binary_search_by_key(&&key, |(k, _)| k) {
            Ok(existing_index) => self.index[existing_index].1,
            Err(insert_index) => {
                let next_alias = self.seq;
                self.seq = unsafe { NonZeroU32::new_unchecked(self.seq.get() + 1) };
                self.index.insert(insert_index, (key, next_alias));
                next_alias
            }
        }
    }

    fn neighbours(index: &Vec<(FractionalKey, KeyAlias)>, i: usize) -> Option<(&[u8], &[u8])> {
        let left = if i == 0 {
            [].as_ref()
        } else {
            let left = index[i - 1].0.as_ref();
            &left[..left.len() - SESSION_HASH_SIZE]
        };
        let right = if i <= index.len() {
            index
                .get(i)
                .map(|(key, _)| &key[..key.len() - SESSION_HASH_SIZE])
                .unwrap_or(&[])
        } else {
            return None;
        };
        Some((left, right))
    }

    /// Generate a new [FractionalKey] that matches a given index. Return that key and an alias to it.
    pub fn create_key(&mut self, index: usize) -> Option<&(FractionalKey, KeyAlias)> {
        let (left, right) = Self::neighbours(&self.index, index)?;
        let mut fkey = Self::create_fractional_key(left, right, InsertStrategy::Middle);
        fkey.extend_from_slice(self.suffix.as_ref());
        let next_alias = self.seq;
        self.seq = unsafe { NonZeroU32::new_unchecked(self.seq.get() + 1) };
        Some(&*self.index.insert_mut(index, (fkey, next_alias)))
    }

    /// Returns an iterator that lazily inserts a contiguous run of new [FractionalKey]s starting at
    /// position `start`, one per [Iterator::next] call, yielding the [KeyAlias] of each inserted key.
    pub fn create_keys(&mut self, start: usize) -> CreateKeys<'_> {
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
pub struct CreateKeys<'a> {
    owner: &'a mut FractionalIndex,
    upper_bound: SmallVec<[u8; 8]>,
    next: FractionalKey,
    index: usize,
}

impl<'a> CreateKeys<'a> {
    fn new(owner: &'a mut FractionalIndex, index: usize) -> Self {
        let (left, right) = FractionalIndex::neighbours(&owner.index, index).unwrap();
        let next = FractionalIndex::create_fractional_key(left, right, InsertStrategy::Start);
        let upper_bound = right.into();
        CreateKeys {
            owner,
            upper_bound,
            next,
            index,
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

impl Iterator for CreateKeys<'_> {
    type Item = (FractionalKey, KeyAlias);

    fn next(&mut self) -> Option<Self::Item> {
        let mut key = self.next.clone();
        self.advance(); // advance to next key
        key.extend_from_slice(self.owner.suffix.as_ref());
        let next_alias = self.owner.seq;
        self.owner.seq = unsafe { NonZeroU32::new_unchecked(self.owner.seq.get() + 1) };
        self.owner
            .index
            .insert(self.index, (key.clone(), next_alias));
        self.index += 1;
        Some((key, next_alias))
    }
}

#[cfg(test)]
mod test {
    use super::FractionalIndex;
    use std::collections::HashSet;

    #[test]
    fn fractional_index() {
        let mut fi = FractionalIndex::new(Default::default());

        let (k2, _) = fi.create_key(0).unwrap().clone(); // [.]
        let (k1, _) = fi.create_key(0).unwrap().clone(); // [. k2]
        let (k4, _) = fi.create_key(2).unwrap().clone(); // [k1 k2 .]
        let (k3, _) = fi.create_key(2).unwrap().clone(); // [k1 k2 . k4]

        let expected = vec![k1, k2, k3, k4];
        let mut actual: Vec<_> = fi.index.iter().map(|(k, _)| k.clone()).collect();
        assert_eq!(actual, expected);

        // entries should be already sorted
        actual.sort();
        assert_eq!(actual, expected);

        // all aliases are unique
        let aliases: HashSet<_> = fi.index.iter().map(|(_, a)| *a).collect();
        assert_eq!(aliases.len(), fi.len()); // all unique => no dedups
    }

    #[test]
    fn fractional_indexes() {
        let mut fi = FractionalIndex::new(Default::default());

        let (k1, _) = fi.create_key(0).unwrap().clone(); // [.]
        let (k2, _) = fi.create_key(1).unwrap().clone(); // [k1 .]

        let keys: Vec<_> = fi.create_keys(1).take(300).map(|(k, _)| k).collect();

        let mut last = k1;
        for k in keys {
            assert!(k > last, "next key should be higher than previous one");
            assert!(k < k2, "next key should be lower than the upper boundary");
            last = k;
        }
    }
}
