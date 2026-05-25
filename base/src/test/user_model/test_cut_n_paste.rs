#![allow(clippy::unwrap_used)]

use crate::expressions::types::Area;
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

// Regression test for: cutting a non anchor part of a dynamic-array formula
//
// Steps that reproduce the bug:
//   1. =SEQUENCE(10) in A1  → spills down A1:A10 (values 1..10)
//   2. Copy A3:A5
//   3. Paste to C3
//      → C3:C5 should not show values
#[test]
fn copy_non_anchor_part_of_array() {
    let mut model = new_empty_user_model();

    // Step 1: =SEQUENCE(10) in A1 — spills down A1:A10
    model.set_user_input(0, 1, 1, "=SEQUENCE(10)").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "1");
    assert_eq!(model.get_formatted_cell_value(0, 2, 1).unwrap(), "2");
    assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "3");

    // Apply a bold style to A3:A5 before copying
    let a3_a5 = Area {
        sheet: 0,
        row: 3,
        column: 1,
        width: 1,
        height: 3,
    };
    model.update_range_style(&a3_a5, "font.b", "true").unwrap();
    assert!(model.get_cell_style(0, 3, 1).unwrap().font.b);
    assert!(model.get_cell_style(0, 4, 1).unwrap().font.b);
    assert!(model.get_cell_style(0, 5, 1).unwrap().font.b);

    // Step 2: copy A3:A5
    model.set_selected_cell(3, 1).unwrap();
    model.set_selected_range(3, 1, 5, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();

    // Step 3: paste to C3
    model.set_selected_cell(3, 3).unwrap();
    model
        .paste_from_clipboard(0, (3, 1, 5, 1), &cp.data, true)
        .unwrap();

    // C3:C5 should be empty — only the anchor cell of a dynamic array formula is copied
    assert_eq!(model.get_formatted_cell_value(0, 3, 3).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 4, 3).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 5, 3).unwrap(), "");

    // The style (bold) should have been copied to C3:C5
    assert!(
        model.get_cell_style(0, 3, 3).unwrap().font.b,
        "C3 should be bold"
    );
    assert!(
        model.get_cell_style(0, 4, 3).unwrap().font.b,
        "C4 should be bold"
    );
    assert!(
        model.get_cell_style(0, 5, 3).unwrap().font.b,
        "C5 should be bold"
    );
}

// When the same non-anchor selection is pasted as CSV the spill values should
// land as plain numbers in the target cells.
#[test]
fn copy_non_anchor_part_of_array_paste_as_csv() {
    let mut model = new_empty_user_model();

    // =SEQUENCE(10) in A1 — spills down A1:A10 (values 1..10)
    model.set_user_input(0, 1, 1, "=SEQUENCE(10)").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "3");
    assert_eq!(model.get_formatted_cell_value(0, 4, 1).unwrap(), "4");
    assert_eq!(model.get_formatted_cell_value(0, 5, 1).unwrap(), "5");

    // Copy A3:A5
    model.set_selected_cell(3, 1).unwrap();
    model.set_selected_range(3, 1, 5, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    // copied as CSV should be the rendered values, not the formula
    assert_eq!(cp.csv, "3\n4\n5");
}

// Cutting a non-anchor part of a dynamic-array formula:
//   - values must NOT appear at the destination (spill cells carry no formula)
//   - styles must be moved to the destination
//   - source cells must lose their style (it was cut, not copied)
//   - source cells still show spill values because the anchor formula is intact
#[test]
fn cut_non_anchor_part_of_array() {
    let mut model = new_empty_user_model();

    // =SEQUENCE(10) in A1 — spills down A1:A10
    model.set_user_input(0, 1, 1, "=SEQUENCE(10)").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "3");
    assert_eq!(model.get_formatted_cell_value(0, 4, 1).unwrap(), "4");
    assert_eq!(model.get_formatted_cell_value(0, 5, 1).unwrap(), "5");

    // Apply bold to A3:A5
    let a3_a5 = Area {
        sheet: 0,
        row: 3,
        column: 1,
        width: 1,
        height: 3,
    };
    model.update_range_style(&a3_a5, "font.b", "true").unwrap();
    assert!(model.get_cell_style(0, 3, 1).unwrap().font.b);
    assert!(model.get_cell_style(0, 4, 1).unwrap().font.b);
    assert!(model.get_cell_style(0, 5, 1).unwrap().font.b);

    // Cut A3:A5 and paste to C3
    model.set_selected_cell(3, 1).unwrap();
    model.set_selected_range(3, 1, 5, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(3, 3).unwrap();
    model
        .paste_from_clipboard(0, (3, 1, 5, 1), &cp.data, true)
        .unwrap();

    // Destination C3:C5: no values (spill cells carry no formula)
    assert_eq!(model.get_formatted_cell_value(0, 3, 3).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 4, 3).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 5, 3).unwrap(), "");

    // Destination C3:C5: style was moved here
    assert!(
        model.get_cell_style(0, 3, 3).unwrap().font.b,
        "C3 should be bold"
    );
    assert!(
        model.get_cell_style(0, 4, 3).unwrap().font.b,
        "C4 should be bold"
    );
    assert!(
        model.get_cell_style(0, 5, 3).unwrap().font.b,
        "C5 should be bold"
    );

    // Source A3:A5: spill values are still there (anchor formula untouched)
    assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "3");
    assert_eq!(model.get_formatted_cell_value(0, 4, 1).unwrap(), "4");
    assert_eq!(model.get_formatted_cell_value(0, 5, 1).unwrap(), "5");
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

// Regression test: cutting a dynamic array formula and pasting it onto a cell
// that overlaps the original spill range, then undoing, must restore the formula
// at its original location — not leave the sheet empty.
//
// Root cause: during undo, the diff is replayed in reverse: the source formula
// (A3) is first restored via RangeClearContents, then the paste target (A1) is
// cleared via range_clear_all.  Because A1 is still a DynamicFormula with
// r=(1,10), range_clear_all sweeps A1:A10 — wiping the just-restored A3.
//
// The reproduction requires prior undo history so the undo stack is non-empty
// before the paste operation is recorded.
//
// Steps:
//   1. Write "1" in B1 then delete it (primes the undo history)
//   2. =SEQUENCE(10) in A3 — spills A3:A12
//   3. Cut A3, paste to A1 (overlaps spill: A1 is inside A3:A12)
//      → A1:A10 show 1..10; A3 moved to new anchor
//   4. Undo — A3 must be the anchor again, spilling A3:A12; A1 must be empty
#[test]
fn cut_paste_overlap_spill_undo_restores_formula() {
    let mut model = new_empty_user_model();

    // Step 1: prime the undo history
    model.set_user_input(0, 1, 2, "1").unwrap(); // B1 = "1"
    model
        .range_clear_contents(&crate::expressions::types::Area {
            sheet: 0,
            row: 1,
            column: 2,
            width: 1,
            height: 1,
        })
        .unwrap(); // delete B1

    // Step 2: =SEQUENCE(10) in A3 — spills A3:A12
    model.set_user_input(0, 3, 1, "=SEQUENCE(10)").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "1");
    assert_eq!(model.get_formatted_cell_value(0, 12, 1).unwrap(), "10");
    assert_eq!(model.get_formatted_cell_value(0, 13, 1).unwrap(), ""); // A13 empty

    // Step 3: cut A3, paste to A1 (A1 is inside the A3:A12 spill range)
    model.set_selected_cell(3, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(1, 1).unwrap();
    model
        .paste_from_clipboard(0, (3, 1, 3, 1), &cp.data, true)
        .unwrap();
    // Formula moved to A1, spills A1:A10
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "1");
    assert_eq!(model.get_formatted_cell_value(0, 10, 1).unwrap(), "10");
    assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "3"); // spill cell
    assert_eq!(model.get_formatted_cell_value(0, 11, 1).unwrap(), ""); // A11 empty

    // Step 4: undo — formula must return to A3
    model.undo().unwrap();

    assert_ne!(
        model.get_formatted_cell_value(0, 3, 1).unwrap(),
        "",
        "A3 must not be empty after undoing the cut-paste"
    );
    assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "1"); // A3 anchor = 1
    assert_eq!(model.get_formatted_cell_value(0, 12, 1).unwrap(), "10"); // A12 = last spill
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), ""); // A1 empty again
    assert_eq!(model.get_formatted_cell_value(0, 2, 1).unwrap(), ""); // A2 empty
    assert_eq!(model.get_formatted_cell_value(0, 13, 1).unwrap(), ""); // A13 empty
}
