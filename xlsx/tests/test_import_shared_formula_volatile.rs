#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use ironcalc::import::load_from_xlsx;

// A daughter cell of a volatile shared formula is an empty f element with ca="1":
//   <c r="C9"><f t="shared" ref="C9:D9" ca="1" si="1">SUM(C3:C8)</f><v>21</v></c>
//   <c r="D9"><f t="shared" ca="1" si="1"/><v>210</v></c>
// It must not be mistaken for a volatile spill placeholder (<f ca="1"/>), which
// would silently drop the formula.
#[test]
fn test_import_volatile_shared_formula_daughter_cell() {
    let mut model =
        load_from_xlsx("tests/shared_formula_volatile.xlsx", "en", "UTC", "en").unwrap();
    model.evaluate();

    assert_eq!(
        model.get_cell_formula(0, 9, 3).unwrap(),
        Some("=SUM(C3:C8)".to_string())
    );
    assert_eq!(
        model.get_cell_formula(0, 9, 4).unwrap(),
        Some("=SUM(D3:D8)".to_string())
    );
    assert_eq!(model.get_formatted_cell_value(0, 9, 4).unwrap(), "210");
}
