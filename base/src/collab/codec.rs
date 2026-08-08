//! Column-oriented codec for the two [`Entry`] spaces of a
//! [`FractionalIndex`](crate::collab::fractional_index::FractionalIndex).
//!
//! An index holds one [`Entry`] per row, column, sheet and conditional-formatting rule, and the
//! whole thing is serialized on every snapshot. Entry by entry, each one is two keys and an
//! [`Hlc`], and a general-purpose coder can only compress what it happens to see side by side —
//! fields of unrelated shape.
//!
//! Written *column by column*, the same data is almost entirely redundant, and the redundancy is a
//! consequence of how the index is built rather than a hope about the data:
//!
//! * one session mints long runs of keys, so the session suffixes repeat
//!   ([`CreateKeys`](crate::collab::fractional_index::CreateKeys) is driven by a single replica),
//! * a bulk run walks its positions by a fixed stride, so consecutive positions are one addition
//!   apart,
//! * one operation stamps its whole batch with one `modified_at`,
//! * most entries were never moved, so their `moved` field is [`FractionalKey::NULL`].
//!
//! Measured against `bitcode::serialize` over the same `Vec<Entry>`, for keys from a single
//! `create_keys` call:
//!
//! ```text
//! entries   columnar   bitcode derive
//!      10       23 B            91 B
//!     100       23 B           754 B
//!   1_000       28 B         7_393 B
//!  10_000       28 B        73_768 B
//! ```
//!
//! The run marker is why the last two rows are the same size: a run costs the same whether it covers
//! a thousand entries or ten thousand. In the *worst* case — every position, session and timestamp
//! distinct, every entry moved — a thousand entries cost 15.0 bytes each here against 16.3 through
//! the derive, so the layout does not lose on incompressible data either.
//!
//! # Layout
//!
//! One column group per field of [`Entry`], in field order:
//!
//! ```text
//! table := count:uvarint ++ if count > 0 { key ++ modified_at ++ moved }
//!
//! key         := key_positions ++ sessions
//! modified_at := one absolute value, then deltas
//! moved       := moved_positions ++ sessions
//! ```
//!
//! A key is a position and a session suffix, and each half gets its own sub-column because each
//! compresses for a different reason. Positions come first: a `moved` field's sessions are only as
//! many as it has non-null positions, and that count is not known until the positions are read.
//!
//! Every column is read until it has accounted for exactly `count` entries — `sessions` under
//! `moved` for the non-null ones only. None carries a byte length of its own.
//!
//! ```text
//! key_positions   := code:uvarint
//!                      code == 0  -> delta:uvarint ++ run:uvarint
//!                                    position[i+k] = position[i+k-1] + delta, big-endian
//!                      code >= 1  -> len = code - 1, then len raw bytes
//!
//! moved_positions := code:uvarint
//!                      code == 0  -> run:uvarint, entries whose `moved` is FractionalKey::NULL
//!                      code >= 1  -> len = code - 1, then len raw bytes
//!
//! sessions        := session:[u8; SESSION_SUFFIX_LEN] ++ run:uvarint
//!
//! modified_at     := slot 0: value:ivarint, absolute
//!                    slot i: delta:ivarint, over the raw `Hlc` payloads
//!                      delta == 0 -> run:uvarint, entries reusing modified_at[i-1]
//!                      delta != 0 -> modified_at[i] = modified_at[i-1] + delta
//! ```
//!
//! Three details of that are load-bearing:
//!
//! * A position literal is `len + 1`, not `len`. That keeps `0` free as the marker *and* leaves an
//!   empty position representable, so the codec is total over every key an [`Entry`] can hold. It
//!   costs nothing: positions run to 251 bytes, and any position up to 126 still codes in one byte.
//! * The run marker carries its `delta` explicitly rather than implying `1`. `CreateKeys` strides by
//!   **2** whenever the gap has room, deliberately, so that a later insert between two run keys
//!   stays inline — an implied `+1` would miss the bulk-insert path entirely. An explicit delta also
//!   covers `delta == 0`, which is two sessions minting at one position.
//! * Slot 0 of `modified_at` is unconditionally absolute and has no marker semantics. Real clock
//!   values are never `0`, but that is an invariant of the producer, and this decoder reads a peer's
//!   bytes; exempting the one slot that has nothing to repeat removes the reliance for free.
//! * A decoded column is folded into the local clock ([`Hlc::sync`]): a stamp read off a snapshot or
//!   a peer is a stamp this replica has seen, and the ones it mints next have to sort above it. An
//!   [`Hlc`] payload is a `u64`, but the column carries it as an `i64` — free until the year 4.4
//!   million, and it keeps a corrupt payload from decoding into a far-future stamp that the clock
//!   would then be stuck with.
//!
//! # Reading untrusted bytes
//!
//! Decoding is the path peer and persisted data travel, so it never panics, and it never allocates
//! on an unvalidated number. Every encoding is canonical and the alternatives are rejected — see
//! [`CodecError`] — which is what lets the tests assert `encode(decode(encode(x))) == encode(x)`
//! byte for byte. That is a much sharper check than value equality: it fails when a compression rule
//! silently stops firing.

use crate::collab::fractional_index::Entry;
use crate::collab::fractional_key::{FractionalKey, KeyBuf, MAX_KEY_LEN, SESSION_SUFFIX_LEN};
use crate::collab::hlc::Hlc;
use crate::collab::varint::{
    read_ivarint, read_uvarint, uvarint_len, write_ivarint, write_uvarint, VarintError,
};
use std::fmt;

/// Version written at the head of an encoded index. Bump it for any change to the layout above.
pub const FORMAT_VERSION: u8 = 1;

/// The longest position the format can carry: a key is a position plus its session suffix, and the
/// whole key has to fit in [`MAX_KEY_LEN`].
pub const MAX_POSITION_LEN: usize = MAX_KEY_LEN - SESSION_SUFFIX_LEN;

/// Code `0` opens a run in every column, and every column's reader dispatches on it the same way.
/// What a run *means* is per-column, hence the three names below — but there is one value, so the
/// columns cannot drift apart.
const MARKER: u64 = 0;

/// A `0` code in a `key` position column: the next `run` positions each step the previous one by
/// `delta`.
const RUN_MARKER: u64 = MARKER;

/// A `0` code in a `moved` position column: the next `run` entries carry [`FractionalKey::NULL`].
const NULL_MARKER: u64 = MARKER;

/// A `0` delta in the `modified_at` column: the next `run` entries reuse the previous timestamp.
const REPEAT_MARKER: i64 = MARKER as i64;

/// Why a payload could not be encoded or decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    /// The input ended mid-value.
    UnexpectedEof,
    /// A varint carried bits past the width of a `u64`.
    VarintOverflow,
    /// A varint was spelled with more bytes than the value needs.
    VarintOverlong,
    /// An entry held a key that is neither [`FractionalKey::NULL`] nor a minted
    /// `position ++ session`. Only reachable by hand-building an [`Entry`].
    MalformedKey { len: usize },
    /// A position literal longer than [`MAX_POSITION_LEN`].
    PositionTooLong { len: u64 },
    /// A run stepped a position past the width it has to fit in.
    PositionOverflow,
    /// An [`Hlc`] payload, or the delta between two of them, did not fit in an `i64`.
    TimestampOverflow,
    /// A run or repeat marker opened a column, with no previous value to continue from.
    RunWithoutPredecessor,
    /// A run of zero entries. Every value has one spelling, and this is not it.
    EmptyRun,
    /// A run claimed more entries than the column still owes.
    RunOverflow { run: u64, remaining: usize },
    /// Keys are not strictly increasing, so the binary searches over this space would misbehave.
    UnsortedKeys { at: usize },
    /// The version byte names a layout this build does not know.
    UnsupportedVersion(u8),
    /// Bytes left over after a complete index.
    TrailingBytes(usize),
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodecError::UnexpectedEof => write!(f, "input ended in the middle of a value"),
            CodecError::VarintOverflow => write!(f, "varint does not fit in 64 bits"),
            CodecError::VarintOverlong => write!(f, "varint is not encoded canonically"),
            CodecError::MalformedKey { len } => {
                write!(f, "key of {len} bytes is neither null nor a minted key")
            }
            CodecError::PositionTooLong { len } => write!(
                f,
                "position of {len} bytes exceeds the {MAX_POSITION_LEN} byte limit"
            ),
            CodecError::PositionOverflow => {
                write!(f, "run stepped a position past the width it must fit in")
            }
            CodecError::TimestampOverflow => {
                write!(
                    f,
                    "timestamp, or a delta between two, does not fit in an i64"
                )
            }
            CodecError::RunWithoutPredecessor => {
                write!(
                    f,
                    "run marker opens a column, with nothing to continue from"
                )
            }
            CodecError::EmptyRun => write!(f, "run covers zero entries"),
            CodecError::RunOverflow { run, remaining } => write!(
                f,
                "run of {run} entries overruns the {remaining} the column still owes"
            ),
            CodecError::UnsortedKeys { at } => {
                write!(f, "key at index {at} does not sort above its predecessor")
            }
            CodecError::UnsupportedVersion(v) => write!(f, "unsupported format version {v}"),
            CodecError::TrailingBytes(n) => write!(f, "{n} bytes left over after the payload"),
        }
    }
}

impl std::error::Error for CodecError {}

impl From<VarintError> for CodecError {
    fn from(e: VarintError) -> Self {
        match e {
            VarintError::UnexpectedEof => CodecError::UnexpectedEof,
            VarintError::Overflow => CodecError::VarintOverflow,
            VarintError::Overlong => CodecError::VarintOverlong,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Framing
//
// One step per field of `Entry`, in field order. Each field's codec owns its own sub-columns, so
// these two functions are the whole of what the reader and the writer have to agree about.
// ---------------------------------------------------------------------------------------------

/// Appends `entries` to `out` as a column table.
///
/// Fails only on an [`Entry`] the index cannot have produced (see [`CodecError::MalformedKey`]) or a
/// timestamp pair no clock can produce ([`CodecError::TimestampOverflow`]).
pub fn encode_entries(entries: &[Entry], out: &mut Vec<u8>) -> Result<(), CodecError> {
    // Validate once up front, so the column passes below can split keys without re-checking.
    for e in entries {
        // `key` always names a position: it was either minted here or decoded from a payload that
        // was, and both produce `position ++ session`.
        if e.key.len() < SESSION_SUFFIX_LEN {
            return Err(CodecError::MalformedKey { len: e.key.len() });
        }
        // `moved` is that, or null. A length in between is neither.
        if (1..SESSION_SUFFIX_LEN).contains(&e.moved.len()) {
            return Err(CodecError::MalformedKey { len: e.moved.len() });
        }
    }

    write_uvarint(out, entries.len() as u64);
    if entries.is_empty() {
        return Ok(());
    }
    write_key_column(out, entries);
    write_modified_at_column(out, entries)?;
    write_moved_column(out, entries);
    Ok(())
}

/// Reads a column table from `input`, advancing it past the bytes consumed.
pub fn decode_entries(input: &mut &[u8]) -> Result<Vec<Entry>, CodecError> {
    let count = read_uvarint(input)?;
    let count = count as usize;
    if count == 0 {
        return Ok(Vec::new());
    }

    let keys = read_key_column(input, count)?;
    let modified_at = read_modified_at_column(input, count)?;
    let moved = read_moved_column(input, count)?;
    // Every reader loops until its column has accounted for exactly `count` entries, so the zip
    // below cannot truncate. Asserted rather than assumed, because a reader that came back short
    // would silently produce a smaller index instead of failing.
    debug_assert!(keys.len() == count && modified_at.len() == count && moved.len() == count);

    let entries: Vec<Entry> = keys
        .into_iter()
        .zip(modified_at)
        .zip(moved)
        .map(|((key, modified_at), moved)| Entry {
            key,
            modified_at,
            moved,
        })
        .collect();
    check_strictly_increasing(&entries)?;
    Ok(entries)
}

/// Both spaces are binary-searched on every lookup, insert and merge, so keys that do not ascend are
/// corruption rather than something to quietly sort out here.
fn check_strictly_increasing(entries: &[Entry]) -> Result<(), CodecError> {
    for (i, pair) in entries.windows(2).enumerate() {
        if pair[0].key >= pair[1].key {
            return Err(CodecError::UnsortedKeys { at: i + 1 });
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// `key`: a position sub-column, then a session sub-column
// ---------------------------------------------------------------------------------------------

fn write_key_column(out: &mut Vec<u8>, entries: &[Entry]) {
    write_key_positions(out, entries);
    write_session_column(out, entries.iter().map(|e| session_of(&e.key)));
}

fn read_key_column(input: &mut &[u8], count: usize) -> Result<Vec<FractionalKey>, CodecError> {
    let positions = read_key_positions(input, count)?;
    let sessions = read_session_column(input, count)?;
    positions
        .iter()
        .zip(&sessions)
        .map(|(position, session)| join(position, session))
        .collect()
}

fn write_key_positions(out: &mut Vec<u8>, entries: &[Entry]) {
    // A run replays a step from the position before it, so the column always opens with a literal —
    // `previous` is `None` only at index 0.
    let mut previous: Option<&[u8]> = None;
    let mut i = 0;
    while i < entries.len() {
        let position = position_of(&entries[i].key);
        let Some(delta) = previous.and_then(|p| be_delta(p, position)) else {
            write_position_literal(out, position);
            previous = Some(position);
            i += 1;
            continue;
        };

        // Spend the marker only when it does not cost more than the literals it replaces: a
        // single-entry run beats a three-byte position but loses to a one-byte one. Every member of
        // a run shares one length, since `be_delta` only pairs positions of equal width.
        let run = run_length(entries, i, delta);
        if run_cost(delta, run) <= literal_cost(position.len(), run) {
            write_uvarint(out, RUN_MARKER);
            write_uvarint(out, delta);
            write_uvarint(out, run);
        } else {
            for e in &entries[i..i + run as usize] {
                write_position_literal(out, position_of(&e.key));
            }
        }
        i += run as usize;
        previous = Some(position_of(&entries[i - 1].key));
    }
}

fn read_key_positions(input: &mut &[u8], count: usize) -> Result<Vec<KeyBuf>, CodecError> {
    let mut out: Vec<KeyBuf> = Vec::with_capacity(count.min(input.len()));
    while out.len() < count {
        let code = read_uvarint(input)?;
        if code == RUN_MARKER {
            let delta = read_uvarint(input)?;
            let run = read_run_length(input, count - out.len())?;
            for _ in 0..run {
                let previous = out.last().ok_or(CodecError::RunWithoutPredecessor)?;
                let next = be_add(previous, delta)?;
                out.push(next);
            }
        } else {
            let len = position_len(code)?;
            out.push(KeyBuf::from(read_bytes(input, len)?));
        }
    }
    Ok(out)
}

/// How many entries from `start` a run of constant `delta` covers, the one at `start` included.
fn run_length(entries: &[Entry], start: usize, delta: u64) -> u64 {
    let mut run = 1;
    while let (Some(previous), Some(next)) =
        (entries.get(start + run - 1), entries.get(start + run))
    {
        if be_delta(position_of(&previous.key), position_of(&next.key)) != Some(delta) {
            break;
        }
        run += 1;
    }
    run as u64
}

/// Bytes a run marker spends: one for the code, then the step and the length.
fn run_cost(delta: u64, run: u64) -> usize {
    1 + uvarint_len(delta) + uvarint_len(run)
}

/// Bytes the literals a run would replace spend. All of them share one length — see `be_delta`.
fn literal_cost(position_len: usize, run: u64) -> usize {
    run as usize * (uvarint_len(position_len as u64 + 1) + position_len)
}

// ---------------------------------------------------------------------------------------------
// `modified_at`: one absolute value, then deltas
// ---------------------------------------------------------------------------------------------

/// Signed distance between two stamps, as the column carries it.
fn stamp_delta(previous: Hlc, next: Hlc) -> Result<i64, CodecError> {
    i64::try_from(next.get() as i128 - previous.get() as i128)
        .map_err(|_| CodecError::TimestampOverflow)
}

fn write_modified_at_column(out: &mut Vec<u8>, entries: &[Entry]) -> Result<(), CodecError> {
    // Slot 0 is absolute and carries no marker semantics: there is nothing to repeat and nothing to
    // be a delta from, so a `0` here is simply a timestamp of zero.
    write_ivarint(
        out,
        i64::try_from(entries[0].modified_at.get()).map_err(|_| CodecError::TimestampOverflow)?,
    );
    let mut i = 1;
    while i < entries.len() {
        let previous = entries[i - 1].modified_at;
        if entries[i].modified_at == previous {
            let start = i;
            while i < entries.len() && entries[i].modified_at == previous {
                i += 1;
            }
            write_ivarint(out, REPEAT_MARKER);
            write_uvarint(out, (i - start) as u64);
        } else {
            write_ivarint(out, stamp_delta(previous, entries[i].modified_at)?);
            i += 1;
        }
    }
    Ok(())
}

fn read_modified_at_column(input: &mut &[u8], count: usize) -> Result<Vec<Hlc>, CodecError> {
    debug_assert!(count > 0);
    let mut out = Vec::with_capacity(count.min(input.len()).max(1));
    let absolute =
        u64::try_from(read_ivarint(input)?).map_err(|_| CodecError::TimestampOverflow)?;
    out.push(Hlc::new(absolute));
    while out.len() < count {
        let delta = read_ivarint(input)?;
        let previous = *out.last().expect("slot 0 is pushed above");
        if delta == REPEAT_MARKER {
            let run = read_run_length(input, count - out.len())?;
            out.extend(std::iter::repeat_n(previous, run));
        } else {
            let next = u64::try_from(previous.get() as i128 + delta as i128)
                .map_err(|_| CodecError::TimestampOverflow)?;
            out.push(Hlc::new(next));
        }
    }
    // Receive rule: the stamps of a loaded index are stamps this replica has now seen. Syncing the
    // maximum is the same as syncing every one of them.
    if let Some(max) = out.iter().max() {
        Hlc::sync(*max);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// `moved`: a position sub-column, then a session sub-column for the non-null positions only
// ---------------------------------------------------------------------------------------------

fn write_moved_column(out: &mut Vec<u8>, entries: &[Entry]) {
    write_moved_positions(out, entries);
    write_session_column(
        out,
        entries
            .iter()
            .filter(|e| !e.moved.is_empty())
            .map(|e| session_of(&e.moved)),
    );
}

fn read_moved_column(input: &mut &[u8], count: usize) -> Result<Vec<FractionalKey>, CodecError> {
    let positions = read_moved_positions(input, count)?;
    // A null `moved` carries no session, so this sub-column is only as long as the positions that
    // need one — which is why the positions come first.
    let needed = positions.iter().filter(|p| p.is_some()).count();
    let mut sessions = read_session_column(input, needed)?.into_iter();
    positions
        .iter()
        .map(|position| match position {
            None => Ok(FractionalKey::NULL),
            Some(position) => match sessions.next() {
                Some(session) => join(position, &session),
                // Unreachable: the sub-column was read to exactly the length `needed` counted.
                None => Err(CodecError::UnexpectedEof),
            },
        })
        .collect()
}

/// Move destinations point at arbitrary positions rather than walking, so this sub-column gets the
/// null run but no delta run — one would essentially never fire.
fn write_moved_positions(out: &mut Vec<u8>, entries: &[Entry]) {
    let mut i = 0;
    while i < entries.len() {
        if entries[i].moved.is_empty() {
            let start = i;
            while i < entries.len() && entries[i].moved.is_empty() {
                i += 1;
            }
            write_uvarint(out, NULL_MARKER);
            write_uvarint(out, (i - start) as u64);
        } else {
            write_position_literal(out, position_of(&entries[i].moved));
            i += 1;
        }
    }
}

fn read_moved_positions(
    input: &mut &[u8],
    count: usize,
) -> Result<Vec<Option<KeyBuf>>, CodecError> {
    let mut out: Vec<Option<KeyBuf>> = Vec::with_capacity(count.min(input.len()));
    while out.len() < count {
        let code = read_uvarint(input)?;
        if code == NULL_MARKER {
            let run = read_run_length(input, count - out.len())?;
            out.extend(std::iter::repeat_n(None, run));
        } else {
            let len = position_len(code)?;
            out.push(Some(KeyBuf::from(read_bytes(input, len)?)));
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// The session sub-column, shared by `key` and `moved`
// ---------------------------------------------------------------------------------------------

fn write_session_column<'a>(out: &mut Vec<u8>, mut sessions: impl Iterator<Item = &'a [u8]>) {
    let Some(first) = sessions.next() else { return };
    let mut current = first;
    let mut run: u64 = 1;
    for session in sessions {
        if session == current {
            run += 1;
            continue;
        }
        out.extend_from_slice(current);
        write_uvarint(out, run);
        current = session;
        run = 1;
    }
    out.extend_from_slice(current);
    write_uvarint(out, run);
}

fn read_session_column(
    input: &mut &[u8],
    count: usize,
) -> Result<Vec<[u8; SESSION_SUFFIX_LEN]>, CodecError> {
    let mut out = Vec::with_capacity(count.min(input.len()));
    while out.len() < count {
        let mut session = [0u8; SESSION_SUFFIX_LEN];
        session.copy_from_slice(read_bytes(input, SESSION_SUFFIX_LEN)?);
        let run = read_run_length(input, count - out.len())?;
        out.extend(std::iter::repeat_n(session, run));
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// Splitting and rejoining keys
// ---------------------------------------------------------------------------------------------

/// Position half of a key `encode_entries` has already accepted.
#[inline]
fn position_of(key: &FractionalKey) -> &[u8] {
    debug_assert!(key.len() >= SESSION_SUFFIX_LEN);
    key.split().0
}

/// Session half of a key `encode_entries` has already accepted.
#[inline]
fn session_of(key: &FractionalKey) -> &[u8] {
    debug_assert!(key.len() >= SESSION_SUFFIX_LEN);
    key.split().1
}

/// Rebuilds a key from the two halves its sub-columns carry.
fn join(position: &[u8], session: &[u8; SESSION_SUFFIX_LEN]) -> Result<FractionalKey, CodecError> {
    let mut buf = KeyBuf::with_capacity(position.len() + SESSION_SUFFIX_LEN);
    buf.extend_from_slice(position);
    buf.extend_from_slice(session);
    // Literals are bounded at `MAX_POSITION_LEN` and `be_add` never widens a position, so this
    // cannot actually fail — but these are a peer's bytes, so it is checked rather than asserted.
    FractionalKey::try_from_bytes(&buf).map_err(|_| CodecError::PositionTooLong {
        len: position.len() as u64,
    })
}

/// Writes a position as `len + 1` followed by its bytes. See the [module docs](self) for why the
/// length is biased.
fn write_position_literal(out: &mut Vec<u8>, position: &[u8]) {
    debug_assert!(position.len() <= MAX_POSITION_LEN);
    write_uvarint(out, position.len() as u64 + 1);
    out.extend_from_slice(position);
}

/// Length a position literal's biased code names.
fn position_len(code: u64) -> Result<usize, CodecError> {
    debug_assert!(code > 0, "the marker is handled by the caller");
    let len = code - 1;
    if len > MAX_POSITION_LEN as u64 {
        return Err(CodecError::PositionTooLong { len });
    }
    Ok(len as usize)
}

// ---------------------------------------------------------------------------------------------
// Reading primitives
// ---------------------------------------------------------------------------------------------

/// Reads a run length, bounded by what the column still owes.
///
/// That bound is what keeps a run from allocating past the entry count, which [`MAX_ENTRIES`] has
/// already vetted.
fn read_run_length(input: &mut &[u8], remaining: usize) -> Result<usize, CodecError> {
    let run = read_uvarint(input)?;
    if run == 0 {
        return Err(CodecError::EmptyRun);
    }
    if run > remaining as u64 {
        return Err(CodecError::RunOverflow { run, remaining });
    }
    Ok(run as usize)
}

fn read_bytes<'a>(input: &mut &'a [u8], len: usize) -> Result<&'a [u8], CodecError> {
    if input.len() < len {
        return Err(CodecError::UnexpectedEof);
    }
    let (head, tail) = input.split_at(len);
    *input = tail;
    Ok(head)
}

// ---------------------------------------------------------------------------------------------
// Big-endian position arithmetic
// ---------------------------------------------------------------------------------------------

/// The constant step from `previous` to `next`, or `None` if one position is not the other stepped
/// up at the same width.
///
/// Positions of unequal length can never share a run: the decoder replays a step without changing
/// the width, so a widening would not round-trip.
fn be_delta(previous: &[u8], next: &[u8]) -> Option<u64> {
    if previous.len() != next.len() {
        return None;
    }
    // A step is carried in a `u64`, so only the trailing eight bytes may differ.
    let head = previous.len().saturating_sub(8);
    if previous[..head] != next[..head] {
        return None;
    }
    be_u64(&next[head..]).checked_sub(be_u64(&previous[head..]))
}

/// `previous` stepped up by `delta` at the same width. This is the codec's counterpart to the
/// `add_assign` that [`CreateKeys`](crate::collab::fractional_index::CreateKeys) walks a run with.
fn be_add(previous: &[u8], delta: u64) -> Result<KeyBuf, CodecError> {
    let head = previous.len().saturating_sub(8);
    let tail = previous.len() - head;
    let sum = be_u64(&previous[head..])
        .checked_add(delta)
        .ok_or(CodecError::PositionOverflow)?;
    // The width is fixed, so a carry off the front of the tail has nowhere to go. Guarding on
    // `tail < 8` first also keeps the shift below its width.
    if tail < 8 && sum >> (8 * tail) != 0 {
        return Err(CodecError::PositionOverflow);
    }
    let mut next = KeyBuf::with_capacity(previous.len());
    next.extend_from_slice(&previous[..head]);
    next.extend_from_slice(&sum.to_be_bytes()[8 - tail..]);
    Ok(next)
}

/// Reads up to eight big-endian bytes as a `u64`.
fn be_u64(bytes: &[u8]) -> u64 {
    debug_assert!(bytes.len() <= 8);
    bytes.iter().fold(0u64, |acc, &b| acc << 8 | b as u64)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::collab::fractional_index::FractionalIndex;

    /// A key as the index mints them: a position with a four-byte session glued on.
    fn key(position: &[u8], session: u8) -> FractionalKey {
        let mut bytes = position.to_vec();
        bytes.extend_from_slice(&[session, 0, 0, 0]);
        FractionalKey::from(bytes.as_slice())
    }

    fn entry(position: &[u8], session: u8, modified_at: Hlc) -> Entry {
        Entry {
            key: key(position, session),
            modified_at,
            moved: FractionalKey::NULL,
        }
    }

    /// Round-trips `entries`, asserting both value equality and byte-level canonicality, and returns
    /// the encoding so callers can make claims about its size.
    fn roundtrip(entries: &[Entry]) -> Vec<u8> {
        let mut encoded = Vec::new();
        encode_entries(entries, &mut encoded).expect("encoding should succeed");

        let mut cursor = encoded.as_slice();
        let decoded = decode_entries(&mut cursor).expect("decoding should succeed");
        assert!(cursor.is_empty(), "decoder left {} bytes", cursor.len());
        assert_eq!(decoded.as_slice(), entries, "value round-trip failed");

        // The encoder picks one spelling per value, so re-encoding what we just read has to
        // reproduce the same bytes. This is what fails if a compression rule stops firing.
        let mut again = Vec::new();
        encode_entries(&decoded, &mut again).expect("re-encoding should succeed");
        assert_eq!(again, encoded, "encoding is not canonical");

        encoded
    }

    /// The entries of a fresh index holding a bulk-generated run — the shape this format exists for.
    fn bulk_run(count: usize, modified_at: Hlc) -> Vec<Entry> {
        let index = FractionalIndex::new(vec![], vec![], [b'a', 0, 0, 0]);
        index
            .create_keys(0, count)
            .map(|key| Entry {
                key,
                modified_at,
                moved: FractionalKey::NULL,
            })
            .collect()
    }

    #[test]
    fn encodes_an_empty_space_as_one_byte() {
        let encoded = roundtrip(&[]);
        assert_eq!(encoded, [0]);
    }

    #[test]
    fn roundtrips_a_single_entry() {
        roundtrip(&[entry(&[0x01], 7, Hlc::new(1_700_000_000_000))]);
        // An empty position is representable, which is what the biased literal length buys.
        roundtrip(&[entry(&[], 7, Hlc::new(1))]);
        // As is the widest position a key can hold.
        roundtrip(&[entry(&[0x5a; MAX_POSITION_LEN], 7, Hlc::new(2))]);
    }

    /// The headline case: a thousand keys from one `create_keys` call, one session, one timestamp,
    /// nothing moved. This is the test that fails if the run marker stops matching `CreateKeys`.
    #[test]
    fn collapses_a_bulk_generated_run() {
        let entries = bulk_run(1000, Hlc::new(1_700_000_000_000));
        assert_eq!(entries.len(), 1000);
        let encoded = roundtrip(&entries);
        assert!(
            encoded.len() < 64,
            "1000 bulk keys should collapse to a few dozen bytes, got {}",
            encoded.len()
        );
        // Entry by entry the same data is at least 17 bytes each.
        assert!(encoded.len() * 100 < entries.len() * 17);
    }

    /// `CreateKeys` strides by 2 in an open gap and by 1 in a tight one, so both have to collapse —
    /// and so does any other constant step, since the marker carries its delta.
    #[test]
    fn collapses_runs_of_either_stride() {
        for stride in [1u32, 2, 7, 1000] {
            let entries: Vec<Entry> = (0..100u32)
                .map(|i| {
                    // A three-byte position walked as a big-endian counter, exactly as `CreateKeys`
                    // walks one.
                    let v = 0x40_0000 + i * stride;
                    entry(
                        &[(v >> 16) as u8, (v >> 8) as u8, v as u8],
                        3,
                        Hlc::new(500),
                    )
                })
                .collect();
            let encoded = roundtrip(&entries);
            assert!(
                encoded.len() < 32,
                "stride {stride} should collapse, got {} bytes",
                encoded.len()
            );
        }
    }

    /// Two sessions minting at one position give equal positions and a zero step, which is a run
    /// like any other. The session column has to break instead.
    #[test]
    fn roundtrips_repeated_positions_across_sessions() {
        let entries = vec![
            entry(&[0x10], 1, Hlc::new(100)),
            entry(&[0x10], 2, Hlc::new(100)),
            entry(&[0x10], 3, Hlc::new(100)),
            entry(&[0x20], 1, Hlc::new(100)),
        ];
        roundtrip(&entries);
    }

    /// Positions wider than a `u64` only run when their leading bytes agree, since the step is
    /// carried in a `u64`.
    #[test]
    fn roundtrips_positions_past_the_step_width() {
        let mut entries = Vec::new();
        for i in 0..8u8 {
            // Twelve bytes: four of shared head, then a stepping tail.
            let mut position = vec![0x01, 0x02, 0x03, 0x04];
            position.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, i * 3]);
            entries.push(entry(&position, 1, Hlc::new(1000)));
        }
        // A differing head breaks the run rather than corrupting it.
        entries.push(entry(
            &[0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            1,
            Hlc::new(1000),
        ));
        roundtrip(&entries);
    }

    /// A run may not widen a position, so a length change has to break it.
    #[test]
    fn roundtrips_positions_of_mixed_width() {
        let entries = vec![
            entry(&[0x01], 1, Hlc::new(10)),
            entry(&[0x01, 0x00, 0x01], 1, Hlc::new(10)),
            entry(&[0x01, 0x00, 0x02], 1, Hlc::new(10)),
            entry(&[0x02], 1, Hlc::new(10)),
            entry(&[0x03, 0x04], 1, Hlc::new(10)),
        ];
        roundtrip(&entries);
    }

    #[test]
    fn roundtrips_every_shape_of_moved_column() {
        let mut base = bulk_run(16, Hlc::new(900));
        // Nothing moved: one NULL run, and no `moved` session column at all.
        roundtrip(&base);

        // Everything moved.
        let destinations: Vec<FractionalKey> =
            (0..16u8).map(|i| key(&[0xf0, 0x00, i], 9)).collect();
        for (e, dest) in base.iter_mut().zip(&destinations) {
            e.moved = dest.clone();
        }
        roundtrip(&base);

        // Alternating, which is the worst case for both markers.
        for (i, e) in base.iter_mut().enumerate() {
            if i % 2 == 0 {
                e.moved = FractionalKey::NULL;
            }
        }
        roundtrip(&base);

        // A `moved` key whose session differs from its own key's, so the two session columns cannot
        // be confused for one another.
        base[1].moved = key(&[0x77], 200);
        roundtrip(&base);
    }

    #[test]
    fn roundtrips_every_shape_of_timestamp_column() {
        let positions: Vec<Vec<u8>> = (0..12u8).map(|i| vec![0x30, i]).collect();
        let build = |stamps: &[u64]| -> Vec<Entry> {
            stamps
                .iter()
                .zip(&positions)
                .map(|(&t, p)| entry(p, 1, Hlc::new(t)))
                .collect()
        };

        // Repeated, ascending, descending, and a mix of all three.
        roundtrip(&build(&[500; 12]));
        roundtrip(&build(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]));
        roundtrip(&build(&[12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]));
        roundtrip(&build(&[5, 5, 5, 9, 9, 1, 1, 1, 1, 400, 400, 2]));
        // Zero is an ordinary timestamp: only slot 0 is exempt from the marker, and it is exempt
        // unconditionally.
        roundtrip(&build(&[0, 0, 0, 1, 1, 5, 0, 0, 7, 900, 900, 900]));
        // Far apart in both directions, so the deltas are wide.
        const WIDE: u64 = 1 << 55;
        roundtrip(&build(&[WIDE, 0, WIDE, 0, 1, 2, 3, 4, 5, 6, 7, 8]));
    }

    /// A timestamp column whose delta cannot be expressed. Unreachable from a clock, but the deltas
    /// are `i64` and the error is the honest answer.
    #[test]
    fn rejects_timestamp_deltas_that_do_not_fit() {
        let entries = vec![
            entry(&[0x01], 1, Hlc::new(0)),
            entry(&[0x02], 1, Hlc::new(u64::MAX)),
        ];
        let mut out = Vec::new();
        assert_eq!(
            encode_entries(&entries, &mut out),
            Err(CodecError::TimestampOverflow)
        );
    }

    /// Only a null or minted key can be encoded, and nothing in the index produces anything else.
    #[test]
    fn rejects_keys_that_are_neither_null_nor_minted() {
        let mut out = Vec::new();
        let short = FractionalKey::from([0x01u8, 0x02].as_slice());

        let entries = vec![Entry {
            key: short.clone(),
            modified_at: Hlc::new(1),
            moved: FractionalKey::NULL,
        }];
        assert_eq!(
            encode_entries(&entries, &mut out),
            Err(CodecError::MalformedKey { len: 2 })
        );

        // A null `key` is just as impossible: `moved` is the only field that may be null.
        let entries = vec![Entry {
            key: FractionalKey::NULL,
            modified_at: Hlc::new(1),
            moved: FractionalKey::NULL,
        }];
        assert_eq!(
            encode_entries(&entries, &mut out),
            Err(CodecError::MalformedKey { len: 0 })
        );

        let entries = vec![Entry {
            key: key(&[0x01], 1),
            modified_at: Hlc::new(1),
            moved: short,
        }];
        assert_eq!(
            encode_entries(&entries, &mut out),
            Err(CodecError::MalformedKey { len: 2 })
        );
    }

    #[test]
    fn appends_rather_than_overwriting() {
        // `encode_entries` is called twice against one buffer to hold both spaces of an index.
        let mut out = vec![0xde, 0xad];
        encode_entries(&[entry(&[0x01], 1, Hlc::new(5))], &mut out).unwrap();
        encode_entries(&[entry(&[0x02], 1, Hlc::new(6))], &mut out).unwrap();

        let mut cursor = &out[2..];
        let first = decode_entries(&mut cursor).unwrap();
        let second = decode_entries(&mut cursor).unwrap();
        assert!(cursor.is_empty());
        assert_eq!(first, vec![entry(&[0x01], 1, Hlc::new(5))]);
        assert_eq!(second, vec![entry(&[0x02], 1, Hlc::new(6))]);
    }

    // -----------------------------------------------------------------------------------------
    // Malformed input
    // -----------------------------------------------------------------------------------------

    /// Truncating a valid payload anywhere must be an error, never a panic and never a short read
    /// that looks like success.
    #[test]
    fn rejects_truncation_at_every_offset() {
        // Small stamps deliberately: a corrupted payload is decoded below, and whatever it decodes
        // to is folded into the process clock. Nothing here should be able to reach the far future.
        let mut entries = bulk_run(8, Hlc::new(50));
        entries[3].moved = key(&[0x99], 4);
        entries[5].modified_at = Hlc::new(entries[5].modified_at.get() + 17);
        let encoded = roundtrip(&entries);

        for len in 0..encoded.len() {
            let mut cursor = &encoded[..len];
            assert!(
                decode_entries(&mut cursor).is_err(),
                "a {len}-byte prefix of a {}-byte payload decoded",
                encoded.len()
            );
        }
    }

    /// Every single-bit corruption of a valid payload. A flip may well produce a different valid
    /// index — the claims are that it never panics, and that the stamps it decodes to, which the
    /// decoder folds into the process clock, never name the far future.
    #[test]
    fn survives_every_single_bit_flip() {
        // Small stamps, for the reason given in `rejects_truncation_at_every_offset`.
        let mut entries = bulk_run(6, Hlc::new(50));
        entries[2].moved = key(&[0x99], 4);
        entries[4].modified_at = Hlc::new(entries[4].modified_at.get() - 3);
        let encoded = roundtrip(&entries);
        let ceiling = Hlc::now();

        for i in 0..encoded.len() {
            for bit in 0..8 {
                let mut corrupted = encoded.clone();
                corrupted[i] ^= 1 << bit;
                let mut cursor = corrupted.as_slice();
                if let Ok(decoded) = decode_entries(&mut cursor) {
                    assert!(
                        decoded.iter().all(|e| e.modified_at < ceiling),
                        "byte {i} bit {bit} decoded into a stamp the clock would be stuck with"
                    );
                }
            }
        }
    }

    #[test]
    fn rejects_a_run_that_opens_a_column() {
        // One entry, whose position column opens with a run marker.
        let mut payload = Vec::new();
        write_uvarint(&mut payload, 1);
        write_uvarint(&mut payload, RUN_MARKER);
        write_uvarint(&mut payload, 1); // delta
        write_uvarint(&mut payload, 1); // run
        assert_eq!(
            decode_entries(&mut payload.as_slice()),
            Err(CodecError::RunWithoutPredecessor)
        );
    }

    #[test]
    fn rejects_empty_and_overrunning_runs() {
        // A position literal, then a run claiming zero entries.
        let mut payload = Vec::new();
        write_uvarint(&mut payload, 4);
        write_uvarint(&mut payload, 2);
        payload.push(0x01);
        write_uvarint(&mut payload, RUN_MARKER);
        write_uvarint(&mut payload, 1);
        write_uvarint(&mut payload, 0);
        assert_eq!(
            decode_entries(&mut payload.as_slice()),
            Err(CodecError::EmptyRun)
        );

        // Same, but claiming more entries than the column owes.
        let mut payload = Vec::new();
        write_uvarint(&mut payload, 4);
        write_uvarint(&mut payload, 2);
        payload.push(0x01);
        write_uvarint(&mut payload, RUN_MARKER);
        write_uvarint(&mut payload, 1);
        write_uvarint(&mut payload, 9);
        assert_eq!(
            decode_entries(&mut payload.as_slice()),
            Err(CodecError::RunOverflow {
                run: 9,
                remaining: 3
            })
        );
    }

    /// A run may not carry a position past the width it has to fit in.
    #[test]
    fn rejects_runs_that_overflow_a_position() {
        let mut payload = Vec::new();
        write_uvarint(&mut payload, 2);
        write_uvarint(&mut payload, 2); // literal of one byte
        payload.push(0xff);
        write_uvarint(&mut payload, RUN_MARKER);
        write_uvarint(&mut payload, 1); // 0xff + 1 does not fit in one byte
        write_uvarint(&mut payload, 1);
        assert_eq!(
            decode_entries(&mut payload.as_slice()),
            Err(CodecError::PositionOverflow)
        );
    }

    #[test]
    fn rejects_positions_past_the_key_limit() {
        let mut payload = Vec::new();
        write_uvarint(&mut payload, 1);
        write_uvarint(&mut payload, MAX_POSITION_LEN as u64 + 2);
        assert_eq!(
            decode_entries(&mut payload.as_slice()),
            Err(CodecError::PositionTooLong {
                len: MAX_POSITION_LEN as u64 + 1
            })
        );
    }

    /// Both spaces are binary-searched, so keys that do not ascend are corruption.
    #[test]
    fn rejects_unsorted_and_duplicated_keys() {
        // Hand-build a two-entry payload whose second position sorts below the first.
        let build = |first: u8, second: u8| {
            let mut payload = Vec::new();
            write_uvarint(&mut payload, 2);
            write_uvarint(&mut payload, 2);
            payload.push(first);
            write_uvarint(&mut payload, 2);
            payload.push(second);
            payload.extend_from_slice(&[1, 0, 0, 0]);
            write_uvarint(&mut payload, 2); // one session, both entries
            write_ivarint(&mut payload, 500);
            write_ivarint(&mut payload, REPEAT_MARKER);
            write_uvarint(&mut payload, 1);
            write_uvarint(&mut payload, NULL_MARKER);
            write_uvarint(&mut payload, 2);
            payload
        };
        assert_eq!(
            decode_entries(&mut build(0x05, 0x04).as_slice()),
            Err(CodecError::UnsortedKeys { at: 1 })
        );
        // Equal keys are just as bad: `binary_search` would find one of two indistinguishable slots.
        assert_eq!(
            decode_entries(&mut build(0x05, 0x05).as_slice()),
            Err(CodecError::UnsortedKeys { at: 1 })
        );
        // The same shape with an ascending pair is of course accepted.
        assert!(decode_entries(&mut build(0x04, 0x05).as_slice()).is_ok());
    }

    /// Non-canonical spellings are rejected, which is what makes the round-trip assertion sharp.
    #[test]
    fn rejects_non_canonical_varints() {
        let mut payload = vec![0x81, 0x00]; // `1`, padded to two bytes
        assert_eq!(
            decode_entries(&mut payload.as_slice()),
            Err(CodecError::VarintOverlong)
        );
        payload = vec![0xff; 11];
        assert_eq!(
            decode_entries(&mut payload.as_slice()),
            Err(CodecError::VarintOverflow)
        );
    }

    // -----------------------------------------------------------------------------------------
    // Position arithmetic
    // -----------------------------------------------------------------------------------------

    #[test]
    fn be_delta_pairs_only_positions_it_can_step_between() {
        assert_eq!(be_delta(&[0x00, 0x02], &[0x00, 0x04]), Some(2));
        assert_eq!(be_delta(&[0x00, 0xff], &[0x01, 0x00]), Some(1));
        // Equal positions step by nothing, which is still a run.
        assert_eq!(be_delta(&[0x05], &[0x05]), Some(0));
        // Backwards is not a step: the columns ascend, so this cannot arise from a sorted space.
        assert_eq!(be_delta(&[0x05], &[0x04]), None);
        // Nor is a width change.
        assert_eq!(be_delta(&[0x05], &[0x05, 0x00]), None);
        assert_eq!(be_delta(&[], &[0x00]), None);
        // Two empty positions are trivially one step apart.
        assert_eq!(be_delta(&[], &[]), Some(0));
        // Past eight bytes only the tail may differ, since the step is carried in a `u64`.
        let head = [1u8, 2, 3];
        let mut a = head.to_vec();
        a.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 1]);
        let mut b = head.to_vec();
        b.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 9]);
        assert_eq!(be_delta(&a, &b), Some(8));
        b[0] = 2;
        assert_eq!(be_delta(&a, &b), None);
    }

    /// `be_add` has to be the exact inverse of `be_delta`, or a run does not round-trip.
    #[test]
    fn be_add_inverts_be_delta() {
        let cases: [&[u8]; 6] = [
            &[],
            &[0x00],
            &[0x7f],
            &[0x00, 0x00, 0x02],
            &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
            &[0xaa; 12],
        ];
        for position in cases {
            for delta in [0u64, 1, 2, 17] {
                match be_add(position, delta) {
                    Ok(next) => {
                        assert_eq!(next.len(), position.len(), "be_add widened a position");
                        assert_eq!(be_delta(position, &next), Some(delta));
                    }
                    // Only ever because the step does not fit at this width.
                    Err(e) => assert_eq!(e, CodecError::PositionOverflow),
                }
            }
        }
    }

    #[test]
    fn be_add_refuses_to_widen_a_position() {
        assert_eq!(be_add(&[0xff], 1), Err(CodecError::PositionOverflow));
        assert_eq!(be_add(&[0xff, 0xff], 1), Err(CodecError::PositionOverflow));
        assert_eq!(be_add(&[], 1), Err(CodecError::PositionOverflow));
        assert_eq!(be_add(&[], 0).as_deref(), Ok([].as_slice()));
        assert_eq!(be_add(&[0xfe], 1).as_deref(), Ok([0xff].as_slice()));
        // A full-width position has the whole `u64` to overflow.
        assert_eq!(be_add(&[0xff; 8], 1), Err(CodecError::PositionOverflow));
        // With a head, only the tail is at stake.
        assert_eq!(
            be_add(&[0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff], 1),
            Err(CodecError::PositionOverflow)
        );
    }

    /// The run marker must never make an encoding longer than the literals it replaces.
    #[test]
    fn never_spends_more_than_literals_would() {
        // Single-byte positions stepping by one: a literal is two bytes, a run marker three, so
        // short isolated runs must fall back to literals.
        for count in 1..6usize {
            let entries: Vec<Entry> = (0..count)
                .map(|i| entry(&[0x10 + i as u8], 1, Hlc::new(1)))
                .collect();
            let encoded = roundtrip(&entries);
            let mut naive = Vec::new();
            write_uvarint(&mut naive, count as u64);
            for e in &entries {
                write_position_literal(&mut naive, position_of(&e.key));
            }
            assert!(
                encoded.len() <= naive.len() + 12,
                "{count} entries: {} bytes beats neither literals nor the other columns",
                encoded.len()
            );
        }
    }
}
