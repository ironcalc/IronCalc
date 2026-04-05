use std::hash::{Hash, Hasher};

// ---------------------------------------------------------------------------
// Vendored FxHash — frozen, deterministic, platform-independent
// ---------------------------------------------------------------------------

const FX_SEED: u64 = 0x517cc1b727220a95;

/// A deterministic, platform-independent hasher based on FxHash.
///
/// Processes bytes one at a time with rotate-xor-multiply, ensuring identical
/// output regardless of target endianness or pointer width.
///
/// **This implementation is frozen** — hashes produced by it are persisted as
/// cell keys in the `.icalc` format and must never change.
#[derive(Default)]
struct FxHasher {
    hash: u64,
}

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.hash = (self.hash.rotate_left(5) ^ (byte as u64)).wrapping_mul(FX_SEED);
        }
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.hash = (self.hash.rotate_left(5) ^ (i as u64)).wrapping_mul(FX_SEED);
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.hash = (self.hash.rotate_left(5) ^ i).wrapping_mul(FX_SEED);
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.write_u64(i as u64);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compute a deterministic content hash for any `Hash` type using FxHasher.
pub fn content_hash<T: Hash + ?Sized>(value: &T) -> u64 {
    let mut hasher = FxHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_deterministic() {
        assert_eq!(content_hash("hello"), content_hash("hello"));
    }

    #[test]
    fn test_content_hash_distinct() {
        assert_ne!(content_hash("hello"), content_hash("world"));
        assert_ne!(content_hash(""), content_hash("a"));
        assert_ne!(content_hash("a"), content_hash("ab"));
    }

    #[test]
    fn test_string_and_str_same_hash() {
        let s = String::from("test");
        assert_eq!(content_hash(s.as_str()), content_hash(s.as_str()));
    }

    #[test]
    fn test_empty_string_hash_nonzero() {
        assert_ne!(content_hash(""), 0);
    }
}
