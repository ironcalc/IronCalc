use crate::test::util::new_empty_model;

#[test]
fn test_update_cell_with_text() {
    let mut model = new_empty_model();

    //below are safe inputs
    model.set_user_input(0, 1, 1, "Hello".to_string());

    //Now testing all the possible error scenarios

    //Case1 : Invalid sheet
    let update_result = model.update_cell_with_text(1, 1, 1, "new value").err();
    assert_eq!(update_result.unwrap(), "Invalid sheet index".to_string());

    //Case2 : Invalid Row
    let update_result = model.update_cell_with_text(0, 0, 1, "new value").err();
    assert_eq!(
        update_result.unwrap(),
        "Incorrect row or column".to_string()
    );

    //Case3 : Invalid Column
    let update_result = model
        .update_cell_with_text(0, 1, 1048579, "new value")
        .err();
    assert_eq!(
        update_result.unwrap(),
        "Incorrect row or column".to_string()
    );
}

#[test]
fn test_update_cell_with_number() {
    let mut model = new_empty_model();

    //below are safe inputs
    model.update_cell_with_number(0, 1, 1, 10.0);

    //Now testing all the possible error scenarios

    //Case1 : Invalid sheet
    let update_result = model.update_cell_with_number(1, 1, 1, 20.0).err();
    assert_eq!(update_result.unwrap(), "Invalid sheet index".to_string());

    //Case2 : Invalid Row
    let update_result = model.update_cell_with_number(0, 0, 1, 20.0).err();
    assert_eq!(
        update_result.unwrap(),
        "Incorrect row or column".to_string()
    );

    //Case3 : Invalid Column
    let update_result = model.update_cell_with_number(0, 1, 1048579, 20.0).err();
    assert_eq!(
        update_result.unwrap(),
        "Incorrect row or column".to_string()
    );
}

#[test]
fn test_update_cell_with_bool() {
    let mut model = new_empty_model();

    //below are safe inputs
    model.update_cell_with_bool(0, 1, 1, true);

    //Now testing all the possible error scenarios

    //Case1 : Invalid sheet
    let update_result = model.update_cell_with_bool(1, 1, 1, false).err();
    assert_eq!(update_result.unwrap(), "Invalid sheet index".to_string());

    //Case2 : Invalid Row
    let update_result = model.update_cell_with_bool(0, 0, 1, false).err();
    assert_eq!(
        update_result.unwrap(),
        "Incorrect row or column".to_string()
    );

    //Case3 : Invalid Column
    let update_result = model.update_cell_with_bool(0, 1, 1048579, false).err();
    assert_eq!(
        update_result.unwrap(),
        "Incorrect row or column".to_string()
    );
}

#[test]
fn test_update_cell_with_formula() {
    let mut model = new_empty_model();

    //below are safe inputs
    model.update_cell_with_number(0, 1, 1, 10.0);
    model.update_cell_with_formula(0, 1, 2, "=A1*2".to_string());

    model.evaluate();
    //Now testing all the possible error scenarios

    //Case1 : Invalid sheet
    let update_result = model
        .update_cell_with_formula(1, 1, 2, "=A1*2".to_string())
        .err();
    assert_eq!(update_result.unwrap(), "Invalid sheet index".to_string());

    //Case2 : Invalid Row
    let update_result = model
        .update_cell_with_formula(0, 0, 2, "=A1*2".to_string())
        .err();
    assert_eq!(
        update_result.unwrap(),
        "Incorrect row or column".to_string()
    );

    //Case3 : Invalid Column
    let update_result = model
        .update_cell_with_formula(0, 1, 1048579, "=A1*2".to_string())
        .err();
    assert_eq!(
        update_result.unwrap(),
        "Incorrect row or column".to_string()
    );
}
