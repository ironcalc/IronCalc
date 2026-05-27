#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use ironcalc::import::load_from_xlsx;

#[test]
fn test_import_randarray_correctly() {
    let mut model = load_from_xlsx("tests/DynamicArrays.xlsx", "en", "UTC", "en").unwrap();
    model.evaluate();
    // B20
    let cell_b20 = model.get_localized_cell_content(0, 20, 2).unwrap();

    // A19 has the formula RANDARRAY(3,3,0,100,TRUE) in the DynamicArrays.xlsx file,
    // which should return a 3x3 array of random integers between 0 and 100.
    assert_ne!(cell_b20, "#ERROR!");
}
