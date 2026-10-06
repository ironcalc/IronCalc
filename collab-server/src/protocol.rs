//! Update classification, mirroring `base/src/crdt/sync.rs`.
//!
//! The relay must apply the same rule as the clients: never hand yrs an
//! update with a causal gap. yrs 0.27.3 parks gapped map-overwrite items in
//! its pending queue and silently fails to re-integrate them once the gap
//! fills, which would desync the room document. (Kept as a copy so this
//! crate stays free of the engine dependency.)

use yrs::{StateVector, Update};

/// An empty v1 update: zero structs, empty delete set. Never worth relaying.
pub const EMPTY_UPDATE_V1: &[u8] = &[0, 0];

/// How an update relates to a document with state `local`.
pub enum UpdateFit {
    /// Some client's blocks start beyond the locally known clock: applying
    /// would leave a causal gap.
    Gap,
    /// Causally contiguous and carries new blocks and/or deletions.
    Content,
    /// Everything in it is already known.
    Empty,
}

pub fn classify_update(update: &Update, local: &StateVector) -> UpdateFit {
    let mut novel = false;
    let insertions = update.insertions(true);
    for client in insertions.client_ids() {
        let Some(id_range) = insertions.get(&client) else {
            continue;
        };
        let mut ranges: Vec<(u32, u32)> = id_range.iter().map(|r| (r.0.start, r.0.end)).collect();
        ranges.sort_unstable();
        let mut cursor = local.get(&client);
        for (start, end) in ranges {
            if end <= cursor {
                continue;
            }
            if start > cursor {
                return UpdateFit::Gap;
            }
            novel = true;
            cursor = end;
        }
    }
    if novel || !update.delete_set().is_empty() {
        UpdateFit::Content
    } else {
        UpdateFit::Empty
    }
}
