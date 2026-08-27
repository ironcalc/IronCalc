//! Repairing name collisions concurrent edits produce.
//!
//! Names are authored blind — a replica cannot see what a peer is naming at the same moment — so
//! uniqueness is restored afterwards, from the replicated registers and their total stamp order
//! alone. Every replica therefore derives the same names whatever order the commits arrived in.

use std::collections::HashSet;

use crate::collab::log::Timestamp;

/// How one family of names is kept unique: sheets fold case and cap at Excel's 31, named styles
/// keep case and cap at 255.
pub(crate) struct NameRepair {
    pub case_insensitive: bool,
    pub max_len: usize,
}

impl NameRepair {
    /// The form `taken` holds a name under, so lookups honour the family's case rule.
    fn key(&self, name: &str) -> String {
        if self.case_insensitive {
            name.to_uppercase()
        } else {
            name.to_string()
        }
    }

    /// The set of names already spoken for, for callers repairing a single name against live state.
    pub(crate) fn taken<'a>(&self, names: impl IntoIterator<Item = &'a str>) -> HashSet<String> {
        names.into_iter().map(|name| self.key(name)).collect()
    }

    /// Splits a name on a trailing `" (n)"`, so repairing `"Data (1)"` continues that numbering
    /// rather than nesting another suffix. Without one, the whole name is the base and `n` is 1.
    fn split_suffix(name: &str) -> (&str, u32) {
        let Some((base, digits)) = name
            .strip_suffix(')')
            .and_then(|rest| rest.rsplit_once(" ("))
        else {
            return (name, 1);
        };
        match digits.parse::<u32>() {
            Ok(n) => (base, n.saturating_add(1)),
            Err(_) => (name, 1),
        }
    }

    /// The first name `taken` does not already hold: `authored` itself, else the numbered variants
    /// of its base. Records the winner in `taken`.
    pub(crate) fn free_name(&self, authored: String, taken: &mut HashSet<String>) -> String {
        if taken.insert(self.key(&authored)) {
            return authored;
        }
        let (base, mut n) = Self::split_suffix(&authored);
        loop {
            let suffix = format!(" ({n})");
            let room = self.max_len.saturating_sub(suffix.chars().count());
            // Truncate by characters, never bytes: the base can hold multi-byte ones.
            let candidate: String = base.chars().take(room).chain(suffix.chars()).collect();
            if taken.insert(self.key(&candidate)) {
                return candidate;
            }
            n += 1;
        }
    }

    /// Display names for every stamped authored name, first write wins: the earliest `(stamp, id)`
    /// keeps what it authored, later ones give way to a numbered variant.
    pub(crate) fn assign<K: Ord>(&self, authored: Vec<(Timestamp, K, String)>) -> Vec<(K, String)> {
        self.assign_within(authored, &mut HashSet::new())
    }

    /// [`Self::assign`] around names already spoken for by something the stamps do not order — the
    /// built-in style entries, which no register owns.
    pub(crate) fn assign_within<K: Ord>(
        &self,
        mut authored: Vec<(Timestamp, K, String)>,
        taken: &mut HashSet<String>,
    ) -> Vec<(K, String)> {
        authored.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
        authored
            .into_iter()
            .map(|(_, id, name)| (id, self.free_name(name, taken)))
            .collect()
    }
}

/// A 64-bit id for `name`, so two replicas naming the same thing at the same moment write the same
/// register. `salt` walks past a collision; `DefaultHasher` is not stable across Rust releases.
pub(crate) fn stable_id(name: &str, salt: u32) -> u64 {
    // FNV-1a.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in name.bytes().chain(salt.to_le_bytes()) {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
