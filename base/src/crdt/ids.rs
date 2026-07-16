//! Stable, compact identifiers for replicated entities (sheets, rows, columns).
//!
//! An id is either:
//! * `Original(k)` — "the k-th row/column/sheet as it existed when the workbook
//!   became collaborative". These are *virtual*: an untouched original needs no
//!   storage at all, and two replicas bootstrapping the same workbook derive the
//!   same ids for the same content, which makes bootstrap convergent by
//!   construction.
//! * `Inserted { client, counter }` — allocated the first time a structural
//!   operation creates a new row/column/sheet. Globally unique, never reused.
//!
//! String encoding (used as yrs map-key fragments):
//! * `Original(k)` → base36 of `k` (e.g. `"7"`, `"2s"`)
//! * `Inserted { client, counter }` → `"~" + base36(client) + "~" + base36(counter)`
//!
//! The id charset (`0-9a-z~`) is disjoint from the key separators used by the
//! doc schema (`!`, `:`, `.`, `/`), so composite keys can be split unambiguously.

/// Maximum number of rows in a sheet (same as the engine grid).
pub const MAX_ROW: u32 = 1_048_576;
/// Maximum number of columns in a sheet (same as the engine grid).
pub const MAX_COLUMN: u32 = 16_384;

/// Stable identity of a row, column or sheet.
///
/// The derived `Ord` (variant order first: all `Original` sort before all
/// `Inserted`) is used as the deterministic tiebreak when two entities end up
/// with the same fractional position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EntityId {
    Original(u32),
    Inserted { client: u64, counter: u32 },
}

const BASE36: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";

fn to_base36(mut value: u64) -> String {
    if value == 0 {
        return "0".to_string();
    }
    let mut digits = Vec::new();
    while value > 0 {
        digits.push(BASE36[(value % 36) as usize]);
        value /= 36;
    }
    digits.reverse();
    String::from_utf8(digits).expect("base36 digits are ascii")
}

fn from_base36(s: &str) -> Option<u64> {
    if s.is_empty() {
        return None;
    }
    let mut value: u64 = 0;
    for b in s.bytes() {
        let digit = match b {
            b'0'..=b'9' => (b - b'0') as u64,
            b'a'..=b'z' => (b - b'a' + 10) as u64,
            _ => return None,
        };
        value = value.checked_mul(36)?.checked_add(digit)?;
    }
    Some(value)
}

impl EntityId {
    pub fn encode(&self) -> String {
        match self {
            EntityId::Original(k) => to_base36(*k as u64),
            EntityId::Inserted { client, counter } => {
                format!("~{}~{}", to_base36(*client), to_base36(*counter as u64))
            }
        }
    }

    pub fn decode(s: &str) -> Option<EntityId> {
        if let Some(rest) = s.strip_prefix('~') {
            let (client, counter) = rest.split_once('~')?;
            Some(EntityId::Inserted {
                client: from_base36(client)?,
                counter: u32::try_from(from_base36(counter)?).ok()?,
            })
        } else {
            Some(EntityId::Original(u32::try_from(from_base36(s)?).ok()?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_round_trip() {
        let cases = [
            EntityId::Original(0),
            EntityId::Original(1),
            EntityId::Original(MAX_ROW),
            EntityId::Inserted {
                client: 0,
                counter: 0,
            },
            EntityId::Inserted {
                client: u64::MAX,
                counter: u32::MAX,
            },
            EntityId::Inserted {
                client: 12345,
                counter: 42,
            },
        ];
        for id in cases {
            assert_eq!(EntityId::decode(&id.encode()), Some(id), "{id:?}");
        }
    }

    #[test]
    fn originals_sort_before_inserted() {
        assert!(
            EntityId::Original(u32::MAX)
                < EntityId::Inserted {
                    client: 0,
                    counter: 0
                }
        );
    }

    #[test]
    fn decode_rejects_garbage() {
        assert_eq!(EntityId::decode(""), None);
        assert_eq!(EntityId::decode("~"), None);
        assert_eq!(EntityId::decode("A1"), None);
        assert_eq!(EntityId::decode("~zz"), None);
    }
}
