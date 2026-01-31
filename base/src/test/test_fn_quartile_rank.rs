use crate::cell::CellValue;
use crate::test::util::new_empty_model;

#[test]
fn test_rank_basic() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("A4", "4");
    model._set("A5", "5");

    // RANK(3, A1:A5) - descending order, 3 is 3rd from top
    model._set("B1", "=RANK(3, A1:A5)");
    model.evaluate();

    assert_eq!(model.get_cell_value_by_ref("Sheet1!B1"), Ok(CellValue::Number(3.0)));

    // RANK(3, A1:A5, 1) - ascending order, 3 is 3rd from bottom
    model._set("B2", "=RANK(3, A1:A5, 1)");
    model.evaluate();

    assert_eq!(model.get_cell_value_by_ref("Sheet1!B2"), Ok(CellValue::Number(3.0)));
}

#[test]
fn test_quartile_basic() {
    let mut model = new_empty_model();
    // Data: 1, 2, 3, 4, 5, 6, 7, 8
    for i in 1..=8 {
        model._set(&format!("A{}", i), &i.to_string());
    }

    // QUARTILE(A1:A8, 0) - minimum
    model._set("B1", "=QUARTILE(A1:A8, 0)");
    // QUARTILE(A1:A8, 2) - median
    model._set("B2", "=QUARTILE(A1:A8, 2)");
    // QUARTILE(A1:A8, 4) - maximum
    model._set("B3", "=QUARTILE(A1:A8, 4)");
    model.evaluate();

    assert_eq!(model.get_cell_value_by_ref("Sheet1!B1"), Ok(CellValue::Number(1.0)));
    assert_eq!(model.get_cell_value_by_ref("Sheet1!B2"), Ok(CellValue::Number(4.5)));
    assert_eq!(model.get_cell_value_by_ref("Sheet1!B3"), Ok(CellValue::Number(8.0)));
}

#[test]
fn test_quartile_inc() {
    let mut model = new_empty_model();
    // Data: 1, 2, 3, 4, 5
    for i in 1..=5 {
        model._set(&format!("A{}", i), &i.to_string());
    }

    // QUARTILE.INC(A1:A5, 1) - Q1
    model._set("B1", "=QUARTILE.INC(A1:A5, 1)");
    // QUARTILE.INC(A1:A5, 3) - Q3
    model._set("B2", "=QUARTILE.INC(A1:A5, 3)");
    model.evaluate();

    assert_eq!(model.get_cell_value_by_ref("Sheet1!B1"), Ok(CellValue::Number(2.0)));
    assert_eq!(model.get_cell_value_by_ref("Sheet1!B2"), Ok(CellValue::Number(4.0)));
}
