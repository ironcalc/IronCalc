#![allow(clippy::unwrap_used)]

//! The `.icalc` wire format is a stored format: a file written by an older build must keep loading.
//!
//! `tests/format_stability.icalc` was produced by the build that predates stable (fractional-key)
//! addressing (the build right before it, after `RangeRef`, upstream's worksheet `links` and
//! upstream's `MergedCell` merged cells landed) and is never regenerated from here — regenerating
//! it would be exactly the regression this guards against.

use ironcalc::import::load_from_icalc;
use ironcalc_base::types::RangeRef;

#[test]
fn icalc_format_stability() {
    let model = load_from_icalc("tests/format_stability.icalc", "en").unwrap();

    let sheets = model.workbook.get_worksheet_names();
    assert_eq!(sheets, vec!["Sheet1".to_string(), "Second".to_string()]);

    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "42");
    assert_eq!(model.get_formatted_cell_value(0, 1, 2).unwrap(), "hello");
    // The stored result of `=A1*2`, not a re-evaluation.
    assert_eq!(model.get_formatted_cell_value(0, 2, 1).unwrap(), "84");
    assert_eq!(model.get_formatted_cell_value(1, 1, 1).unwrap(), "7");

    let merges = &model.workbook.worksheets[0].merged_cells;
    assert_eq!(merges.len(), 1);
    assert_eq!(RangeRef::from(&merges[0]).to_a1(), "C3:D4");

    let rules = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].range, "A1:B5");
}
