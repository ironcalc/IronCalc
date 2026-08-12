use crate::get_milliseconds_since_epoch;
use bitcode::__private::{Buffer, Decoder, Encoder, View};
use bitcode::{Decode, Encode};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};

/// Hybrid Logical Timestamp:
/// - upper 48bits: UNIX epoch timestamp (in millis)
/// - lower 16bits: sequence num
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Hlc(u64);

static COUNTER: AtomicU64 = AtomicU64::new(0);

impl Hlc {
    #[inline(always)]
    pub const fn new(value: u64) -> Self {
        Hlc(value)
    }

    /// The raw payload: wall clock in the upper 48 bits, sequence in the lower 16.
    #[inline(always)]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Send rule: the next stamp this replica mints, strictly above every stamp it has seen.
    pub fn now() -> Self {
        loop {
            let latest = COUNTER.load(Ordering::SeqCst);
            // the wall clock occupies the upper 48 bits, leaving the low 16 to count within one ms
            let wall = (get_milliseconds_since_epoch() as u64) << 16;
            let next = wall.max(latest) + 1;

            if COUNTER
                .compare_exchange(latest, next, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return Hlc(next);
            }
        }
    }

    /// Receive rule: registers a stamp minted elsewhere as a potential high watermark for the local
    /// clock, and returns it unchanged.
    pub fn sync(timestamp: Self) -> Self {
        COUNTER.fetch_max(timestamp.0, Ordering::SeqCst);
        timestamp
    }
}

impl<'de> Deserialize<'de> for Hlc {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TimestampVisitor;
        impl Visitor<'_> for TimestampVisitor {
            type Value = Hlc;

            fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
                f.write_str("HLC timestamp")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Hlc, E> {
                let t = Hlc(value);
                // synchronize the timestamp with our current knowledge
                Hlc::sync(t);
                Ok(t)
            }
        }

        deserializer.deserialize_u64(TimestampVisitor)
    }
}

impl Serialize for Hlc {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

/// bitcode coders delegating to the `u64` ones — hand-rolled for the same reason as
/// [`FractionalKeyEncoder`](crate::collab::fractional_key::FractionalKeyEncoder), and applying the
/// same receive rule on the way in as the serde path above.
#[derive(Default)]
pub struct HlcEncoder(<u64 as Encode>::Encoder);

impl Buffer for HlcEncoder {
    fn collect_into(&mut self, out: &mut Vec<u8>) {
        self.0.collect_into(out);
    }

    fn reserve(&mut self, additional: NonZeroUsize) {
        self.0.reserve(additional);
    }
}

impl Encoder<Hlc> for HlcEncoder {
    #[inline]
    fn encode(&mut self, t: &Hlc) {
        Encoder::<u64>::encode(&mut self.0, &t.0);
    }
}

impl Encode for Hlc {
    type Encoder = HlcEncoder;
}

#[derive(Default)]
pub struct HlcDecoder<'a>(<u64 as Decode<'a>>::Decoder);

impl<'a> View<'a> for HlcDecoder<'a> {
    fn populate(&mut self, input: &mut &'a [u8], length: usize) -> bitcode::__private::Result<()> {
        self.0.populate(input, length)
    }
}

impl<'a> Decoder<'a, Hlc> for HlcDecoder<'a> {
    #[inline]
    fn decode(&mut self) -> Hlc {
        // synchronize the timestamp with our current knowledge
        Hlc::sync(Hlc(self.0.decode()))
    }
}

impl<'a> Decode<'a> for Hlc {
    type Decoder = HlcDecoder<'a>;
}

#[cfg(test)]
mod test {
    use super::*;

    /// The clock is process-global, so this asserts only what holds whatever else the test binary
    /// stamps in parallel. The headroom below is what keeps the future stamps ahead of the wall
    /// clock for the few microseconds this test takes.
    const AHEAD: u64 = 1 << 24;

    #[test]
    fn hlc_clock_and_codecs() {
        // The wall clock sits in the upper 48 bits.
        let before = get_milliseconds_since_epoch() as u64;
        let t = Hlc::now();
        let after = get_milliseconds_since_epoch() as u64;
        // Holds only while no test running in parallel syncs a stamp from the future: the clock is
        // process-global, so such a stamp would push `now()` past `after`. Test stamps are anchored
        // in the past for exactly this reason — see `PAST` in `collab::apply`'s tests.
        assert!((before..=after).contains(&(t.get() >> 16)));

        // Stamps strictly increase, and those minted within one millisecond differ in the low 16
        // bits alone.
        let stamps: Vec<Hlc> = (0..100).map(|_| Hlc::now()).collect();
        assert!(stamps.windows(2).all(|w| w[0] < w[1]));
        assert!(stamps
            .windows(2)
            .any(|w| w[0].get() >> 16 == w[1].get() >> 16));

        // Receive rule: `sync` hands the stamp back unchanged, and the clock it registers it with
        // now mints above it...
        let future = Hlc::new(Hlc::now().get() + AHEAD);
        assert_eq!(Hlc::sync(future), future);
        assert!(Hlc::now() > future);
        // ...where a stamp from the past is no watermark at all.
        assert_eq!(Hlc::sync(Hlc::new(1)), Hlc::new(1));
        assert!(Hlc::now() > future);

        // Both codecs round-trip the value, and both apply the receive rule while decoding.
        let ahead = Hlc::new(Hlc::now().get() + AHEAD);
        let via_serde: Hlc = bitcode::deserialize(&bitcode::serialize(&ahead).unwrap()).unwrap();
        assert_eq!(via_serde, ahead);
        assert!(Hlc::now() > ahead);

        let ahead = Hlc::new(Hlc::now().get() + AHEAD);
        let via_bitcode: Hlc = bitcode::decode(&bitcode::encode(&ahead)).unwrap();
        assert_eq!(via_bitcode, ahead);
        assert!(Hlc::now() > ahead);
    }
}
