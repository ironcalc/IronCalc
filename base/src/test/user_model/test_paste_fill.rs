#![allow(clippy::unwrap_used)]

//! Pasting a copy into a selection that is a whole multiple of it repeats ("fills") the copy
//! across the selection, re-anchoring relative references in every repetition — Excel's behavior.

use crate::cf_types::{CfRuleInput, Cfvo, ColorScaleThreshold};
use crate::expressions::types::Area;
use crate::test::user_model::util::new_empty_user_model;
use crate::types::Color;
use crate::UserModel;

fn color_scale() -> CfRuleInput {
    CfRuleInput::ColorScale {
        thresholds: vec![
            ColorScaleThreshold {
                cfvo: Cfvo::Min,
                color: Color::Rgb("#FF0000".to_string()),
            },
            ColorScaleThreshold {
                cfvo: Cfvo::Max,
                color: Color::Rgb("#00FF00".to_string()),
            },
        ],
    }
}

/// Copies `range` (e.g. `(4, 2, 4, 2)` for B4) to the clipboard and pastes it over the
/// `(first_row, first_column, last_row, last_column)` selection.
fn copy_and_paste_over(
    model: &mut UserModel,
    source: (i32, i32, i32, i32),
    target: (i32, i32, i32, i32),
) {
    let (source_first_row, source_first_column, source_last_row, source_last_column) = source;
    model
        .set_selected_cell(source_first_row, source_first_column)
        .unwrap();
    model
        .set_selected_range(
            source_first_row,
            source_first_column,
            source_last_row,
            source_last_column,
        )
        .unwrap();
    let clipboard = model.copy_to_clipboard().unwrap();

    let (target_first_row, target_first_column, target_last_row, target_last_column) = target;
    model
        .set_selected_cell(target_first_row, target_first_column)
        .unwrap();
    model
        .set_selected_range(
            target_first_row,
            target_first_column,
            target_last_row,
            target_last_column,
        )
        .unwrap();
    model
        .paste_from_clipboard(0, source, &clipboard.data, false)
        .unwrap();
}

#[test]
fn paste_of_a_single_cell_fills_the_selection_readjusting_references_per_cell() {
    let mut model = new_empty_user_model();
    // B3 = 1, B4 = =B3+1. Copy B4 and paste it over B5:B10.
    model.set_user_input(0, 3, 2, "1").unwrap();
    model.set_user_input(0, 4, 2, "=B3+1").unwrap();

    copy_and_paste_over(&mut model, (4, 2, 4, 2), (5, 2, 10, 2));

    // Every filled cell points at the cell above it, not at B4's source displacement.
    for row in 5..=10 {
        assert_eq!(
            model.get_cell_content(0, row, 2).unwrap(),
            format!("=B{}+1", row - 1),
            "B{row} should reference the cell above it"
        );
        assert_eq!(
            model.get_formatted_cell_value(0, row, 2).unwrap(),
            (row - 2).to_string()
        );
    }
}

#[test]
fn paste_fill_keeps_absolute_references_pinned() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "10").unwrap();
    model.set_user_input(0, 3, 2, "=$A$1+A2").unwrap();

    copy_and_paste_over(&mut model, (3, 2, 3, 2), (4, 2, 6, 2));

    assert_eq!(model.get_cell_content(0, 4, 2).unwrap(), "=$A$1+A3");
    assert_eq!(model.get_cell_content(0, 5, 2).unwrap(), "=$A$1+A4");
    assert_eq!(model.get_cell_content(0, 6, 2).unwrap(), "=$A$1+A5");
}

#[test]
fn paste_fill_repeats_a_block_in_both_axes() {
    let mut model = new_empty_user_model();
    // A 1x2 block in A1:B1 (a value and a formula pointing one row up).
    model.set_user_input(0, 1, 1, "7").unwrap();
    model.set_user_input(0, 1, 2, "=A1").unwrap();

    // Fill C3:F4 — two repetitions right, two down.
    copy_and_paste_over(&mut model, (1, 1, 1, 2), (3, 3, 4, 6));

    for (row, column) in [(3, 3), (3, 5), (4, 3), (4, 5)] {
        assert_eq!(model.get_cell_content(0, row, column).unwrap(), "7");
        let reference = format!(
            "={}{}",
            crate::expressions::utils::number_to_column(column).unwrap(),
            row
        );
        assert_eq!(
            model.get_cell_content(0, row, column + 1).unwrap(),
            reference,
            "the formula copy must point at its own repetition's left cell"
        );
    }
}

#[test]
fn paste_fill_repeats_values_and_styles() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "hello").unwrap();
    let a1 = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 1,
    };
    model.update_range_style(&a1, "font.b", "true").unwrap();

    copy_and_paste_over(&mut model, (1, 1, 1, 1), (3, 1, 5, 2));

    for row in 3..=5 {
        for column in 1..=2 {
            assert_eq!(
                model.get_formatted_cell_value(0, row, column).unwrap(),
                "hello"
            );
            assert!(model.get_cell_style(0, row, column).unwrap().font.b);
        }
    }
}

#[test]
fn paste_fill_adds_one_conditional_formatting_rule_spanning_the_filled_block() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "1").unwrap();
    model
        .add_conditional_formatting(0, "A1", color_scale())
        .unwrap();

    // Fill C3:D12 from A1 — 20 repetitions of the single copied cell.
    copy_and_paste_over(&mut model, (1, 1, 1, 1), (3, 3, 12, 4));

    let rules = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(
        rules.len(),
        2,
        "the fill adds ONE rule, not one per filled cell"
    );
    // The list is ordered by priority, highest first — the copied rule is the newest.
    assert_eq!(
        rules[0].range, "C3:D12",
        "the copied rule spans the whole filled block"
    );
    assert_eq!(rules[1].range, "A1", "the source rule is untouched");

    model.undo().unwrap();
    assert_eq!(
        model.get_conditional_formatting_list(0).unwrap().len(),
        1,
        "one undo removes the rule the fill added"
    );
}

#[test]
fn paste_fill_conditional_formatting_over_part_of_the_copy_maps_each_repetition() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "1").unwrap();
    model.set_user_input(0, 2, 1, "2").unwrap();
    // The rule covers only the TOP cell of the 2-row copy, so the repetitions are not contiguous.
    model
        .add_conditional_formatting(0, "A1", color_scale())
        .unwrap();

    // Fill C1:C4 from A1:A2 — two repetitions down.
    copy_and_paste_over(&mut model, (1, 1, 2, 1), (1, 3, 4, 3));

    let rules = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(rules.len(), 2, "still ONE added rule");
    assert_eq!(
        rules[0].range, "C1 C3",
        "its sqref lists the mapped cell of each repetition"
    );
}

#[test]
fn paste_fill_selects_the_whole_filled_area() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "1").unwrap();

    copy_and_paste_over(&mut model, (1, 1, 1, 1), (3, 1, 6, 2));

    assert_eq!(model.get_selected_view().range, [3, 1, 6, 2]);
}

#[test]
fn paste_fill_is_a_single_undo_step() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 3, 2, "1").unwrap();
    model.set_user_input(0, 4, 2, "=B3+1").unwrap();
    model.set_user_input(0, 7, 2, "keep me").unwrap();

    copy_and_paste_over(&mut model, (4, 2, 4, 2), (5, 2, 10, 2));
    assert_eq!(model.get_cell_content(0, 10, 2).unwrap(), "=B9+1");

    model.undo().unwrap();

    assert_eq!(model.get_cell_content(0, 7, 2).unwrap(), "keep me");
    for row in [5, 6, 8, 9, 10] {
        assert_eq!(
            model.get_cell_content(0, row, 2).unwrap(),
            "",
            "one undo must clear every filled cell"
        );
    }

    model.redo().unwrap();
    assert_eq!(model.get_cell_content(0, 5, 2).unwrap(), "=B4+1");
    assert_eq!(model.get_cell_content(0, 10, 2).unwrap(), "=B9+1");
}

#[test]
fn paste_into_a_selection_that_is_not_a_whole_multiple_pastes_once() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "1").unwrap();
    model.set_user_input(0, 2, 1, "2").unwrap();

    // A 2-row copy into a 3-row selection is not a whole multiple: paste it once, at the
    // selection's top-left corner.
    copy_and_paste_over(&mut model, (1, 1, 2, 1), (4, 1, 6, 1));

    assert_eq!(model.get_formatted_cell_value(0, 4, 1).unwrap(), "1");
    assert_eq!(model.get_formatted_cell_value(0, 5, 1).unwrap(), "2");
    assert_eq!(model.get_formatted_cell_value(0, 6, 1).unwrap(), "");
}

#[test]
fn paste_into_a_smaller_selection_pastes_the_whole_copy() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "1").unwrap();
    model.set_user_input(0, 2, 1, "2").unwrap();
    model.set_user_input(0, 3, 1, "3").unwrap();

    // A single selected cell is the classic paste: the copy lands whole, from the anchor.
    copy_and_paste_over(&mut model, (1, 1, 3, 1), (5, 1, 5, 1));

    assert_eq!(model.get_formatted_cell_value(0, 5, 1).unwrap(), "1");
    assert_eq!(model.get_formatted_cell_value(0, 6, 1).unwrap(), "2");
    assert_eq!(model.get_formatted_cell_value(0, 7, 1).unwrap(), "3");
}

#[test]
fn cut_into_a_larger_selection_moves_the_cells_once() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "1").unwrap();

    model.set_selected_cell(1, 1).unwrap();
    let clipboard = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(3, 1).unwrap();
    model.set_selected_range(3, 1, 6, 1).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 1, 1), &clipboard.data, true)
        .unwrap();

    // A cut is a move: exactly one cell moves, the rest of the selection is untouched.
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "1");
    for row in 4..=6 {
        assert_eq!(model.get_formatted_cell_value(0, row, 1).unwrap(), "");
    }
}
