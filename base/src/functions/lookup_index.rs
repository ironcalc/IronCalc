//! Exact-match lookups over a range of the sheet, answered from an index that
//! is built once per evaluation pass.
//!
//! A sheet with thousands of VLOOKUP/MATCH/XLOOKUP formulas into the same table
//! used to read and compare the whole table once per formula. The first
//! formula of a pass that searches a range now reads it once and records, for
//! every value, the first and last position where it occurs; the others look
//! their value up in that record.
//!
//! The index never changes an answer. It only takes values whose equality is
//! exact: a number whose comparison would use a tolerance (below 2 in absolute
//! value, where two different doubles can be closer than `f64::EPSILON`), a
//! boolean, an empty string or a string with wildcard characters are left to
//! the linear scan. The values of a pass are final once read, and the index is
//! dropped with the pass: a restart, an unwinding or a new evaluation start
//! from nothing. The first formula reads every position of the range, so what
//! the pass records about who read what is the same as before.

use std::collections::HashMap;
use std::sync::Arc;

use crate::calc_result::CalcResult;
use crate::expressions::types::CellReferenceIndex;
use crate::model::Model;
use crate::number_format::to_excel_precision;

/// How a family of functions decides that two values are equal.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum ExactEquality {
    /// VLOOKUP and HLOOKUP: numbers through `compare_values` (15 significant
    /// digits), text through a case-insensitive wildcard regex.
    Vlookup,
    /// MATCH: numbers through `values_are_equal` (raw values), text through a
    /// case-insensitive wildcard regex.
    Match,
    /// XLOOKUP and XMATCH exact match: everything through `compare_values`;
    /// text compares upper-cased.
    Xlookup,
}

/// A range of one row or one column, and the equality it is searched with.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct LookupIndexKey {
    pub(crate) sheet: u32,
    pub(crate) row1: i32,
    pub(crate) column1: i32,
    pub(crate) row2: i32,
    pub(crate) column2: i32,
    pub(crate) equality: ExactEquality,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum ValueKey {
    Number(u64),
    Text(String),
}

/// First and last 0-based position of every value of a range.
#[derive(Debug, Default)]
pub(crate) struct ExactIndex {
    first: HashMap<ValueKey, usize>,
    last: HashMap<ValueKey, usize>,
}

impl ExactIndex {
    pub(crate) fn first(&self, key: &ValueKey) -> Option<usize> {
        self.first.get(key).copied()
    }

    pub(crate) fn last(&self, key: &ValueKey) -> Option<usize> {
        self.last.get(key).copied()
    }
}

fn number_key(value: f64, equality: ExactEquality) -> Option<ValueKey> {
    let value = match equality {
        ExactEquality::Vlookup | ExactEquality::Xlookup => to_excel_precision(value, 15),
        ExactEquality::Match => value,
    };
    if !value.is_finite() || value.abs() < 2.0 {
        return None;
    }
    Some(ValueKey::Number(value.to_bits()))
}

fn text_key(text: &str, equality: ExactEquality) -> ValueKey {
    match equality {
        ExactEquality::Vlookup | ExactEquality::Match => ValueKey::Text(text.to_lowercase()),
        ExactEquality::Xlookup => ValueKey::Text(text.to_uppercase()),
    }
}

/// The key under which `target` would be found, or `None` when only the
/// linear scan answers exactly like before.
pub(crate) fn target_key(target: &CalcResult, equality: ExactEquality) -> Option<ValueKey> {
    match target {
        CalcResult::Number(value) => number_key(*value, equality),
        CalcResult::String(text) => {
            if text.is_empty() {
                return None;
            }
            let wildcards = match equality {
                ExactEquality::Vlookup | ExactEquality::Match => {
                    text.contains(['*', '?', '~'])
                }
                ExactEquality::Xlookup => false,
            };
            if wildcards {
                return None;
            }
            Some(text_key(text, equality))
        }
        _ => None,
    }
}

/// The key a cell value of the range is stored under. Values that no target
/// with a key can be equal to are not stored.
fn value_key(value: &CalcResult, equality: ExactEquality) -> Option<ValueKey> {
    match value {
        CalcResult::Number(number) => number_key(*number, equality),
        CalcResult::String(text) if !text.is_empty() => Some(text_key(text, equality)),
        _ => None,
    }
}

impl<'a> Model<'a> {
    /// The exact-match index of a range of one row or one column, built on the
    /// first call of a pass. `None` outside a pass, or when the pass is being
    /// abandoned or unwinding while the range is read: then the caller scans.
    pub(crate) fn exact_lookup_index(&mut self, key: LookupIndexKey) -> Option<Arc<ExactIndex>> {
        if !self.evaluation.in_pass {
            return None;
        }
        if let Some(index) = self.evaluation.lookup_indexes.get(&key) {
            return Some(index.clone());
        }
        let vertical = key.column1 == key.column2;
        let length = if vertical {
            key.row2 - key.row1 + 1
        } else {
            key.column2 - key.column1 + 1
        };
        let mut index = ExactIndex::default();
        for offset in 0..length.max(0) {
            let position = if vertical {
                CellReferenceIndex {
                    sheet: key.sheet,
                    row: key.row1 + offset,
                    column: key.column1,
                }
            } else {
                CellReferenceIndex {
                    sheet: key.sheet,
                    row: key.row1,
                    column: key.column1 + offset,
                }
            };
            let value = self.evaluate_cell(position);
            if let Some(value_key) = value_key(&value, key.equality) {
                let offset = offset as usize;
                index.first.entry(value_key.clone()).or_insert(offset);
                index.last.insert(value_key, offset);
            }
        }
        if self.evaluation.restart.is_some() || self.evaluation.unwinding {
            return None;
        }
        let index = Arc::new(index);
        self.evaluation.lookup_indexes.insert(key, index.clone());
        Some(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_numbers_and_wildcards_are_left_to_the_scan() {
        assert!(target_key(&CalcResult::Number(1.0), ExactEquality::Vlookup).is_none());
        assert!(target_key(&CalcResult::Number(-1.5), ExactEquality::Match).is_none());
        assert!(target_key(&CalcResult::Number(0.0), ExactEquality::Xlookup).is_none());
        assert!(target_key(&CalcResult::String("a*".to_string()), ExactEquality::Vlookup).is_none());
        assert!(target_key(&CalcResult::String("".to_string()), ExactEquality::Xlookup).is_none());
        assert!(target_key(&CalcResult::Boolean(true), ExactEquality::Match).is_none());
        assert!(target_key(&CalcResult::String("a*".to_string()), ExactEquality::Xlookup).is_some());
    }

    #[test]
    fn vlookup_numbers_use_fifteen_digits() {
        let a = target_key(&CalcResult::Number(2.000000000000001), ExactEquality::Vlookup);
        let b = target_key(&CalcResult::Number(2.0), ExactEquality::Vlookup);
        assert_eq!(a, b);
        let c = target_key(&CalcResult::Number(2.000000000000001), ExactEquality::Match);
        let d = target_key(&CalcResult::Number(2.0), ExactEquality::Match);
        assert_ne!(c, d);
    }

    #[test]
    fn text_case() {
        assert_eq!(
            target_key(&CalcResult::String("Łódź".to_string()), ExactEquality::Vlookup),
            Some(ValueKey::Text("łódź".to_string()))
        );
        assert_eq!(
            target_key(&CalcResult::String("Łódź".to_string()), ExactEquality::Xlookup),
            Some(ValueKey::Text("ŁÓDŹ".to_string()))
        );
    }
}
