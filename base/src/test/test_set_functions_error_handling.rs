use crate::test::util::new_empty_model;

#[test]
fn test_update_cell_with_text() {
    let mut model = new_empty_model();

    // Below are safe inputs
    model.set_user_input(0, 1, 1, "Hello".to_string()).unwrap();

    // Now testing all the possible error scenarios

    // Case1 : Invalid sheet
    let update_result = model.update_cell_with_text(1, 1, 1, "new value");
    assert_eq!(update_result, Err("Invalid sheet index".to_string()));

    // Case2 : Invalid Row
    let update_result = model.update_cell_with_text(0, 0, 1, "new value");
    assert_eq!(update_result, Err("Incorrect row or column".to_string()));

    // Case3 : Invalid Column
    let update_result = model.update_cell_with_text(0, 1, 1048579, "new value");
    assert_eq!(update_result, Err("Incorrect row or column".to_string()));
}

#[test]
fn test_update_cell_with_number() {
    let mut model = new_empty_model();

    // Below are safe inputs
    model.update_cell_with_number(0, 1, 1, 10.0).unwrap();

    // Now testing all the possible error scenarios

    // Case1 : Invalid sheet
    let update_result = model.update_cell_with_number(1, 1, 1, 20.0);
    assert_eq!(update_result, Err("Invalid sheet index".to_string()));

    // Case2 : Invalid Row
    let update_result = model.update_cell_with_number(0, 0, 1, 20.0);
    assert_eq!(update_result, Err("Incorrect row or column".to_string()));

    // Case3 : Invalid Column
    let update_result = model.update_cell_with_number(0, 1, 1048579, 20.0);
    assert_eq!(update_result, Err("Incorrect row or column".to_string()));
}

#[test]
fn test_update_cell_with_bool() {
    let mut model = new_empty_model();

    // Below are safe inputs
    model.update_cell_with_bool(0, 1, 1, true).unwrap();

    // Now testing all the possible error scenarios

    // Case1 : Invalid sheet
    let update_result = model.update_cell_with_bool(1, 1, 1, false);
    assert_eq!(update_result, Err("Invalid sheet index".to_string()));

    // Case2 : Invalid Row
    let update_result = model.update_cell_with_bool(0, 0, 1, false);
    assert_eq!(update_result, Err("Incorrect row or column".to_string()));

    // Case3 : Invalid Column
    let update_result = model.update_cell_with_bool(0, 1, 1048579, false);
    assert_eq!(update_result, Err("Incorrect row or column".to_string()));
}

#[test]
fn test_update_cell_with_formula() {
    let mut model = new_empty_model();

    // Below are safe inputs
    model.update_cell_with_number(0, 1, 1, 10.0).unwrap();
    model
        .update_cell_with_formula(0, 1, 2, "=A1*2".to_string())
        .unwrap();

    model.evaluate();
    // Now testing all the possible error scenarios

    // Case1 : Invalid sheet
    let update_result = model.update_cell_with_formula(1, 1, 2, "=A1*2".to_string());
    assert_eq!(update_result, Err("Invalid sheet index".to_string()));

    // Case2 : Invalid Row
    let update_result = model.update_cell_with_formula(0, 0, 2, "=A1*2".to_string());
    assert_eq!(update_result, Err("Incorrect row or column".to_string()));

    // Case3 : Invalid Column
    let update_result = model.update_cell_with_formula(0, 1, 1048579, "=A1*2".to_string());
    assert_eq!(update_result, Err("Incorrect row or column".to_string()));
}

#[test]
fn test_set_user_input() {
    let mut model = new_empty_model();

    // Below are safe inputs
    model.update_cell_with_number(0, 1, 1, 10.0).unwrap();
    model.evaluate();
    // Now testing all the possible error scenarios

    // Case1 : Invalid sheet
    let update_result = model.set_user_input(1, 1, 2, "20.0".to_string());
    assert_eq!(update_result, Err("Invalid sheet index".to_string()));

    // Case2 : Invalid Row
    let update_result = model.set_user_input(0, 0, 2, "20.0".to_string());
    assert_eq!(update_result, Err("Incorrect row or column".to_string()));

    // Case3 : Invalid Column
    let update_result = model.set_user_input(0, 1, 1048579, "20.0".to_string());
    assert_eq!(update_result, Err("Incorrect row or column".to_string()));
}

#[test]
fn workbook_styles_get_style_error_handling() {
    let model = new_empty_model();

    // case 1 : Invalid index
    assert_eq!(
        model.workbook.styles.get_style(15),
        Err("Invalid index provided".to_string())
    );
}
