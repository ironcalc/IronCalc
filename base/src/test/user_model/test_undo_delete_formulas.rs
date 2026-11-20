#![allow(clippy::unwrap_used)]

use crate::test::user_model::util::new_empty_user_model;

#[test]
fn undo_delete_rows_restores_formulas_that_read_them() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "1").unwrap();
    model.set_user_input(0, 2, 1, "2").unwrap();
    model.set_user_input(0, 3, 1, "3").unwrap();
    model.set_user_input(0, 4, 1, "=A1+A2").unwrap();
    model.set_user_input(0, 5, 1, "=SUM(A1:A3)").unwrap();
    model.set_user_input(0, 6, 1, "=A4*2").unwrap();

    model.delete_rows(0, 2, 2).unwrap();
    assert_eq!(model.get_cell_content(0, 2, 1).unwrap(), "=A1+#REF!");
    assert_eq!(model.get_cell_content(0, 4, 1).unwrap(), "=A2*2");

    model.undo().unwrap();
    assert_eq!(model.get_cell_content(0, 2, 1).unwrap(), "2");
    assert_eq!(model.get_cell_content(0, 3, 1).unwrap(), "3");
    assert_eq!(model.get_cell_content(0, 4, 1).unwrap(), "=A1+A2");
    assert_eq!(model.get_cell_content(0, 5, 1).unwrap(), "=SUM(A1:A3)");
    assert_eq!(model.get_cell_content(0, 6, 1).unwrap(), "=A4*2");
    assert_eq!(model.get_formatted_cell_value(0, 4, 1).unwrap(), "3");
    assert_eq!(model.get_formatted_cell_value(0, 5, 1).unwrap(), "6");
    assert_eq!(model.get_formatted_cell_value(0, 6, 1).unwrap(), "6");

    model.redo().unwrap();
    assert_eq!(model.get_cell_content(0, 2, 1).unwrap(), "=A1+#REF!");
    assert_eq!(model.get_cell_content(0, 4, 1).unwrap(), "=A2*2");
}

#[test]
fn undo_delete_columns_restores_formulas_that_read_them() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "1").unwrap();
    model.set_user_input(0, 1, 2, "2").unwrap();
    model.set_user_input(0, 1, 3, "3").unwrap();
    model.set_user_input(0, 1, 4, "=A1+B1").unwrap();
    model.set_user_input(0, 1, 5, "=SUM(A1:C1)").unwrap();

    model.delete_columns(0, 2, 2).unwrap();
    assert_eq!(model.get_cell_content(0, 1, 2).unwrap(), "=A1+#REF!");

    model.undo().unwrap();
    assert_eq!(model.get_cell_content(0, 1, 4).unwrap(), "=A1+B1");
    assert_eq!(model.get_cell_content(0, 1, 5).unwrap(), "=SUM(A1:C1)");
    assert_eq!(model.get_formatted_cell_value(0, 1, 5).unwrap(), "6");

    model.redo().unwrap();
    assert_eq!(model.get_cell_content(0, 1, 2).unwrap(), "=A1+#REF!");
}

#[test]
fn undo_delete_rows_restores_formulas_in_other_sheets() {
    let mut model = new_empty_user_model();
    model.new_sheet().unwrap();
    model.set_user_input(0, 2, 1, "5").unwrap();
    model.set_user_input(1, 1, 1, "=Sheet1!A2*2").unwrap();
    model.set_user_input(1, 2, 1, "=SUM(Sheet1!A1:A3)").unwrap();

    model.delete_rows(0, 2, 1).unwrap();
    assert_eq!(model.get_cell_content(1, 1, 1).unwrap(), "=#REF!*2");

    model.undo().unwrap();
    assert_eq!(model.get_cell_content(1, 1, 1).unwrap(), "=Sheet1!A2*2");
    assert_eq!(
        model.get_cell_content(1, 2, 1).unwrap(),
        "=SUM(Sheet1!A1:A3)"
    );
    assert_eq!(model.get_formatted_cell_value(1, 1, 1).unwrap(), "10");
}
