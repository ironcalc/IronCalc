#![allow(clippy::unwrap_used)]

//! A range with one absolute end and one relative end, such as `$B$1:B3`, has a
//! size that depends on the cell the formula is in. Typed in H10, `$B$1:B3` is
//! the same three cells as `B1:B3`, and a function that works cell by cell
//! should spill three values for either.

use crate::expressions::parser::static_analysis::{run_static_analysis_on_node, StaticResult};
use crate::expressions::types::CellReferenceRC;
use crate::test::util::new_empty_model;
use crate::Model;

fn column(model: &Model, column: &str, first_row: i32) -> Vec<String> {
    (first_row..first_row + 3)
        .map(|row| model._get_text(&format!("{column}{row}")))
        .collect()
}

fn row(model: &Model, first_column: char, row: i32) -> Vec<String> {
    (0..3)
        .map(|offset| model._get_text(&format!("{}{row}", (first_column as u8 + offset) as char)))
        .collect()
}

fn data() -> Model<'static> {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "a");
    model._set("B3", "3");
    model._set("C1", "-1");
    model._set("C2", "-2");
    model._set("C3", "-3");
    model
}

#[test]
fn a_mixed_range_is_not_known_to_be_one_cell() {
    let mut model = new_empty_model();
    let h10 = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 10,
        column: 8,
    };
    for formula in [
        "$B$1:B3",
        "B$1:B3",
        "$A5:C5",
        "ISNUMBER($B$1:B3)",
        "ABS($C$1:C3)",
        "IF($C$1:C3<-1,1,0)",
    ] {
        let node = model.parser.parse(formula, &h10);
        assert!(
            !matches!(run_static_analysis_on_node(&node), StaticResult::Scalar),
            "{formula} in H10 was taken for a single value"
        );
    }
}

#[test]
fn functions_over_a_mixed_column_spill_like_the_relative_range() {
    let mut model = data();
    // H: one absolute end; I: the same cells written relative
    model._set("H10", "=ISNUMBER($B$1:B3)");
    model._set("I10", "=ISNUMBER(B1:B3)");
    model._set("H20", "=ABS($C$1:C3)");
    model._set("I20", "=ABS(C1:C3)");
    model._set("H30", "=LEN($C$1:C3)");
    model._set("I30", "=LEN(C1:C3)");
    model._set("H40", "=IF($C$1:C3<-1,1,0)");
    model._set("I40", "=IF(C1:C3<-1,1,0)");
    model.evaluate();

    assert_eq!(column(&model, "I", 10), ["TRUE", "FALSE", "TRUE"]);
    assert_eq!(column(&model, "H", 10), column(&model, "I", 10));
    assert_eq!(column(&model, "I", 20), ["1", "2", "3"]);
    assert_eq!(column(&model, "H", 20), column(&model, "I", 20));
    assert_eq!(column(&model, "I", 30), ["2", "2", "2"]);
    assert_eq!(column(&model, "H", 30), column(&model, "I", 30));
    assert_eq!(column(&model, "I", 40), ["0", "1", "1"]);
    assert_eq!(column(&model, "H", 40), column(&model, "I", 40));
}

#[test]
fn formulatext_over_a_mixed_column_spills() {
    let mut model = data();
    model._set("E1", "=1+1");
    model._set("E2", "=2+2");
    model._set("E3", "=3+3");
    model._set("H20", "=FORMULATEXT($E$1:E3)");
    model.evaluate();

    assert_eq!(column(&model, "H", 20), ["=1+1", "=2+2", "=3+3"]);
}

#[test]
fn a_mixed_row_spills_like_the_relative_range() {
    let mut model = new_empty_model();
    model._set("A5", "-1");
    model._set("B5", "-2");
    model._set("C5", "-3");
    // the columns have one absolute end; the rows are relative at both ends
    model._set("J5", "=ABS($A5:C5)");
    model._set("J6", "=ABS(A5:C5)");
    model.evaluate();

    assert_eq!(row(&model, 'J', 6), ["1", "2", "3"]);
    assert_eq!(row(&model, 'J', 5), row(&model, 'J', 6));
}

#[test]
fn the_same_formula_spills_wherever_it_is() {
    // Typed in H1 and H2, `$B$1:B3` is three cells in both; it must not
    // depend on how far the cell is from the absolute end.
    for cell in ["H1", "H2", "H10"] {
        let mut model = data();
        model._set(cell, "=ISNUMBER($B$1:B3)");
        model.evaluate();
        let first_row: i32 = cell[1..].parse().unwrap();
        assert_eq!(
            column(&model, "H", first_row),
            ["TRUE", "FALSE", "TRUE"],
            "in {cell}"
        );
    }
}

#[test]
fn a_one_cell_mixed_range_still_gives_one_value() {
    let mut model = data();
    model._set("H1", "=ABS($C$1:C1)");
    model._set("H2", "=ISNUMBER($B$2:B2)");
    model.evaluate();

    assert_eq!(model._get_text("H1"), "1");
    assert_eq!(model._get_text("H2"), "FALSE");
}
