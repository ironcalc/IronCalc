//! Transport-level compression of large frames.
//!
//! The y-sync handshake ships a room's full state in one message; for a big
//! workbook that is tens of MB of highly repetitive yrs v1 bytes (about 30 MB
//! per million cells, gzip shrinks it 6-7x). Tungstenite has no
//! permessage-deflate, so both ends of the relay agree on a custom y-sync
//! message instead: a frame at least [`COMPRESS_THRESHOLD`] bytes long is
//! sent as `Message::Custom(MSG_GZIP, gzip(frame))`, on its own in the
//! websocket message. Small frames travel as they are. The browser client
//! (`webapp/IronCalc/src/collab/CollabProvider.ts`) speaks the same wrapper
//! with the platform's `CompressionStream`/`DecompressionStream`.
//!
//! The wrapper is purely a transport concern: the CRDT peers never see it.

use std::borrow::Cow;
use std::io::{Read, Write};

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use yrs::sync::Message;
use yrs::updates::decoder::{Decode, DecoderV1};
use yrs::updates::encoder::Encode;

/// Custom y-sync message tag of a gzip-wrapped frame (standard tags are
/// 0..=3; anything else is `Message::Custom`).
pub const MSG_GZIP: u8 = 0x10;

/// Frames shorter than this are sent uncompressed.
pub const COMPRESS_THRESHOLD: usize = 64 * 1024;

/// Wraps `frame` in a gzip custom message when it is large enough to be
/// worth it; smaller frames (and frames gzip cannot handle) pass through.
pub fn wrap_frame(frame: Vec<u8>) -> Vec<u8> {
    if frame.len() < COMPRESS_THRESHOLD {
        return frame;
    }
    let mut encoder = GzEncoder::new(Vec::with_capacity(frame.len() / 4), Compression::fast());
    if encoder.write_all(&frame).is_err() {
        return frame;
    }
    match encoder.finish() {
        Ok(compressed) => Message::Custom(MSG_GZIP, compressed).encode_v1(),
        Err(_) => frame,
    }
}

/// Inflates a gzip-wrapped frame; any other frame is returned as is. The
/// inflated size is capped at `max_len` bytes (a decompression bomb would
/// otherwise exhaust memory).
pub fn unwrap_frame(frame: &[u8], max_len: usize) -> Result<Cow<'_, [u8]>, String> {
    if frame.first() != Some(&MSG_GZIP) {
        return Ok(Cow::Borrowed(frame));
    }
    let mut decoder = DecoderV1::from(frame);
    let message = Message::decode(&mut decoder).map_err(|e| format!("bad gzip frame: {e}"))?;
    let Message::Custom(MSG_GZIP, compressed) = message else {
        return Ok(Cow::Borrowed(frame));
    };
    let mut inflated = Vec::new();
    GzDecoder::new(compressed.as_slice())
        .take(max_len as u64 + 1)
        .read_to_end(&mut inflated)
        .map_err(|e| format!("bad gzip payload: {e}"))?;
    if inflated.len() > max_len {
        return Err(format!("inflated frame exceeds {max_len} bytes"));
    }
    Ok(Cow::Owned(inflated))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;
    use yrs::sync::SyncMessage;

    fn update_frame(payload: Vec<u8>) -> Vec<u8> {
        Message::Sync(SyncMessage::Update(payload)).encode_v1()
    }

    #[test]
    fn small_frames_pass_through_untouched() {
        let frame = update_frame(vec![7; 100]);
        assert_eq!(wrap_frame(frame.clone()), frame);
        assert_eq!(
            unwrap_frame(&frame, 1 << 20).unwrap().as_ref(),
            frame.as_slice()
        );
    }

    #[test]
    fn large_frames_round_trip_through_gzip() {
        // Repetitive like a real update: compresses well.
        let payload: Vec<u8> = (0..COMPRESS_THRESHOLD * 4)
            .map(|i| (i % 13) as u8)
            .collect();
        let frame = update_frame(payload);
        let wrapped = wrap_frame(frame.clone());
        assert_eq!(wrapped[0], MSG_GZIP);
        assert!(
            wrapped.len() < frame.len() / 4,
            "gzip did not shrink the frame"
        );
        assert_eq!(
            unwrap_frame(&wrapped, 1 << 24).unwrap().as_ref(),
            frame.as_slice()
        );
    }

    #[test]
    fn oversized_payloads_are_rejected() {
        let frame = update_frame(vec![0; COMPRESS_THRESHOLD * 2]);
        let wrapped = wrap_frame(frame);
        assert!(unwrap_frame(&wrapped, COMPRESS_THRESHOLD).is_err());
    }

    #[test]
    fn garbage_with_the_gzip_tag_is_an_error() {
        // Tag, length 3, three bytes that are not a gzip stream.
        assert!(unwrap_frame(&[MSG_GZIP, 3, 1, 2, 3], 1 << 20).is_err());
    }
}
