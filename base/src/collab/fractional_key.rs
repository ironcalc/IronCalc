//! [`FractionalKey`]: an immutable, ordered byte string packed into a single machine word.
//!
//! Fractional keys are stored in bulk — one per row, column, cell and conditional-formatting rule —
//! so their footprint matters more than their peak length. Real keys are a handful of bytes, so the
//! representation keeps them inline and only reaches for the heap in the tail case:
//!
//! ```text
//! byte 0    bytes 1..8
//! ------    -------------------------------------------
//! len       payload[0..len]                 (len <= 7)
//! len       heap address, 7 bytes LE        (len >= 8) -> [ AtomicUsize strong | len bytes ]
//! ```
//!
//! The heap block is refcounted, so cloning a long key bumps a counter instead of copying — keys
//! are immutable and get cloned constantly (every `Iter`, every patch), so sharing is free and
//! always safe. The counter is atomic rather than plain, which is what keeps [`FractionalKey`]
//! `Send + Sync` and therefore keeps `Patch` and `CollaborativeWorkbook` sendable.
//!
//! Two consequences of that layout are load-bearing and enforced here:
//!
//! * The length lives in one byte, so [`MAX_KEY_LEN`] is 255. Every fallible entry point returns
//!   [`KeyTooLong`] rather than truncating. Key generation is what keeps this from binding: see
//!   [`FractionalIndex`](crate::collab::fractional_index::FractionalIndex), whose keys grow
//!   logarithmically with the number of positions rather than linearly.
//! * The heap address gets 7 bytes, so it must fit in 56 bits. That holds for user-space addresses
//!   on every target we run on (x86-64 caps user space at 2^47, and even under 5-level paging at
//!   2^56-1; aarch64 tops out at 2^52), and it is asserted unconditionally on the allocation path
//!   rather than assumed.
//!
//! Ordering is in two parts. Every minted key is `position ++ session`, where the trailing
//! [`SESSION_SUFFIX_LEN`] bytes identify the session that minted it, so the *position* is compared
//! lexicographically on its own and the session only breaks ties between keys minted for the same
//! spot by different peers. Ordering on the whole byte string instead would let the session bytes
//! outweigh the position: `ff` and `ff 00` name adjacent positions, but with a session suffix of
//! `61 00 00 00` glued on, `ff 00 | 61 00 00 00` sorts *below* `ff | 61 00 00 00`.
//!
//! Note that neither half is the ordering of the raw 8 bytes, since those lead with the length.

use bitcode::__private::{Buffer, Decoder, Encoder, View};
use bitcode::{Decode, Encode};
use serde::de::{Error as _, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::atomic::{fence, AtomicUsize, Ordering as AtomicOrdering};

/// Payload bytes that fit alongside the length byte, without allocating.
pub const INLINE_CAP: usize = 7;

/// Trailing bytes of every minted key that identify the session rather than the position. See the
/// [module docs] for what that split means for ordering.
///
/// [module docs]: self
pub const SESSION_SUFFIX_LEN: usize = 4;

/// The longest key the length byte can describe.
pub const MAX_KEY_LEN: usize = u8::MAX as usize;

/// Largest address the 7 pointer bytes of the heap variant can hold.
const MAX_ADDR: u64 = 0x00FF_FFFF_FFFF_FFFF;

/// Size of the refcount that prefixes the payload in a heap block.
const HEADER: usize = std::mem::size_of::<AtomicUsize>();

/// Refcount ceiling, as in `Arc`: a count this high can only come from leaked clones, and treating
/// it as fatal keeps the counter from wrapping around to zero and causing a use-after-free.
const MAX_REFCOUNT: usize = isize::MAX as usize;

/// The scratch buffer keys are built in before being sealed into a [`FractionalKey`].
///
/// [`FractionalKey`] is immutable by design: growing it in place would mean repeatedly promoting
/// between the inline and heap forms, so key generation accumulates here and converts once.
pub type KeyBuf = Vec<u8>;

/// A byte string longer than [`MAX_KEY_LEN`] was offered to a [`FractionalKey`] constructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyTooLong {
    pub len: usize,
}

impl fmt::Display for KeyTooLong {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "fractional key of {} bytes exceeds the {MAX_KEY_LEN} byte limit",
            self.len
        )
    }
}

impl std::error::Error for KeyTooLong {}

/// An immutable byte string ordered lexicographically by its payload. See the [module docs] for the
/// memory layout.
///
/// [module docs]: self
pub struct FractionalKey([u8; 8]);

// Same reasoning as `Arc<[u8]>`: the heap block is shared but immutable, and the only mutable state
// in it is the refcount, which is atomic. The raw address bytes are all that opt us out of the
// automatic impls. A non-atomic counter would make both of these unsound.
unsafe impl Send for FractionalKey {}
unsafe impl Sync for FractionalKey {}

impl FractionalKey {
    /// Null key represents trash space.
    pub const NULL: Self = FractionalKey([0; 8]);

    /// The empty key.
    pub const fn new() -> Self {
        FractionalKey([0; 8])
    }

    /// Builds a key from `bytes`, or fails if it is longer than [`MAX_KEY_LEN`].
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, KeyTooLong> {
        if bytes.len() > MAX_KEY_LEN {
            return Err(KeyTooLong { len: bytes.len() });
        }
        Ok(Self::from_bytes_unchecked(bytes))
    }

    /// Builds a key from `bytes`, which the caller has already established is short enough.
    fn from_bytes_unchecked(bytes: &[u8]) -> Self {
        debug_assert!(bytes.len() <= MAX_KEY_LEN);
        let len = bytes.len();
        if len <= INLINE_CAP {
            let mut raw = [0u8; 8];
            raw[0] = len as u8;
            raw[1..1 + len].copy_from_slice(bytes);
            FractionalKey(raw)
        } else {
            let layout = Self::heap_layout(len);
            // SAFETY: `len > INLINE_CAP` so the layout has non-zero size. The block is written in
            // full before anyone can observe it: refcount 1 at offset 0, payload after it.
            let block = unsafe {
                let block = alloc(layout);
                if block.is_null() {
                    handle_alloc_error(layout);
                }
                (block as *mut AtomicUsize).write(AtomicUsize::new(1));
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), block.add(HEADER), len);
                block
            };
            let addr = block as usize as u64;
            assert!(
                addr <= MAX_ADDR,
                "heap address {addr:#x} does not fit in the 56 bits FractionalKey reserves for it"
            );
            // Little-endian regardless of target endianness: the byte order here is our own wire
            // order within the word, and `from_le_bytes` reverses it symmetrically.
            let a = addr.to_le_bytes();
            FractionalKey([len as u8, a[0], a[1], a[2], a[3], a[4], a[5], a[6]])
        }
    }

    /// Number of payload bytes.
    #[inline]
    pub const fn len(&self) -> usize {
        self.0[0] as usize
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0[0] == 0
    }

    /// Whether the payload lives in the word itself rather than on the heap.
    #[inline]
    pub const fn is_inline(&self) -> bool {
        self.len() <= INLINE_CAP
    }

    /// Layout of the heap block backing a key of `len` bytes: refcount followed by the payload.
    #[inline]
    fn heap_layout(len: usize) -> Layout {
        // The payload is `u8`, so it needs no padding after the header and the layout is exact.
        Layout::from_size_align(HEADER + len, std::mem::align_of::<AtomicUsize>())
            .expect("fractional key layout is far below the size limit")
    }

    /// Address of the heap *block* (the refcount, not the payload). Only meaningful when
    /// `!self.is_inline()`.
    #[inline]
    fn heap_ptr(&self) -> *mut u8 {
        let b = &self.0;
        let addr = u64::from_le_bytes([b[1], b[2], b[3], b[4], b[5], b[6], b[7], 0]);
        addr as usize as *mut u8
    }

    /// The refcount shared by every clone of this key. Only meaningful when `!self.is_inline()`.
    #[inline]
    fn strong(&self) -> &AtomicUsize {
        // SAFETY: for a heap key the block always starts with an initialised `AtomicUsize` written
        // by `from_bytes_unchecked`, and it stays live for as long as this key holds a count.
        unsafe { &*(self.heap_ptr() as *const AtomicUsize) }
    }

    /// The payload, whichever form it is stored in.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        let len = self.len();
        if len <= INLINE_CAP {
            &self.0[1..1 + len]
        } else {
            // SAFETY: `len > INLINE_CAP` means the word holds a block address produced by
            // `from_bytes_unchecked`, whose payload runs for exactly `len` bytes past the header.
            // This key holds a strong count, so the block outlives the borrow, and the payload is
            // never mutated after construction, so handing out a shared slice is sound.
            unsafe { std::slice::from_raw_parts(self.heap_ptr().add(HEADER), len) }
        }
    }

    /// Splits the key into the position it names and the session that minted it — the two halves
    /// [`Ord`] compares, in that order.
    ///
    /// A key shorter than [`SESSION_SUFFIX_LEN`] is all session and no position. That only happens
    /// for keys nobody minted, such as [`FractionalKey::new`], which is exactly where sorting below
    /// every real key is the wanted behaviour.
    #[inline]
    pub fn split(&self) -> (&[u8], &[u8]) {
        let bytes = self.as_bytes();
        bytes.split_at(bytes.len().saturating_sub(SESSION_SUFFIX_LEN))
    }

    /// The position half of the key: everything before the session suffix.
    #[inline]
    pub fn position(&self) -> &[u8] {
        self.split().0
    }

    /// Number of keys currently sharing this key's heap block. Inline keys share nothing.
    #[cfg(test)]
    fn strong_count(&self) -> usize {
        if self.is_inline() {
            0
        } else {
            self.strong().load(AtomicOrdering::Acquire)
        }
    }
}

impl Default for FractionalKey {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for FractionalKey {
    fn drop(&mut self) {
        if self.is_inline() {
            return;
        }
        // `Release` publishes everything this thread did with the payload before giving up its
        // count, so the thread that observes the count hit zero sees it all. Mirrors `Arc::drop`.
        if self.strong().fetch_sub(1, AtomicOrdering::Release) != 1 {
            return;
        }
        // The matching `Acquire`: no access to the block may be reordered after the free below.
        fence(AtomicOrdering::Acquire);
        // SAFETY: we just took the count from 1 to 0, so no other key references this block and
        // none can appear — counts are only ever created by cloning an existing key. The layout is
        // recomputed from the same `len` that `from_bytes_unchecked` allocated with.
        unsafe { dealloc(self.heap_ptr(), Self::heap_layout(self.len())) }
    }
}

impl Clone for FractionalKey {
    /// Shares the heap block rather than copying it: keys are immutable, so every clone can point
    /// at the same payload. Inline keys are just a word copy.
    fn clone(&self) -> Self {
        if !self.is_inline() {
            // `Relaxed` is enough: we already hold a count, so the block cannot be freed underneath
            // us, and no memory beyond the counter is being published here.
            if self.strong().fetch_add(1, AtomicOrdering::Relaxed) > MAX_REFCOUNT {
                std::process::abort();
            }
        }
        FractionalKey(self.0)
    }
}

impl fmt::Debug for FractionalKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FractionalKey(\"")?;
        for b in self.as_bytes() {
            write!(f, "{b:02x}")?;
        }
        write!(f, "\")")
    }
}

impl PartialEq for FractionalKey {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl Eq for FractionalKey {}

impl PartialOrd for FractionalKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FractionalKey {
    /// Position first, session second — see the [module docs]. Both halves compare
    /// lexicographically, and neither compares the raw word, which leads with the length.
    ///
    /// [module docs]: self
    fn cmp(&self, other: &Self) -> Ordering {
        let (position, session) = self.split();
        let (other_position, other_session) = other.split();
        position
            .cmp(other_position)
            .then_with(|| session.cmp(other_session))
    }
}

impl Hash for FractionalKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the payload so that `Hash` and `Eq` agree across the inline/heap split.
        self.as_bytes().hash(state);
    }
}

impl AsRef<[u8]> for FractionalKey {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

/// A blanket `impl<T: AsRef<[u8]>> From<T>` is impossible: it would overlap core's reflexive
/// `impl<T> From<T> for T` once `FractionalKey: AsRef<[u8]>`. These concrete impls stand in for it,
/// and all of them panic past [`MAX_KEY_LEN`] — use [`FractionalKey::try_from_bytes`] for input
/// whose length you do not control.
impl From<&[u8]> for FractionalKey {
    fn from(bytes: &[u8]) -> Self {
        Self::try_from_bytes(bytes).expect("fractional key too long")
    }
}

impl<const N: usize> From<&[u8; N]> for FractionalKey {
    fn from(bytes: &[u8; N]) -> Self {
        Self::from(bytes.as_slice())
    }
}

impl From<Vec<u8>> for FractionalKey {
    fn from(bytes: Vec<u8>) -> Self {
        Self::from(bytes.as_slice())
    }
}

impl Serialize for FractionalKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            use std::fmt::Write as _;
            let mut hex = String::with_capacity(self.len() * 2);
            for b in self.as_bytes() {
                let _ = write!(hex, "{b:02x}");
            }
            serializer.serialize_str(&hex)
        } else {
            serializer.serialize_bytes(self.as_bytes())
        }
    }
}

impl<'de> Deserialize<'de> for FractionalKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(FractionalKeyVisitor)
        } else {
            deserializer.deserialize_bytes(FractionalKeyVisitor)
        }
    }
}

/// Accepts every shape a `FractionalKey` may have been written as — hex string, borrowed or owned
/// bytes, or a sequence of `u8` — so a payload round-trips even through a format whose
/// `is_human_readable` disagrees with the one that produced it.
struct FractionalKeyVisitor;

impl<'de> Visitor<'de> for FractionalKeyVisitor {
    type Value = FractionalKey;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "a hex string or byte sequence of at most {MAX_KEY_LEN} bytes"
        )
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        if !v.len().is_multiple_of(2) {
            return Err(E::custom("fractional key hex string has an odd length"));
        }
        if v.len() / 2 > MAX_KEY_LEN {
            return Err(E::custom(KeyTooLong { len: v.len() / 2 }));
        }
        let mut buf = KeyBuf::with_capacity(v.len() / 2);
        for pair in v.as_bytes().chunks(2) {
            let s = std::str::from_utf8(pair).map_err(E::custom)?;
            buf.push(u8::from_str_radix(s, 16).map_err(E::custom)?);
        }
        Ok(FractionalKey::from_bytes_unchecked(&buf))
    }

    fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        FractionalKey::try_from_bytes(v).map_err(E::custom)
    }

    fn visit_byte_buf<E: serde::de::Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
        self.visit_bytes(&v)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut buf = KeyBuf::new();
        while let Some(b) = seq.next_element::<u8>()? {
            if buf.len() == MAX_KEY_LEN {
                return Err(A::Error::custom(KeyTooLong {
                    len: MAX_KEY_LEN + 1,
                }));
            }
            buf.push(b);
        }
        Ok(FractionalKey::from_bytes_unchecked(&buf))
    }
}

/// bitcode has no public escape hatch for adapting a foreign type (its `impl_convert!` is
/// `pub(crate)`), so the coder pair is written out by hand. Both delegate to the byte-slice coders
/// bitcode already provides, so the wire format is exactly that of a `Vec<u8>`.
#[derive(Default)]
pub struct FractionalKeyEncoder(<[u8] as Encode>::Encoder);

impl Buffer for FractionalKeyEncoder {
    fn collect_into(&mut self, out: &mut Vec<u8>) {
        self.0.collect_into(out);
    }

    fn reserve(&mut self, additional: NonZeroUsize) {
        self.0.reserve(additional);
    }
}

impl Encoder<FractionalKey> for FractionalKeyEncoder {
    #[inline]
    fn encode(&mut self, t: &FractionalKey) {
        Encoder::<[u8]>::encode(&mut self.0, t.as_bytes());
    }
}

impl Encode for FractionalKey {
    type Encoder = FractionalKeyEncoder;
}

#[derive(Default)]
pub struct FractionalKeyDecoder<'a>(<Vec<u8> as Decode<'a>>::Decoder);

impl<'a> View<'a> for FractionalKeyDecoder<'a> {
    fn populate(&mut self, input: &mut &'a [u8], length: usize) -> bitcode::__private::Result<()> {
        self.0.populate(input, length)
    }
}

impl<'a> Decoder<'a, FractionalKey> for FractionalKeyDecoder<'a> {
    /// # Panics
    ///
    /// On a payload carrying a key longer than [`MAX_KEY_LEN`]. bitcode requires all validation to
    /// happen in [`View::populate`], and the length data it would need lives in `VecDecoder`'s
    /// `pub(crate)` fields, so this is the one entry point that cannot report [`KeyTooLong`] as an
    /// error. Untrusted input should come in through `serde`, which rejects it cleanly.
    #[inline]
    fn decode(&mut self) -> FractionalKey {
        let bytes: Vec<u8> = self.0.decode();
        FractionalKey::try_from_bytes(&bytes).expect("malformed bitcode payload")
    }
}

impl<'a> Decode<'a> for FractionalKey {
    type Decoder = FractionalKeyDecoder<'a>;
}

#[cfg(test)]
mod test {
    use super::{FractionalKey, KeyTooLong, INLINE_CAP, MAX_KEY_LEN};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hash_of(k: &FractionalKey) -> u64 {
        let mut h = DefaultHasher::new();
        k.hash(&mut h);
        h.finish()
    }

    /// Every length across the inline/heap boundary must round-trip its payload unchanged.
    #[test]
    fn roundtrips_across_the_inline_boundary() {
        for len in 0..=MAX_KEY_LEN {
            let bytes: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();
            let key = FractionalKey::from(bytes.as_slice());
            assert_eq!(key.len(), len);
            assert_eq!(key.as_bytes(), &bytes[..], "payload lost at len {len}");
            assert_eq!(key.is_inline(), len <= INLINE_CAP);
            assert_eq!(key.is_empty(), len == 0);
            let cloned = key.clone();
            assert_eq!(cloned, key);
            assert_eq!(hash_of(&cloned), hash_of(&key));
            if key.is_inline() {
                assert_eq!(key.strong_count(), 0);
            } else {
                // Heap clones share one block rather than copying it.
                assert_eq!(key.as_bytes().as_ptr(), cloned.as_bytes().as_ptr());
                assert_eq!(key.strong_count(), 2);
            }
        }
    }

    /// The refcount must track clones exactly, and the payload must stay readable through every
    /// surviving handle until the last one drops.
    #[test]
    fn heap_clones_share_one_refcounted_block() {
        let bytes: Vec<u8> = (0..64u8).collect();
        let key = FractionalKey::from(bytes.as_slice());
        assert_eq!(key.strong_count(), 1);

        let clones: Vec<FractionalKey> = (0..16).map(|_| key.clone()).collect();
        assert_eq!(key.strong_count(), 17);
        for c in &clones {
            assert_eq!(c.as_bytes(), &bytes[..]);
            assert_eq!(c.as_bytes().as_ptr(), key.as_bytes().as_ptr());
        }

        drop(clones);
        assert_eq!(key.strong_count(), 1);
        // The original still owns a live block after all its clones are gone.
        assert_eq!(key.as_bytes(), &bytes[..]);

        // Dropping the original last is what actually frees; outliving it is fine either way.
        let survivor = key.clone();
        drop(key);
        assert_eq!(survivor.strong_count(), 1);
        assert_eq!(survivor.as_bytes(), &bytes[..]);
    }

    /// `Send`/`Sync` are asserted by hand, so pin them down: a shared block must survive being
    /// cloned and dropped from several threads at once.
    #[test]
    fn shared_block_survives_concurrent_clone_and_drop() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<FractionalKey>();

        let bytes: Vec<u8> = (0..100u8).collect();
        let key = FractionalKey::from(bytes.as_slice());
        std::thread::scope(|s| {
            for _ in 0..8 {
                let borrowed = &key;
                let expected = &bytes;
                s.spawn(move || {
                    for _ in 0..1000 {
                        let c = borrowed.clone();
                        assert_eq!(c.as_bytes(), &expected[..]);
                    }
                });
            }
        });
        assert_eq!(key.strong_count(), 1, "every clone released its count");
        assert_eq!(key.as_bytes(), &bytes[..]);
    }

    /// Ordering is lexicographic over the payload. The raw-word ordering would sort by length
    /// first, so the heap/inline split must be invisible here.
    #[test]
    fn orders_lexicographically_across_representations() {
        // `b` is inline, `c` spills to the heap, yet `b < c` because 0x01 < 0x02 at index 0.
        let a = FractionalKey::from([0x01u8].as_slice());
        let b = FractionalKey::from([0x01u8, 0x02].as_slice());
        let c = FractionalKey::from([0x02u8; 40].as_slice());
        let d = FractionalKey::from([0x02u8; 41].as_slice());

        assert!(a < b, "prefix sorts before its extension");
        assert!(
            b < c,
            "inline key sorts below a longer-but-greater heap key"
        );
        assert!(c < d, "prefix sorts before its extension on the heap too");
        assert_eq!(FractionalKey::new().cmp(&a), std::cmp::Ordering::Less);

        let mut sorted = vec![
            d.clone(),
            b.clone(),
            FractionalKey::new(),
            c.clone(),
            a.clone(),
        ];
        sorted.sort();
        assert_eq!(sorted, vec![FractionalKey::new(), a, b, c, d]);
    }

    /// Position dominates, session only breaks ties. The session bytes must never be able to drag a
    /// key below one whose position it sits above — that is what makes a run of generated keys
    /// monotone.
    #[test]
    fn orders_by_position_then_session() {
        fn key(position: &[u8], session: u8) -> FractionalKey {
            let mut bytes = position.to_vec();
            bytes.extend_from_slice(&[session, 0, 0, 0]);
            FractionalKey::from(bytes.as_slice())
        }

        // `ff 00` extends `ff`, so it sorts above it even though its session byte is far smaller.
        assert!(key(&[0xff], 0x61) < key(&[0xff, 0x00], 0x00));
        // Same position, two sessions: the suffix is the tie-break.
        assert!(key(&[0x05], 0x61) < key(&[0x05], 0x62));
        assert_eq!(key(&[0x05], 0x61), key(&[0x05], 0x61));
        // A key with no session suffix at all is below every minted key.
        assert!(FractionalKey::new() < key(&[], 0x00));
    }

    #[test]
    fn rejects_keys_past_the_length_limit() {
        let too_long = vec![0u8; MAX_KEY_LEN + 1];
        assert_eq!(
            FractionalKey::try_from_bytes(&too_long),
            Err(KeyTooLong {
                len: MAX_KEY_LEN + 1
            })
        );
        assert!(FractionalKey::try_from_bytes(&too_long[..MAX_KEY_LEN]).is_ok());
    }

    #[test]
    fn serde_roundtrips_both_binary_and_human_readable() {
        for len in [0usize, 7, 8, 200] {
            let bytes: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();
            let key = FractionalKey::from(bytes.as_slice());

            let json = serde_json::to_string(&key).unwrap();
            assert_eq!(
                serde_json::from_str::<FractionalKey>(&json).unwrap(),
                key,
                "json roundtrip failed at len {len}"
            );

            let bin = bitcode::serialize(&key).unwrap();
            assert_eq!(
                bitcode::deserialize::<FractionalKey>(&bin).unwrap(),
                key,
                "bitcode/serde roundtrip failed at len {len}"
            );
        }
    }

    /// A JSON payload longer than the limit must be an error, not a panic — this is the path
    /// untrusted peer data travels.
    #[test]
    fn serde_rejects_over_long_input() {
        let hex = "00".repeat(MAX_KEY_LEN + 1);
        let json = format!("\"{hex}\"");
        assert!(serde_json::from_str::<FractionalKey>(&json).is_err());
        // A `u8` sequence is accepted too, and is bounded the same way.
        let seq = format!("[{}]", vec!["0"; MAX_KEY_LEN + 1].join(","));
        assert!(serde_json::from_str::<FractionalKey>(&seq).is_err());
    }

    #[test]
    fn bitcode_derive_roundtrips() {
        for len in [0usize, 7, 8, 200] {
            let bytes: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();
            let keys = vec![
                FractionalKey::from(bytes.as_slice()),
                FractionalKey::new(),
                FractionalKey::from([9u8; 9].as_slice()),
            ];
            let encoded = bitcode::encode(&keys);
            assert_eq!(
                bitcode::decode::<Vec<FractionalKey>>(&encoded).unwrap(),
                keys
            );
        }
    }

    #[test]
    fn debug_renders_the_payload_as_hex() {
        let key = FractionalKey::from([0x0au8, 0xff, 0x00].as_slice());
        assert_eq!(format!("{key:?}"), "FractionalKey(\"0aff00\")");
    }

    #[test]
    fn stays_one_word_wide() {
        assert_eq!(std::mem::size_of::<FractionalKey>(), 8);
    }
}
