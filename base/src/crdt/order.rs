//! Fractional ordering for rows, columns and sheets.
//!
//! Every ordered entity carries a *position*: a non-empty string over a base-62
//! alphabet, compared lexicographically (byte order). Ordering rules:
//!
//! * `Original(k)` has the implicit position `fixed4(k)` (fixed-width, 4 digits)
//!   and therefore needs no storage while untouched.
//! * An insert between two neighbours takes a fresh string strictly between
//!   their positions ([`between`]). Concurrent inserts into the same gap get
//!   distinct strings and are ordered deterministically by the `(pos, id)`
//!   tiebreak.
//! * A move is a last-write-wins overwrite of the position register — identity
//!   is a map key, so moves cannot duplicate an element.
//!
//! [`AxisOrder`] resolves display indices without materializing the grid: the
//! number of implicit originals below any position is computed arithmetically,
//! so all operations are `O(materialized)`, never `O(grid)`.

use std::collections::HashMap;

use super::ids::EntityId;

const ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const BASE: u64 = 62;

fn digit_value(byte: u8) -> u64 {
    match byte {
        b'0'..=b'9' => (byte - b'0') as u64,
        b'A'..=b'Z' => (byte - b'A' + 10) as u64,
        b'a'..=b'z' => (byte - b'a' + 36) as u64,
        _ => {
            debug_assert!(false, "invalid position byte: {byte}");
            0
        }
    }
}

/// Implicit position of `Original(k)`: `k` in base 62, fixed width 4.
/// Covers rows (`k ≤ 1_048_576`) and columns comfortably (62^4 ≈ 14.7M).
pub(crate) fn original_position(k: u32) -> String {
    debug_assert!((k as u64) < BASE * BASE * BASE * BASE);
    let mut v = k as u64;
    let mut out = [b'0'; 4];
    for slot in out.iter_mut().rev() {
        *slot = ALPHABET[(v % BASE) as usize];
        v /= BASE;
    }
    String::from_utf8(out.to_vec()).expect("alphabet is ascii")
}

/// Number of `k ≥ 1` such that `fixed4(k) < pos`, capped at `max`.
fn originals_strictly_below(pos: &str, max: u32) -> u64 {
    let bytes = pos.as_bytes();
    let mut v: u64 = 0;
    for i in 0..4 {
        let d = bytes.get(i).map(|b| digit_value(*b)).unwrap_or(0);
        v = v * BASE + d;
    }
    // For pos longer than 4 digits: fixed4(k) < pos ⟺ k ≤ v (prefix equality
    // makes the shorter string smaller). For pos of length ≤ 4: fixed4(k) < pos
    // ⟺ k < v.
    let count = if bytes.len() > 4 { v } else { v.saturating_sub(1) };
    count.min(max as u64)
}

fn trim_trailing_zeros(s: &str) -> &str {
    s.trim_end_matches('0')
}

/// The midpoint algorithm of fractional indexing (after rocicorp's
/// `fractional-indexing`). `a` must be lexicographically smaller than `b`;
/// `None` for `b` means "+infinity". Inputs must not end in `'0'`
/// (callers strip trailing zeros); the result never ends in `'0'`.
fn midpoint(a: &str, b: Option<&str>) -> String {
    if let Some(b) = b {
        debug_assert!(a < b, "midpoint requires a < b, got {a:?} {b:?}");
        // Strip the longest common prefix, treating `a` as padded with '0'.
        let a_bytes = a.as_bytes();
        let b_bytes = b.as_bytes();
        let mut n = 0;
        while n < b_bytes.len() && a_bytes.get(n).copied().unwrap_or(b'0') == b_bytes[n] {
            n += 1;
        }
        if n > 0 {
            let a_rest = if n < a.len() { &a[n..] } else { "" };
            return format!("{}{}", &b[..n], midpoint(a_rest, Some(&b[n..])));
        }
    }
    let digit_a = a.as_bytes().first().map(|b| digit_value(*b)).unwrap_or(0);
    let digit_b = b
        .map(|b| digit_value(b.as_bytes()[0]))
        .unwrap_or(BASE);
    if digit_b - digit_a > 1 {
        let mid = (digit_a + digit_b).div_ceil(2);
        debug_assert!(digit_a < mid && mid < digit_b);
        return (ALPHABET[mid as usize] as char).to_string();
    }
    // Consecutive digits.
    match b {
        Some(b) if b.len() > 1 => b[..1].to_string(),
        _ => {
            let a_rest = if a.is_empty() { "" } else { &a[1..] };
            format!(
                "{}{}",
                ALPHABET[digit_a as usize] as char,
                midpoint(a_rest, None)
            )
        }
    }
}

/// A position strictly between `a` and `b` (`None` = unbounded on that side).
pub(crate) fn between(a: Option<&str>, b: Option<&str>) -> String {
    let a_trim = a.map(trim_trailing_zeros).unwrap_or("");
    let b_trim = b.map(trim_trailing_zeros);
    debug_assert!(
        b_trim != Some(""),
        "upper bound must not be the minimal position"
    );
    let result = midpoint(a_trim, b_trim);
    if let Some(a) = a {
        debug_assert!(result.as_str() > a, "between: {result:?} !> {a:?}");
    }
    if let Some(b) = b {
        debug_assert!(result.as_str() < b, "between: {result:?} !< {b:?}");
    }
    result
}

/// Encodes `value` as 4 base-62 digits (used for disambiguator suffixes).
fn fixed4_of(value: u64) -> String {
    original_position((value % (BASE * BASE * BASE * BASE)) as u32)
}

/// A position strictly between `a` and `b` that is **globally unique**: the
/// deterministic midpoint is extended with a `(client, counter)` suffix so two
/// replicas inserting concurrently into the same gap can never produce the
/// same string. Without this, tied positions would make the gap between them
/// unsplittable (there is no string strictly between two equal strings).
pub(crate) fn unique_position(
    a: Option<&str>,
    b: Option<&str>,
    client: u64,
    counter: u32,
) -> String {
    let m = between(a, b);
    // Trailing '1' keeps the no-trailing-zero invariant.
    let suffix = format!("{}{}1", fixed4_of(client), fixed4_of(counter as u64));
    let result = match b {
        // If `m` is a proper prefix of `b`, a bare extension could overshoot
        // `b`. Skid under it: match `b`'s zero run after the prefix and then
        // append a '0' before the suffix — that digit is strictly smaller than
        // `b`'s first non-zero digit, so the result stays below `b`.
        Some(b) if b.starts_with(&m) && b.len() > m.len() => {
            let zeros = b[m.len()..].bytes().take_while(|&d| d == b'0').count();
            format!("{}{}{}", m, "0".repeat(zeros + 1), suffix)
        }
        _ => format!("{m}{suffix}"),
    };
    if let Some(a) = a {
        debug_assert!(result.as_str() > a, "unique_position: {result:?} !> {a:?}");
    }
    if let Some(b) = b {
        debug_assert!(result.as_str() < b, "unique_position: {result:?} !< {b:?}");
    }
    result
}

/// Ordering key: position first, id as deterministic tiebreak.
type Key = (String, EntityId);

fn key_ref(key: &Key) -> (&str, EntityId) {
    (key.0.as_str(), key.1)
}

/// The visible ordering of one axis (rows or columns) of one sheet.
///
/// Only *materialized* entities are stored; every `Original(k)` that has never
/// been touched is implicit with position `fixed4(k)`. Display indices are
/// 1-based, truncated at `max` conceptually (queries beyond the grid return
/// `None`).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AxisOrder {
    max: u32,
    /// Visible materialized entries, sorted by `(pos, id)`.
    entries: Vec<Key>,
    /// Every materialized `Original(k)` (visible or not), ascending. These are
    /// excluded from the implicit stream.
    materialized_originals: Vec<u32>,
    /// Position and visibility of every materialized id.
    materialized: HashMap<EntityId, (String, bool)>,
}

impl AxisOrder {
    /// Builds the order from materialized axis entries:
    /// `(id, explicit position if any, visible)`.
    pub(crate) fn new<I>(max: u32, materialized: I) -> AxisOrder
    where
        I: IntoIterator<Item = (EntityId, Option<String>, bool)>,
    {
        let mut order = AxisOrder {
            max,
            entries: Vec::new(),
            materialized_originals: Vec::new(),
            materialized: HashMap::new(),
        };
        for (id, pos, visible) in materialized {
            let pos = pos.unwrap_or_else(|| match id {
                EntityId::Original(k) => original_position(k),
                EntityId::Inserted { .. } => {
                    debug_assert!(false, "inserted id without a position");
                    original_position(0)
                }
            });
            order.add_materialized(id, pos, visible);
        }
        order.entries.sort();
        order.materialized_originals.sort_unstable();
        order
    }

    fn add_materialized(&mut self, id: EntityId, pos: String, visible: bool) {
        if let EntityId::Original(k) = id {
            self.materialized_originals.push(k);
        }
        if visible {
            self.entries.push((pos.clone(), id));
        }
        self.materialized.insert(id, (pos, visible));
    }

    /// Number of implicit (non-materialized) originals strictly before `key`.
    fn implicit_below(&self, key: (&str, EntityId)) -> u64 {
        let (pos, id) = key;
        let mut t = originals_strictly_below(pos, self.max);
        // Tie: an implicit original whose fixed4 equals `pos` sorts by id.
        if pos.len() == 4 {
            let candidate = t + 1;
            if candidate <= self.max as u64
                && original_position(candidate as u32) == pos
                && EntityId::Original(candidate as u32) < id
            {
                t = candidate;
            }
        }
        // Subtract materialized originals in the same range: the qualifying set
        // is exactly {1..=t} because fixed4 is monotone.
        let mats_le_t = self
            .materialized_originals
            .partition_point(|&m| (m as u64) <= t);
        t - mats_le_t as u64
    }

    /// The `(pos, id)` key of an id, whether implicit or materialized.
    /// Returns `None` if the id is materialized and invisible.
    fn key_of(&self, id: EntityId) -> Option<Key> {
        match self.materialized.get(&id) {
            Some((pos, visible)) => visible.then(|| (pos.clone(), id)),
            None => match id {
                EntityId::Original(k) if k >= 1 && k <= self.max => {
                    Some((original_position(k), id))
                }
                _ => None,
            },
        }
    }

    /// Current position string of an id (implicit or materialized), if visible.
    pub(crate) fn position_of(&self, id: EntityId) -> Option<String> {
        self.key_of(id).map(|(pos, _)| pos)
    }

    /// 1-based display index of a visible id.
    pub(crate) fn index_of(&self, id: EntityId) -> Option<u32> {
        let key = self.key_of(id)?;
        let kref = key_ref(&key);
        let implicit = self.implicit_below(kref);
        let entries_below = self.entries.partition_point(|e| key_ref(e) < kref);
        let index = implicit + entries_below as u64 + 1;
        (index <= self.max as u64).then_some(index as u32)
    }

    /// The `r`-th non-materialized original, ascending (1-based rank).
    fn nth_implicit(&self, rank: u64) -> Option<EntityId> {
        let mut k = rank;
        for &m in &self.materialized_originals {
            if (m as u64) <= k {
                k += 1;
            } else {
                break;
            }
        }
        (k >= 1 && k <= self.max as u64).then_some(EntityId::Original(k as u32))
    }

    /// Visible id at 1-based display `index`.
    pub(crate) fn id_at(&self, index: u32) -> Option<EntityId> {
        if index == 0 || index > self.max {
            return None;
        }
        let index = index as u64;
        for (j, entry) in self.entries.iter().enumerate() {
            let merged_pos_of_entry = self.implicit_below(key_ref(entry)) + j as u64 + 1;
            if index < merged_pos_of_entry {
                return self.nth_implicit(index - j as u64);
            }
            if index == merged_pos_of_entry {
                return Some(entry.1);
            }
        }
        self.nth_implicit(index - self.entries.len() as u64)
    }

    /// Position bounds for inserting before display `index` (1-based): the
    /// positions of the current occupants of `index - 1` and `index`.
    ///
    /// Backstop for tied positions (which [`unique_position`] should prevent):
    /// if the two neighbours share a position, scan forward for the next
    /// strictly greater one so a splittable gap is always returned. The new
    /// element then lands after the tied group instead of inside it — a
    /// one-slot placement inaccuracy instead of a failure.
    pub(crate) fn insert_bounds(&self, index: u32) -> (Option<String>, Option<String>) {
        let lower = if index > 1 {
            self.id_at(index - 1).and_then(|id| self.position_of(id))
        } else {
            None
        };
        let mut probe = index;
        let upper = loop {
            let Some(pos) = self.id_at(probe).and_then(|id| self.position_of(id)) else {
                break None;
            };
            match &lower {
                Some(lo) if pos <= *lo => probe += 1,
                _ => break Some(pos),
            }
        };
        (lower, upper)
    }

    /// Registers a newly inserted visible id (used to keep the cached order in
    /// step while translating a batch of local diffs).
    pub(crate) fn insert(&mut self, id: EntityId, pos: String) {
        if let EntityId::Original(k) = id {
            if !self.materialized_originals.contains(&k) {
                let at = self.materialized_originals.partition_point(|&m| m < k);
                self.materialized_originals.insert(at, k);
            }
        }
        self.materialized.insert(id, (pos.clone(), true));
        let key = (pos, id);
        let at = self.entries.partition_point(|e| *e < key);
        self.entries.insert(at, key);
    }

    /// Marks an id invisible (deleted). Originals are materialized as
    /// tombstones so they leave the implicit stream.
    pub(crate) fn remove(&mut self, id: EntityId) {
        let pos = self
            .position_of(id)
            .or_else(|| self.materialized.get(&id).map(|(p, _)| p.clone()));
        let Some(pos) = pos else { return };
        if let EntityId::Original(k) = id {
            if !self.materialized_originals.contains(&k) {
                let at = self.materialized_originals.partition_point(|&m| m < k);
                self.materialized_originals.insert(at, k);
            }
        }
        self.materialized.insert(id, (pos.clone(), false));
        let key = (pos, id);
        if let Ok(at) = self.entries.binary_search(&key) {
            self.entries.remove(at);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ins(client: u64, counter: u32) -> EntityId {
        EntityId::Inserted { client, counter }
    }

    #[test]
    fn original_positions_are_ordered_and_fixed_width() {
        assert_eq!(original_position(1), "0001");
        assert!(original_position(1) < original_position(2));
        assert!(original_position(61) < original_position(62));
        assert!(original_position(1_048_575) < original_position(1_048_576));
    }

    #[test]
    fn between_generates_strictly_ordered_positions() {
        let cases: Vec<(Option<String>, Option<String>)> = vec![
            (None, None),
            (None, Some(original_position(1))),
            (Some(original_position(1)), Some(original_position(2))),
            (Some(original_position(61)), Some(original_position(62))),
            (Some(original_position(1_048_576)), None),
            (Some("0001V".to_string()), Some(original_position(2))),
        ];
        for (a, b) in cases {
            let p = between(a.as_deref(), b.as_deref());
            if let Some(a) = &a {
                assert!(p.as_str() > a.as_str(), "{p:?} !> {a:?}");
            }
            if let Some(b) = &b {
                assert!(p.as_str() < b.as_str(), "{p:?} !< {b:?}");
            }
            assert!(!p.ends_with('0'));
        }
    }

    #[test]
    fn unique_positions_never_collide_in_the_same_gap() {
        // Two clients inserting concurrently into the same gap must produce
        // distinct positions, and the gap between those must stay splittable.
        let lo = original_position(4);
        let hi = original_position(5);
        let p1 = unique_position(Some(&lo), Some(&hi), 1, 10);
        let p2 = unique_position(Some(&lo), Some(&hi), 2, 10);
        assert_ne!(p1, p2);
        assert!(p1.as_str() > lo.as_str() && p1.as_str() < hi.as_str());
        assert!(p2.as_str() > lo.as_str() && p2.as_str() < hi.as_str());
        let (a, b) = if p1 < p2 { (&p1, &p2) } else { (&p2, &p1) };
        let mid = unique_position(Some(a), Some(b), 3, 11);
        assert!(mid.as_str() > a.as_str() && mid.as_str() < b.as_str());
    }

    #[test]
    fn unique_position_survives_prefix_shaped_bounds() {
        // Craft an upper bound that the midpoint is a prefix of, including a
        // zero run right after the prefix.
        let cases = [
            (Some("1"), Some("21")),
            (Some("1"), Some("2001")),
            (None, Some("0001")),
            (Some("0zzz"), Some("1001")),
        ];
        for (a, b) in cases {
            let p = unique_position(a, b, u64::MAX, u32::MAX);
            if let Some(a) = a {
                assert!(p.as_str() > a, "{p:?} !> {a:?}");
            }
            if let Some(b) = b {
                assert!(p.as_str() < b, "{p:?} !< {b:?}");
            }
        }
    }

    #[test]
    fn between_repeated_inserts_stay_bounded() {
        // Insert repeatedly at the same spot; positions must stay ordered.
        let lo = original_position(1);
        let mut hi = original_position(2);
        for _ in 0..100 {
            let p = between(Some(&lo), Some(&hi));
            assert!(p.as_str() > lo.as_str() && p.as_str() < hi.as_str());
            hi = p;
        }
    }

    #[test]
    fn pristine_axis_is_identity() {
        let order = AxisOrder::new(1_048_576, Vec::new());
        assert_eq!(order.id_at(1), Some(EntityId::Original(1)));
        assert_eq!(order.id_at(1_048_576), Some(EntityId::Original(1_048_576)));
        assert_eq!(order.id_at(1_048_577), None);
        assert_eq!(order.index_of(EntityId::Original(7)), Some(7));
    }

    #[test]
    fn insert_shifts_originals_down() {
        let mut order = AxisOrder::new(100, Vec::new());
        // Insert before display row 3.
        let (lo, hi) = order.insert_bounds(3);
        assert_eq!(lo.as_deref(), Some(original_position(2).as_str()));
        assert_eq!(hi.as_deref(), Some(original_position(3).as_str()));
        let id = ins(1, 0);
        order.insert(id, between(lo.as_deref(), hi.as_deref()));

        assert_eq!(order.id_at(2), Some(EntityId::Original(2)));
        assert_eq!(order.id_at(3), Some(id));
        assert_eq!(order.id_at(4), Some(EntityId::Original(3)));
        assert_eq!(order.index_of(EntityId::Original(3)), Some(4));
        assert_eq!(order.index_of(id), Some(3));
    }

    #[test]
    fn delete_shifts_originals_up() {
        let mut order = AxisOrder::new(100, Vec::new());
        order.remove(EntityId::Original(2));
        assert_eq!(order.id_at(1), Some(EntityId::Original(1)));
        assert_eq!(order.id_at(2), Some(EntityId::Original(3)));
        assert_eq!(order.index_of(EntityId::Original(2)), None);
        assert_eq!(order.index_of(EntityId::Original(3)), Some(2));
    }

    #[test]
    fn mixed_insert_and_delete() {
        let mut order = AxisOrder::new(100, Vec::new());
        // Delete rows 2 and 3, then insert a row before (new) display row 2.
        order.remove(EntityId::Original(2));
        order.remove(EntityId::Original(3));
        let (lo, hi) = order.insert_bounds(2);
        let id = ins(9, 1);
        order.insert(id, between(lo.as_deref(), hi.as_deref()));
        assert_eq!(order.id_at(1), Some(EntityId::Original(1)));
        assert_eq!(order.id_at(2), Some(id));
        assert_eq!(order.id_at(3), Some(EntityId::Original(4)));
    }

    #[test]
    fn concurrent_positions_tiebreak_by_id() {
        // Two entries with the same position must order deterministically.
        let a = ins(1, 0);
        let b = ins(2, 0);
        let pos = between(Some(&original_position(1)), Some(&original_position(2)));
        let order = AxisOrder::new(
            100,
            vec![
                (b, Some(pos.clone()), true),
                (a, Some(pos.clone()), true),
            ],
        );
        assert_eq!(order.id_at(2), Some(a));
        assert_eq!(order.id_at(3), Some(b));
        assert_eq!(order.id_at(4), Some(EntityId::Original(2)));
    }

    #[test]
    fn materialized_original_with_default_position() {
        // An original materialized (e.g. it got a height) keeps its slot.
        let order = AxisOrder::new(100, vec![(EntityId::Original(5), None, true)]);
        assert_eq!(order.id_at(5), Some(EntityId::Original(5)));
        assert_eq!(order.id_at(4), Some(EntityId::Original(4)));
        assert_eq!(order.id_at(6), Some(EntityId::Original(6)));
        assert_eq!(order.index_of(EntityId::Original(5)), Some(5));
    }

    #[test]
    fn resurrected_original_regains_slot() {
        let mut order = AxisOrder::new(100, Vec::new());
        order.remove(EntityId::Original(2));
        assert_eq!(order.index_of(EntityId::Original(2)), None);
        order.insert(EntityId::Original(2), original_position(2));
        assert_eq!(order.index_of(EntityId::Original(2)), Some(2));
        assert_eq!(order.id_at(3), Some(EntityId::Original(3)));
    }
}
