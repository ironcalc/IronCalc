#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Replacing a dynamic-array formula through the direct update_cell_with_*
// calls must clear its old spill, as set_user_input already does.

fn spilling_model() -> crate::Model<'static> {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(3)");
    model.evaluate();
    assert_eq!(model._get_text("A3"), "3");
    model
}

#[test]
fn number_over_spilling_formula_clears_spill() {
    let mut model = spilling_model();
    model.update_cell_with_number(0, 1, 1, 9.0).unwrap();
    model.evaluate();
    assert_eq!(model._get_text("A1"), "9");
    assert_eq!(model._get_text("A2"), "");
    assert_eq!(model._get_text("A3"), "");
}

#[test]
fn text_and_bool_over_spilling_formula_clear_spill() {
    let mut model = spilling_model();
    model.update_cell_with_text(0, 1, 1, "hello").unwrap();
    model.evaluate();
    assert_eq!(model._get_text("A3"), "");

    let mut model = spilling_model();
    model.update_cell_with_bool(0, 1, 1, true).unwrap();
    model.evaluate();
    assert_eq!(model._get_text("A3"), "");
}

#[test]
fn shorter_formula_clears_the_rest_of_the_spill() {
    let mut model = spilling_model();
    model
        .update_cell_with_formula(0, 1, 1, "=SEQUENCE(2)".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(model._get_text("A2"), "2");
    assert_eq!(model._get_text("A3"), "");
}

#[test]
fn writing_inside_a_spill_blocks_it_and_it_comes_back() {
    let mut model = spilling_model();
    model.update_cell_with_text(0, 2, 1, "mine").unwrap();
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#SPILL!");
    assert_eq!(model._get_text("A2"), "mine");
}
