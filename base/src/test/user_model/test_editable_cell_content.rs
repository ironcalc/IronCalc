#![allow(clippy::unwrap_used)]

use crate::test::user_model::util::new_empty_user_model;

#[test]
fn dynamic_arrays() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "=SEQUENCE(10)").unwrap();

    assert_eq!(
        model.get_editable_cell_content(0, 1, 1).unwrap(),
        "=SEQUENCE(10)"
    );
    assert_eq!(model.get_editable_cell_content(0, 2, 1).unwrap(), "");
    assert_eq!(model.get_editable_cell_content(0, 3, 1).unwrap(), "");
    assert_eq!(model.get_cell_content(0, 2, 1).unwrap(), "2");
}

#[test]
fn array_formulas() {
    let mut model = new_empty_user_model();
    model
        .set_user_array_formula(0, 1, 1, 2, 2, "={1,2;3,4}")
        .unwrap();
    assert_eq!(
        model.get_editable_cell_content(0, 1, 1).unwrap(),
        "={1,2;3,4}"
    );
    assert_eq!(model.get_editable_cell_content(0, 1, 2).unwrap(), "");
    assert_eq!(model.get_editable_cell_content(0, 2, 1).unwrap(), "");
    assert_eq!(model.get_editable_cell_content(0, 2, 2).unwrap(), "");

    assert_eq!(model.get_cell_content(0, 1, 1).unwrap(), "={1,2;3,4}");
    assert_eq!(model.get_cell_content(0, 1, 2).unwrap(), "2");
    assert_eq!(model.get_cell_content(0, 2, 1).unwrap(), "3");
    assert_eq!(model.get_cell_content(0, 2, 2).unwrap(), "4");
}
