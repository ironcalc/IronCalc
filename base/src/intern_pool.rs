use std::collections::HashMap;
use std::hash::Hash;

use bitcode::{Decode, Encode};

pub use crate::hasher::content_hash;

// ---------------------------------------------------------------------------
// InternPool<T>
// ---------------------------------------------------------------------------

/// A generic pool for interned, reference-counted values.
///
/// Each value is identified by a `u64` content hash. The pool maintains a
/// reference count for each entry and eagerly removes it when the count
/// reaches zero (see [`InternPool::release`]).
///
/// The `entries` map is serialized by bitcode; `ref_counts` is rebuilt from
/// live cell data after deserialization.
#[derive(Debug, Clone, Encode, Decode)]
pub struct InternPool<T> {
    entries: HashMap<u64, T>,
    #[bitcode(skip)]
    ref_counts: HashMap<u64, u32>,
}

impl<T: PartialEq> PartialEq for InternPool<T> {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl<T: Hash + Eq> InternPool<T> {
    pub fn new() -> Self {
        InternPool {
            entries: HashMap::new(),
            ref_counts: HashMap::new(),
        }
    }

    /// Hash `value`, insert it (or bump ref count if already present), and
    /// return the content-hash key.
    pub fn insert(&mut self, value: T) -> u64 {
        let hash = content_hash(&value);
        if let Some(existing) = self.entries.get(&hash) {
            debug_assert!(existing == &value, "InternPool hash collision detected");
            *self.ref_counts.entry(hash).or_insert(0) += 1;
        } else {
            self.entries.insert(hash, value);
            self.ref_counts.insert(hash, 1);
        }
        hash
    }

    /// Increment the reference count for an existing key.
    pub fn retain(&mut self, key: u64) {
        if self.entries.contains_key(&key) {
            *self.ref_counts.entry(key).or_insert(0) += 1;
        }
    }

    /// Decrement the reference count for `key`.
    /// When the count reaches zero the entry is **immediately removed**.
    /// Returns `true` if the entry was removed.
    pub fn release(&mut self, key: u64) -> bool {
        if let Some(count) = self.ref_counts.get_mut(&key) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.entries.remove(&key);
                self.ref_counts.remove(&key);
                return true;
            }
        }
        false
    }

    /// Read-only access to the value behind `key`.
    pub fn get(&self, key: u64) -> Option<&T> {
        self.entries.get(&key)
    }

    /// Iterate over `(key, value)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (u64, &T)> {
        self.entries.iter().map(|(k, v)| (*k, v))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Serialize to a list of `(key, value)` pairs, stripping ref counts.
    pub fn to_pairs(&self) -> Vec<(u64, T)>
    where
        T: Clone,
    {
        self.entries.iter().map(|(k, v)| (*k, v.clone())).collect()
    }

    /// Deserialize from `(key, value)` pairs. All ref counts start at zero;
    /// call [`rebuild_ref_counts`](InternPool::rebuild_ref_counts) afterwards.
    pub fn from_pairs(pairs: impl IntoIterator<Item = (u64, T)>) -> Self {
        let entries: HashMap<u64, T> = pairs.into_iter().collect();
        InternPool {
            entries,
            ref_counts: HashMap::new(),
        }
    }

    /// Reset every ref count to zero, then walk `keys` and increment each.
    /// Finally remove entries that are still zero (unreferenced).
    pub fn rebuild_ref_counts(&mut self, keys: impl Iterator<Item = u64>) {
        self.ref_counts.clear();
        for key in keys {
            if self.entries.contains_key(&key) {
                *self.ref_counts.entry(key).or_insert(0) += 1;
            }
        }
        self.entries.retain(|k, _| self.ref_counts.contains_key(k));
    }
}

impl<T: Hash + Eq> Default for InternPool<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut pool = InternPool::new();
        let k = pool.insert("hello".to_string());
        assert_eq!(pool.get(k), Some(&"hello".to_string()));
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_insert_deduplicates() {
        let mut pool = InternPool::new();
        let k1 = pool.insert("hello".to_string());
        let k2 = pool.insert("hello".to_string());
        assert_eq!(k1, k2);
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_retain_and_release() {
        let mut pool = InternPool::new();
        let k = pool.insert("a".to_string());
        pool.retain(k);
        assert!(!pool.release(k));
        assert_eq!(pool.get(k), Some(&"a".to_string()));
        assert!(pool.release(k));
        assert_eq!(pool.get(k), None);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_release_eagerly_removes() {
        let mut pool = InternPool::new();
        let k1 = pool.insert("keep".to_string());
        let k2 = pool.insert("drop".to_string());
        assert!(pool.release(k2));
        assert_eq!(pool.get(k1), Some(&"keep".to_string()));
        assert_eq!(pool.get(k2), None);
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_to_pairs_and_from_pairs() {
        let mut pool = InternPool::new();
        pool.insert("a".to_string());
        pool.insert("b".to_string());
        let pairs = pool.to_pairs();
        assert_eq!(pairs.len(), 2);

        let pool2 = InternPool::<String>::from_pairs(pairs);
        assert_eq!(pool2.len(), 2);
    }

    #[test]
    fn test_rebuild_ref_counts() {
        let k1 = content_hash("used");
        let k2 = content_hash("unused");
        let mut pool = InternPool::from_pairs(vec![
            (k1, "used".to_string()),
            (k2, "unused".to_string()),
        ]);
        pool.rebuild_ref_counts([k1, k1].into_iter());
        assert_eq!(pool.get(k1), Some(&"used".to_string()));
        assert_eq!(pool.get(k2), None);
    }

    #[test]
    fn test_iter() {
        let mut pool = InternPool::new();
        pool.insert("x".to_string());
        pool.insert("y".to_string());
        let mut items: Vec<_> = pool.iter().map(|(_, v)| v.clone()).collect();
        items.sort();
        assert_eq!(items, vec!["x".to_string(), "y".to_string()]);
    }

    #[test]
    fn test_release_saturates_at_zero() {
        let mut pool = InternPool::new();
        let k = pool.insert("x".to_string());
        assert!(pool.release(k));
        assert!(!pool.release(k));
        assert!(!pool.release(k));
        assert!(pool.is_empty());
    }
}
