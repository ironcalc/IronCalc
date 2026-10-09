#![allow(clippy::unwrap_used)]

use ironcalc::export::save_to_xlsx;
use ironcalc::import::load_from_xlsx;
use ironcalc_base::Model;

// A row can carry a height, a hidden flag or a style and still hold no cells.
// Such a row is not in the sheet data, so it needs its own <row> entry or the
// attributes are lost on a save/load round trip.
#[test]
fn test_row_attributes_of_a_row_without_cells() {
    let mut model = Model::new_empty("my_model", "en", "UTC", "en").unwrap();

    model.set_row_height(0, 1, 40.0).unwrap();
    model.set_row_height(0, 3, 35.1).unwrap();
    model.set_row_hidden(0, 5, true).unwrap();
    model.set_user_input(0, 6, 1, "hello".to_string()).unwrap();
    model.set_row_height(0, 6, 21.0).unwrap();

    let file_name = "temp_file_row_attributes.xlsx";
    save_to_xlsx(&model, file_name).unwrap();
    let model = load_from_xlsx(file_name, "en", "UTC", "en").unwrap();
    std::fs::remove_file(file_name).unwrap();

    assert_eq!(model.get_row_height(0, 1).unwrap(), 40.0);
    assert_eq!(model.get_row_height(0, 3).unwrap(), 35.1);
    assert!(model.is_row_hidden(0, 5).unwrap());
    assert!(!model.is_row_hidden(0, 1).unwrap());
    // A row with cells keeps working, and its height survives too.
    assert!(!model.is_empty_cell(0, 6, 1).unwrap());
    assert_eq!(model.get_row_height(0, 6).unwrap(), 21.0);
    // Rows that were never touched keep the default height.
    assert_eq!(model.get_row_height(0, 4).unwrap(), 25.0);
}
