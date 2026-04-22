#![allow(clippy::unwrap_used)]
use crate::test::user_model::util::new_empty_user_model;

#[test]
fn undo_redo_dynamic_array() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 2, "10").unwrap();
    model.set_user_input(0, 2, 2, "20").unwrap();
    model.set_user_input(0, 3, 2, "30").unwrap();

    // Dynamic formula in A1 that returns a 5×1 array
    model.set_user_input(0, 1, 1, "=B1:B5").unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1),
        Ok("10".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("20".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 3, 1),
        Ok("30".to_string())
    );

    model.set_user_input(0, 3, 1, "Bong!").unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1),
        Ok("#SPILL!".to_string())
    );

    // Undo the formula entry
    model.undo().unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1),
        Ok("10".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("20".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 3, 1),
        Ok("30".to_string())
    );
}

#[test]
fn undo_redo_dynamic_array_with_cse() {
    let mut model = new_empty_user_model();
    // B1
    model.set_user_input(0, 1, 2, "10").unwrap();
    // B2
    model.set_user_input(0, 2, 2, "20").unwrap();
    // B3
    model.set_user_input(0, 3, 2, "30").unwrap();

    // Dynamic formula in A1 that returns a dynamic array
    model.set_user_input(0, 1, 1, "=B1:B5").unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1),
        Ok("10".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("20".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 3, 1),
        Ok("30".to_string())
    );

    // value in A3 causes SPILL! (CSE range A3:E3, anchor overlaps dynamic spill)
    model.set_user_array_formula(0, 3, 1, 5, 1, "=123").unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1),
        Ok("#SPILL!".to_string())
    );

    // Undo the formula entry
    model.undo().unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1),
        Ok("10".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("20".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 3, 1),
        Ok("30".to_string())
    );
}

#[test]
fn spill_array_dynamic() {
    let mut model = new_empty_user_model();
    // A13
    model.set_user_input(0, 13, 1, "10").unwrap();
    // A14
    model.set_user_input(0, 14, 1, "20").unwrap();
    // A15
    model.set_user_input(0, 15, 1, "30").unwrap();
    // A16
    model.set_user_input(0, 16, 1, "40").unwrap();

    // Dynamic array formula in D13
    model.set_user_input(0, 13, 4, "=A13:A16").unwrap();

    // Array formula is C15:G15 — D15 is in D13's spill range
    model
        .set_user_array_formula(0, 15, 3, 5, 1, "=123")
        .unwrap();

    // The dynamic array in D13 should show #SPILL! because D15 (part of C15:G15) is in the spill range
    assert_eq!(
        model.get_formatted_cell_value(0, 13, 4),
        Ok("#SPILL!".to_string())
    );
}

#[test]
fn array_in_spill_formula() {
    let mut model = new_empty_user_model();
    // A13
    model.set_user_input(0, 13, 1, "10").unwrap();
    // A14
    model.set_user_input(0, 14, 1, "20").unwrap();
    // A15
    model.set_user_input(0, 15, 1, "30").unwrap();
    // A16
    model.set_user_input(0, 16, 1, "40").unwrap();

    model.set_user_input(0, 13, 4, "=A13:A16").unwrap();

    // an array formula with anchor in the spill range of the dynamic array formula
    model
        .set_user_array_formula(0, 15, 4, 1, 5, "=123")
        .unwrap();

    // The dynamic array in D13 should now spill into E13, F13, G13 and show #SPILL! error because of the array formula in D15:D19
    assert_eq!(
        model.get_formatted_cell_value(0, 13, 4),
        Ok("#SPILL!".to_string())
    );
}
