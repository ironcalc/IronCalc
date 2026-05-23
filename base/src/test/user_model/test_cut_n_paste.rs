#![allow(clippy::unwrap_used)]

use crate::test::user_model::util::new_empty_user_model;
use crate::test::util::new_empty_model;
use crate::UserModel;

#[test]
fn cun_n_paste_same_area() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    // B3:D5 with data
    model.set_user_input(0, 3, 2, "A").unwrap();
    model.set_user_input(0, 3, 3, "B").unwrap();
    model.set_user_input(0, 3, 4, "C").unwrap();
    model.set_user_input(0, 4, 2, "D").unwrap();
    model.set_user_input(0, 4, 3, "E").unwrap();
    model.set_user_input(0, 4, 4, "F").unwrap();
    model.set_user_input(0, 5, 2, "G").unwrap();
    model.set_user_input(0, 5, 3, "H").unwrap();
    model.set_user_input(0, 5, 4, "I").unwrap();

    // Cut it and paste it in C4
    model.set_selected_cell(3, 2).unwrap();
    model.set_selected_range(3, 2, 5, 4).unwrap();
    let cp = model.copy_to_clipboard().unwrap();

    // C4
    model.set_selected_cell(4, 3).unwrap();

    let source_range = (3, 2, 5, 4);
    model
        .paste_from_clipboard(0, source_range, &cp.data, true)
        .unwrap();

    // Check data is in C4:E6
    assert_eq!(model.get_formatted_cell_value(0, 4, 3).unwrap(), "A");
    assert_eq!(model.get_formatted_cell_value(0, 4, 4).unwrap(), "B");
    assert_eq!(model.get_formatted_cell_value(0, 4, 5).unwrap(), "C");
    assert_eq!(model.get_formatted_cell_value(0, 5, 3).unwrap(), "D");
    assert_eq!(model.get_formatted_cell_value(0, 5, 4).unwrap(), "E");
    assert_eq!(model.get_formatted_cell_value(0, 5, 5).unwrap(), "F");
    assert_eq!(model.get_formatted_cell_value(0, 6, 3).unwrap(), "G");
    assert_eq!(model.get_formatted_cell_value(0, 6, 4).unwrap(), "H");
    assert_eq!(model.get_formatted_cell_value(0, 6, 5).unwrap(), "I");
}

#[test]
fn cun_n_paste_different_sheet() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    // B3:D5 with data
    model.set_user_input(0, 3, 2, "A").unwrap();
    model.set_user_input(0, 3, 3, "B").unwrap();
    model.set_user_input(0, 3, 4, "C").unwrap();
    model.set_user_input(0, 4, 2, "D").unwrap();
    model.set_user_input(0, 4, 3, "E").unwrap();
    model.set_user_input(0, 4, 4, "F").unwrap();
    model.set_user_input(0, 5, 2, "G").unwrap();
    model.set_user_input(0, 5, 3, "H").unwrap();
    model.set_user_input(0, 5, 4, "I").unwrap();

    // Cut it and paste it in C4
    model.set_selected_cell(3, 2).unwrap();
    model.set_selected_range(3, 2, 5, 4).unwrap();
    let cp = model.copy_to_clipboard().unwrap();

    // New sheet and select it
    model.new_sheet().unwrap();
    model.set_selected_sheet(1).unwrap();

    // C4
    model.set_selected_cell(4, 3).unwrap();

    let source_range = (3, 2, 5, 4);
    model
        .paste_from_clipboard(0, source_range, &cp.data, true)
        .unwrap();

    // Check data is in Sheet2!C4:E6
    assert_eq!(model.get_formatted_cell_value(1, 4, 3).unwrap(), "A");
    assert_eq!(model.get_formatted_cell_value(1, 4, 4).unwrap(), "B");
    assert_eq!(model.get_formatted_cell_value(1, 4, 5).unwrap(), "C");
    assert_eq!(model.get_formatted_cell_value(1, 5, 3).unwrap(), "D");
    assert_eq!(model.get_formatted_cell_value(1, 5, 4).unwrap(), "E");
    assert_eq!(model.get_formatted_cell_value(1, 5, 5).unwrap(), "F");
    assert_eq!(model.get_formatted_cell_value(1, 6, 3).unwrap(), "G");
    assert_eq!(model.get_formatted_cell_value(1, 6, 4).unwrap(), "H");
    assert_eq!(model.get_formatted_cell_value(1, 6, 5).unwrap(), "I");

    // Check original range is empty Sheet1!B3:D5
    assert_eq!(model.get_formatted_cell_value(0, 3, 2).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 3, 3).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 3, 4).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 4, 2).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 4, 3).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 4, 4).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 5, 2).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 5, 3).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 5, 4).unwrap(), "");
}

// Regression test for: cutting a dynamic-array formula and pasting it over
// another dynamic-array formula, then undoing, left stray SpillCells behind
// from the paste.
//
// Steps that reproduce the bug:
//   1. =SEQUENCE(10) in A1  → spills down A1:A10 (values 1..10)
//   2. =SEQUENCE(1,3) in C3 → spills right C3:E3 (values 1, 2, 3)
//   3. Cut C3, paste to A1
//      → A1 gets =SEQUENCE(1,3), spills right to B1=2, C1=3
//      → C3 becomes empty
//   4. Undo the paste
//      → A1 restores =SEQUENCE(10)  ✓
//      → C3 restores =SEQUENCE(1,3) ✓
//      BUG: B1 and C1 still show 2 and 3 — stray SpillCells from the undone paste
#[test]
fn cut_paste_dynamic_array_undo_clears_spill_cells() {
    let mut model = new_empty_user_model();

    // Step 1: =SEQUENCE(10) in A1 — spills down A1:A10
    model.set_user_input(0, 1, 1, "=SEQUENCE(10)").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "1");
    assert_eq!(model.get_formatted_cell_value(0, 2, 1).unwrap(), "2");
    assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "3");

    // Step 2: =SEQUENCE(1,3) in C3 — spills right C3:E3 (1, 2, 3)
    model.set_user_input(0, 3, 3, "=SEQUENCE(1,3)").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 3, 3).unwrap(), "1"); // C3
    assert_eq!(model.get_formatted_cell_value(0, 3, 4).unwrap(), "2"); // D3
    assert_eq!(model.get_formatted_cell_value(0, 3, 5).unwrap(), "3"); // E3

    // Step 3: cut C3 and paste to A1
    model.set_selected_cell(3, 3).unwrap(); // select C3
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(1, 1).unwrap(); // select A1 as paste target
    model
        .paste_from_clipboard(0, (3, 3, 3, 3), &cp.data, true)
        .unwrap();

    // After paste: A1 = =SEQUENCE(1,3), spills right → B1=2, C1=3
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "1"); // A1 anchor
    assert_eq!(model.get_formatted_cell_value(0, 1, 2).unwrap(), "2"); // B1 spill
    assert_eq!(model.get_formatted_cell_value(0, 1, 3).unwrap(), "3"); // C1 spill
                                                                       // C3 was cut — it should be empty
    assert_eq!(model.get_formatted_cell_value(0, 3, 3).unwrap(), "");

    // Step 4: undo the paste
    model.undo().unwrap();

    // A1 must be restored to =SEQUENCE(10), spilling down
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "1"); // A1
    assert_eq!(model.get_formatted_cell_value(0, 2, 1).unwrap(), "2"); // A2
    assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "3"); // A3

    // C3 must be restored to =SEQUENCE(1,3), spilling right
    assert_eq!(model.get_formatted_cell_value(0, 3, 3).unwrap(), "1"); // C3
    assert_eq!(model.get_formatted_cell_value(0, 3, 4).unwrap(), "2"); // D3
    assert_eq!(model.get_formatted_cell_value(0, 3, 5).unwrap(), "3"); // E3

    // The spill cells written by the pasted formula (B1, C1) must be gone
    // after undo — this is the regression that the bug introduced.
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2).unwrap(),
        "",
        "B1 must be empty after undoing the cut-paste (was a stray SpillCell)"
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 3).unwrap(),
        "",
        "C1 must be empty after undoing the cut-paste (was a stray SpillCell)"
    );
}

// Regression test: cutting a dynamic-array formula and pasting it onto one of
// its own spill cells must move the formula to the new anchor, not erase it.
//
// Bug: the is_cut path called range_clear_contents on the original anchor after
// writing the formula to the target.  Because range_clear_contents expands a
// DynamicFormula anchor to its full spill range, it also wiped the just-pasted
// formula, leaving the sheet empty.
//
// Steps:
//   1. =SEQUENCE(10) in A1 — spills A1:A10 (values 1..10)
//   2. Cut A1, paste to A5 (a spill cell of SEQUENCE)
//      → A5 should become the new anchor, spilling A5:A14
//      → A1:A4 should be empty
#[test]
fn cut_paste_array_onto_own_spill_cell_moves_formula() {
    let mut model = new_empty_user_model();

    // Step 1: =SEQUENCE(10) in A1
    model.set_user_input(0, 1, 1, "=SEQUENCE(10)").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "1");
    assert_eq!(model.get_formatted_cell_value(0, 5, 1).unwrap(), "5");
    assert_eq!(model.get_formatted_cell_value(0, 10, 1).unwrap(), "10");

    // Step 2: cut A1, paste to A5
    model.set_selected_cell(1, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(5, 1).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 1, 1), &cp.data, true)
        .unwrap();

    // The formula must have moved to A5 — not disappeared.
    assert_ne!(
        model.get_formatted_cell_value(0, 5, 1).unwrap(),
        "",
        "A5 must not be empty after pasting =SEQUENCE(10) onto it"
    );
    // A5:A14 should spill 1..10
    assert_eq!(model.get_formatted_cell_value(0, 5, 1).unwrap(), "1");
    assert_eq!(model.get_formatted_cell_value(0, 6, 1).unwrap(), "2");
    assert_eq!(model.get_formatted_cell_value(0, 14, 1).unwrap(), "10");
    // A1:A4 must be empty (formula moved away)
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 4, 1).unwrap(), "");
    // A15 must be empty (spill does not exceed 10 rows)
    assert_eq!(model.get_formatted_cell_value(0, 15, 1).unwrap(), "");
}

// Regression test: pasting a value onto a spill cell (causing #SPILL!) and then
// undoing must restore the full spill — not leave A1 stuck in #SPILL!.
//
// Root cause: when the array formula evaluates to #SPILL! it resets r=(1,1) in
// the anchor cell.  On undo, the old SpillCell for the blocking position is
// restored by the diff, but the pre-eval clear loop (bounded by r=(1,1)) never
// reaches it.  The blocking check then treats that SpillCell as an occupied cell,
// keeping the formula in #SPILL! permanently.
//
// Steps:
//   1. =SEQUENCE(6) in A1 — spills A1:A6
//   2. Write "X" in C3; cut C3, paste onto A3 (a spill cell)
//      → A3 now holds "X", blocking SEQUENCE → A1 = #SPILL!
//   3. Undo the paste
//      → A3 must be restored to its spill value (3), A1 must show 1..6 again
#[test]
fn paste_onto_spill_cell_then_undo_restores_spill() {
    let mut model = new_empty_user_model();

    // Step 1: =SEQUENCE(6) in A1
    model.set_user_input(0, 1, 1, "=SEQUENCE(6)").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "1");
    assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "3"); // A3 is a spill cell
    assert_eq!(model.get_formatted_cell_value(0, 6, 1).unwrap(), "6");

    // Step 2: put "X" in C3, cut it, paste onto A3
    model.set_user_input(0, 3, 3, "X").unwrap();
    model.set_selected_cell(3, 3).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(3, 1).unwrap();
    model
        .paste_from_clipboard(0, (3, 3, 3, 3), &cp.data, true)
        .unwrap();
    // A3 now holds "X", which blocks SEQUENCE(6)
    assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "X");
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1).unwrap(),
        "#SPILL!",
        "A1 must be #SPILL! while A3 is blocked"
    );

    // Step 3: undo — A3 must revert to its spill value, A1 must recover
    model.undo().unwrap();

    assert_ne!(
        model.get_formatted_cell_value(0, 1, 1).unwrap(),
        "#SPILL!",
        "A1 must not be #SPILL! after undoing the paste"
    );
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "1");
    assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "3");
    assert_eq!(model.get_formatted_cell_value(0, 6, 1).unwrap(), "6");
    // "X" must be back in C3 (the cut source was restored)
    assert_eq!(model.get_formatted_cell_value(0, 3, 3).unwrap(), "X");
}
