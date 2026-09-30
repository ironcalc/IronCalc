//! LEB128 variable-length integers, with zigzag for the signed form.
//!
//! Every count, length and delta in the collaborative wire format goes through here. The values are
//! overwhelmingly small — run lengths, prefix lengths, session counts — so a fixed-width integer
//! would spend seven bytes saying "two", and the column encoding in
//! [`codec`](crate::collab::codec) leans on that being cheap.
//!
//! # Canonical encodings
//!
//! Every value has exactly one spelling, and the readers reject the others:
//!
//! * a multi-byte encoding whose final byte contributes no bits ([`VarintError::Overlong`]) — it is
//!   a padded spelling of a value that already fit,
//! * an encoding carrying bits past the width of a `u64` ([`VarintError::Overflow`]).
//!
//! Rejecting rather than accepting-and-normalising is what lets the codec assert
//! `encode(decode(encode(x))) == encode(x)` byte for byte, which is a much sharper test than value
//! equality: it catches an encoder that quietly stops using a compression rule.
//!
//! Readers take `&mut &[u8]` and advance it past what they consumed, so a column decoder is a
//! sequence of reads against one cursor.

use std::fmt;

/// Bytes an LEB128-encoded `u64` occupies at its widest: nine 7-bit groups plus the 64th bit.
pub const MAX_UVARINT_LEN: usize = 10;

/// Why a varint could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarintError {
    /// The input ended inside the varint.
    UnexpectedEof,
    /// The varint carries bits past the width of a `u64`.
    Overflow,
    /// A padded spelling of a value that fits in fewer bytes. See the [module docs](self).
    Overlong,
}

impl fmt::Display for VarintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VarintError::UnexpectedEof => write!(f, "input ended in the middle of a varint"),
            VarintError::Overflow => write!(f, "varint does not fit in 64 bits"),
            VarintError::Overlong => write!(f, "varint is not encoded canonically"),
        }
    }
}

impl std::error::Error for VarintError {}

/// Maps a signed value onto an unsigned one that is small when the value is small either way:
/// `0, -1, 1, -2, 2, ...` become `0, 1, 2, 3, 4, ...`.
///
/// This is what makes a *negative* timestamp delta cost one byte rather than ten. Two's-complement
/// `-1` is all ones, so encoding it directly would be the widest varint there is.
#[inline]
pub const fn zigzag_encode(v: i64) -> u64 {
    // Cast before shifting: `v << 1` on its own overflows for half the domain.
    ((v as u64) << 1) ^ ((v >> 63) as u64)
}

/// Inverse of [`zigzag_encode`]. Total: every `u64` is the image of some `i64`.
#[inline]
pub const fn zigzag_decode(v: u64) -> i64 {
    ((v >> 1) as i64) ^ -((v & 1) as i64)
}

/// Appends `v` to `out`.
pub fn write_uvarint(out: &mut Vec<u8>, mut v: u64) {
    while v >= 0x80 {
        out.push(v as u8 | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}

/// Appends `v` to `out`, zigzagged so that small magnitudes of either sign stay short.
pub fn write_ivarint(out: &mut Vec<u8>, v: i64) {
    write_uvarint(out, zigzag_encode(v));
}

/// Bytes [`write_uvarint`] would append for `v`.
///
/// Encoders use this to decide whether a compression rule is worth its marker before committing to
/// it, so it has to agree with [`write_uvarint`] exactly.
#[inline]
pub const fn uvarint_len(v: u64) -> usize {
    match v {
        // Seven payload bits per byte, rounded up — and zero still needs a byte to say so.
        0 => 1,
        v => (70 - v.leading_zeros() as usize) / 7,
    }
}

/// Bytes [`write_ivarint`] would append for `v`.
#[inline]
pub const fn ivarint_len(v: i64) -> usize {
    uvarint_len(zigzag_encode(v))
}

/// Reads a varint from `input`, advancing it past the bytes consumed.
pub fn read_uvarint(input: &mut &[u8]) -> Result<u64, VarintError> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    for (i, &byte) in input.iter().enumerate() {
        if shift == 63 {
            // Nine groups are in; only bit 63 is left to place. The one canonical spelling of a
            // tenth byte is `1`: `0` pads a value that already fit, and anything above `1` would
            // set bits the `u64` does not have.
            return match byte {
                0 => Err(VarintError::Overlong),
                1 => {
                    *input = &input[i + 1..];
                    Ok(result | 1 << 63)
                }
                _ => Err(VarintError::Overflow),
            };
        }
        result |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            if i > 0 && byte == 0 {
                // The last byte of a multi-byte encoding contributed nothing, so this is a padded
                // spelling of a shorter one.
                return Err(VarintError::Overlong);
            }
            *input = &input[i + 1..];
            return Ok(result);
        }
        shift += 7;
    }
    Err(VarintError::UnexpectedEof)
}

/// Reads a zigzagged varint from `input`, advancing it past the bytes consumed.
pub fn read_ivarint(input: &mut &[u8]) -> Result<i64, VarintError> {
    read_uvarint(input).map(zigzag_decode)
}

#[cfg(test)]
mod test {
    use super::*;

    fn roundtrip_u(v: u64) -> Vec<u8> {
        let mut buf = Vec::new();
        write_uvarint(&mut buf, v);
        assert_eq!(
            buf.len(),
            uvarint_len(v),
            "uvarint_len disagrees with write_uvarint for {v}"
        );
        let mut cursor = buf.as_slice();
        assert_eq!(read_uvarint(&mut cursor), Ok(v), "roundtrip failed for {v}");
        assert!(cursor.is_empty(), "reader left bytes behind for {v}");
        buf
    }

    fn roundtrip_i(v: i64) -> Vec<u8> {
        let mut buf = Vec::new();
        write_ivarint(&mut buf, v);
        assert_eq!(
            buf.len(),
            ivarint_len(v),
            "ivarint_len disagrees with write_ivarint for {v}"
        );
        let mut cursor = buf.as_slice();
        assert_eq!(read_ivarint(&mut cursor), Ok(v), "roundtrip failed for {v}");
        assert!(cursor.is_empty(), "reader left bytes behind for {v}");
        buf
    }

    /// Every width boundary, plus the ends of the domain.
    #[test]
    fn roundtrips_unsigned_across_width_boundaries() {
        for v in [
            0,
            1,
            127,
            128,
            255,
            256,
            16_383,
            16_384,
            u32::MAX as u64,
            1 << 56,
            u64::MAX >> 1,
            u64::MAX - 1,
            u64::MAX,
        ] {
            roundtrip_u(v);
        }
        // The boundary is at every multiple of seven bits.
        for bits in 0..64 {
            roundtrip_u(1u64 << bits);
            roundtrip_u((1u64 << bits) - 1);
        }
    }

    #[test]
    fn roundtrips_signed_across_width_boundaries() {
        for v in [
            0,
            1,
            -1,
            63,
            -64,
            64,
            -65,
            i32::MIN as i64,
            i64::MIN,
            i64::MAX,
        ] {
            roundtrip_i(v);
        }
        for bits in 0..63 {
            roundtrip_i(1i64 << bits);
            roundtrip_i(-(1i64 << bits));
        }
    }

    /// Widths must be what the format budget assumes: one byte up to 127, ten at the very top.
    #[test]
    fn spends_bytes_where_expected() {
        assert_eq!(roundtrip_u(0).len(), 1);
        assert_eq!(roundtrip_u(127).len(), 1);
        assert_eq!(roundtrip_u(128).len(), 2);
        assert_eq!(roundtrip_u(1000).len(), 2);
        assert_eq!(roundtrip_u(u64::MAX).len(), MAX_UVARINT_LEN);
        // Zigzag keeps small negatives as cheap as small positives.
        assert_eq!(roundtrip_i(-1).len(), 1);
        assert_eq!(roundtrip_i(63).len(), 1);
        assert_eq!(roundtrip_i(-64).len(), 1);
        assert_eq!(roundtrip_i(64).len(), 2);
        // A millisecond timestamp, which the modified_at column pays for exactly once.
        assert_eq!(roundtrip_i(1_700_000_000_000).len(), 6);
    }

    #[test]
    fn zigzag_is_a_bijection() {
        for v in [0i64, 1, -1, i64::MIN, i64::MAX, 1234, -1234] {
            assert_eq!(zigzag_decode(zigzag_encode(v)), v);
        }
        for u in [0u64, 1, 2, 3, u64::MAX, u64::MAX - 1] {
            assert_eq!(zigzag_encode(zigzag_decode(u)), u);
        }
        // The mapping is order-preserving on magnitude, which is the whole point.
        assert_eq!(zigzag_encode(0), 0);
        assert_eq!(zigzag_encode(-1), 1);
        assert_eq!(zigzag_encode(1), 2);
        assert_eq!(zigzag_encode(-2), 3);
    }

    #[test]
    fn rejects_truncated_input() {
        assert_eq!(
            read_uvarint(&mut [].as_slice()),
            Err(VarintError::UnexpectedEof)
        );
        // Continuation bit set on every byte, then nothing.
        for len in 1..MAX_UVARINT_LEN {
            let buf = vec![0x80u8; len];
            assert_eq!(
                read_uvarint(&mut buf.as_slice()),
                Err(VarintError::UnexpectedEof),
                "{len} continuation bytes should not decode"
            );
        }
    }

    #[test]
    fn rejects_values_past_64_bits() {
        // Nine full groups then a tenth byte holding more than bit 63.
        let mut buf = vec![0xffu8; 9];
        buf.push(0x02);
        assert_eq!(
            read_uvarint(&mut buf.as_slice()),
            Err(VarintError::Overflow)
        );
        // An eleventh byte is past the point where any bits remain.
        let mut buf = vec![0xffu8; 10];
        buf.push(0x01);
        assert_eq!(
            read_uvarint(&mut buf.as_slice()),
            Err(VarintError::Overflow)
        );
    }

    #[test]
    fn rejects_overlong_encodings() {
        // `0` padded to two bytes, and to ten.
        assert_eq!(
            read_uvarint(&mut [0x80u8, 0x00].as_slice()),
            Err(VarintError::Overlong)
        );
        let buf = [0x80u8, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x00];
        assert_eq!(
            read_uvarint(&mut buf.as_slice()),
            Err(VarintError::Overlong)
        );
        // `1` padded by one byte.
        assert_eq!(
            read_uvarint(&mut [0x81u8, 0x00].as_slice()),
            Err(VarintError::Overlong)
        );
        // A genuine two-byte value is of course fine.
        assert_eq!(read_uvarint(&mut [0x80u8, 0x01].as_slice()), Ok(128));
    }

    /// Readers advance the cursor by exactly what they consumed, which is what makes a column a
    /// sequence of reads rather than an offset calculation.
    #[test]
    fn advances_the_cursor_past_each_value() {
        let mut buf = Vec::new();
        write_uvarint(&mut buf, 1);
        write_uvarint(&mut buf, 300);
        write_ivarint(&mut buf, -7);
        buf.push(0xaa); // trailing byte the reader must not touch

        let mut cursor = buf.as_slice();
        assert_eq!(read_uvarint(&mut cursor), Ok(1));
        assert_eq!(read_uvarint(&mut cursor), Ok(300));
        assert_eq!(read_ivarint(&mut cursor), Ok(-7));
        assert_eq!(cursor, [0xaa]);
    }
}
