#![allow(clippy::unwrap_used)]

use bitcode::decode;

use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::types::Area;
use crate::test::user_model::util::new_empty_user_model;
use crate::types::{Link, MergedCell};
use crate::user_model::history::{Diff, QueueDiffs};
use crate::UserModel;

fn area(sheet: u32, row: i32, column: i32, width: i32, height: i32) -> Area {
    Area {
        sheet,
        row,
        column,
        width,
        height,
    }
}

fn merged(row: i32, column: i32, width: i32, height: i32) -> MergedCell {
    MergedCell {
        row,
        column,
        width,
        height,
    }
}

fn last_diff_list(model: &mut UserModel) -> Vec<Diff> {
    let bytes = model.flush_send_queue();
    let queue: Vec<QueueDiffs> = decode(&bytes).unwrap();
    queue.last().unwrap().list.clone()
}

#[test]
fn merge_undo_redo() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 2, 2, "5").unwrap();
    model.set_user_input(0, 3, 3, "hello").unwrap();

    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();
    assert_eq!(model.get_merged_cells(0).unwrap(), vec![merged(2, 2, 2, 2)]);
    assert_eq!(model.get_formatted_cell_value(0, 2, 2), Ok("5".to_string()));
    assert_eq!(model.get_formatted_cell_value(0, 3, 3), Ok("".to_string()));

    model.undo().unwrap();
    assert!(model.get_merged_cells(0).unwrap().is_empty());
    assert_eq!(
        model.get_formatted_cell_value(0, 3, 3),
        Ok("hello".to_string())
    );

    model.redo().unwrap();
    assert_eq!(model.get_merged_cells(0).unwrap(), vec![merged(2, 2, 2, 2)]);
    assert_eq!(model.get_formatted_cell_value(0, 3, 3), Ok("".to_string()));

    // undo again after the redo
    model.undo().unwrap();
    assert!(model.get_merged_cells(0).unwrap().is_empty());
    assert_eq!(
        model.get_formatted_cell_value(0, 3, 3),
        Ok("hello".to_string())
    );
}

#[test]
fn unmerge_undo_redo() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 2, 2, "5").unwrap();
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();

    model.unmerge_cells(&area(0, 2, 2, 2, 2)).unwrap();
    assert!(model.get_merged_cells(0).unwrap().is_empty());

    model.undo().unwrap();
    assert_eq!(model.get_merged_cells(0).unwrap(), vec![merged(2, 2, 2, 2)]);

    model.redo().unwrap();
    assert!(model.get_merged_cells(0).unwrap().is_empty());
    // the anchor content survives merge + unmerge
    assert_eq!(model.get_formatted_cell_value(0, 2, 2), Ok("5".to_string()));
}

#[test]
fn merge_diff_shape() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 3, 3, "42").unwrap();
    model.flush_send_queue();

    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();
    let list = last_diff_list(&mut model);
    // one SetCellValue for the cleared covered cell C3 plus the SetMergedCells
    assert_eq!(list.len(), 2);
    assert!(matches!(
        &list[0],
        Diff::SetCellValue {
            sheet: 0,
            row: 3,
            column: 3,
            ..
        }
    ));
    assert!(
        matches!(&list[1], Diff::SetMergedCells { sheet: 0, old_value, new_value }
            if old_value.is_empty() && new_value == &vec![merged(2, 2, 2, 2)])
    );
}

#[test]
fn no_op_unmerge_does_not_pollute_history() {
    let mut model = new_empty_user_model();
    model.unmerge_cells(&area(0, 2, 2, 2, 2)).unwrap();
    assert!(!model.can_undo());
}

#[test]
fn failed_merge_does_not_pollute_history() {
    let mut model = new_empty_user_model();
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();
    model.flush_send_queue();
    // overlapping merge fails and must leave no trace
    assert!(model.merge_cells(&area(0, 3, 3, 2, 2)).is_err());
    let queue: Vec<QueueDiffs> = decode(&model.flush_send_queue()).unwrap();
    assert!(queue.is_empty());
}

#[test]
fn external_diffs_replay() {
    let mut model1 = new_empty_user_model();
    let mut model2 = new_empty_user_model();

    model1.set_user_input(0, 2, 2, "5").unwrap();
    model1.set_user_input(0, 3, 3, "hello").unwrap();
    model2
        .apply_external_diffs(&model1.flush_send_queue())
        .unwrap();

    model1.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();
    model2
        .apply_external_diffs(&model1.flush_send_queue())
        .unwrap();

    assert_eq!(
        model2.get_merged_cells(0).unwrap(),
        vec![merged(2, 2, 2, 2)]
    );
    assert_eq!(model2.get_formatted_cell_value(0, 3, 3), Ok("".to_string()));

    // the undo also replays
    model1.undo().unwrap();
    model2
        .apply_external_diffs(&model1.flush_send_queue())
        .unwrap();
    assert!(model2.get_merged_cells(0).unwrap().is_empty());
    assert_eq!(
        model2.get_formatted_cell_value(0, 3, 3),
        Ok("hello".to_string())
    );
}

#[test]
fn merge_clears_covered_links_and_undo_restores_them() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 3, 3, "ironcalc").unwrap();
    let link = Link::External {
        target: "https://www.ironcalc.com".to_string(),
        tooltip: None,
    };
    model.set_cell_link(0, 3, 3, link.clone(), None).unwrap();

    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();
    assert_eq!(model.get_cell_link(0, 3, 3), Ok(None));

    model.undo().unwrap();
    assert_eq!(model.get_cell_link(0, 3, 3), Ok(Some(link)));
}

#[test]
fn delete_rows_wiping_a_merge_is_undoable() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 2, 2, "5").unwrap();
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();

    // rows 1-5 contain the whole merge: it is gone
    model.delete_rows(0, 1, 5).unwrap();
    assert!(model.get_merged_cells(0).unwrap().is_empty());

    model.undo().unwrap();
    assert_eq!(model.get_merged_cells(0).unwrap(), vec![merged(2, 2, 2, 2)]);
    assert_eq!(model.get_formatted_cell_value(0, 2, 2), Ok("5".to_string()));

    model.redo().unwrap();
    assert!(model.get_merged_cells(0).unwrap().is_empty());
}

#[test]
fn delete_rows_above_a_merge_is_undoable() {
    // Deleting rows that overlap the top of a merge shrinks and shifts it in a
    // way plain re-insertion cannot reverse; the snapshot diff must restore it.
    let mut model = new_empty_user_model();
    // merge B2:C4
    model.merge_cells(&area(0, 2, 2, 2, 3)).unwrap();
    model.delete_rows(0, 1, 2).unwrap();
    assert_eq!(model.get_merged_cells(0).unwrap(), vec![merged(1, 2, 2, 2)]);

    model.undo().unwrap();
    assert_eq!(model.get_merged_cells(0).unwrap(), vec![merged(2, 2, 2, 3)]);

    model.redo().unwrap();
    assert_eq!(model.get_merged_cells(0).unwrap(), vec![merged(1, 2, 2, 2)]);
}

#[test]
fn insert_rows_into_a_merge_is_undoable() {
    let mut model = new_empty_user_model();
    // merge B2:C3
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();
    model.insert_rows(0, 3, 2).unwrap();
    assert_eq!(model.get_merged_cells(0).unwrap(), vec![merged(2, 2, 2, 4)]);

    model.undo().unwrap();
    assert_eq!(model.get_merged_cells(0).unwrap(), vec![merged(2, 2, 2, 2)]);
}

#[test]
fn structural_ops_replay_on_external_model() {
    let mut model1 = new_empty_user_model();
    let mut model2 = new_empty_user_model();

    model1.merge_cells(&area(0, 2, 2, 2, 3)).unwrap();
    model1.delete_rows(0, 1, 2).unwrap();
    model2
        .apply_external_diffs(&model1.flush_send_queue())
        .unwrap();
    assert_eq!(
        model2.get_merged_cells(0).unwrap(),
        model1.get_merged_cells(0).unwrap()
    );

    // undo on model1 replays on model2
    model1.undo().unwrap();
    model2
        .apply_external_diffs(&model1.flush_send_queue())
        .unwrap();
    assert_eq!(
        model2.get_merged_cells(0).unwrap(),
        vec![merged(2, 2, 2, 3)]
    );
}

// ── Navigation and selection ─────────────────────────────────────────────────

fn selected(model: &UserModel) -> (i32, i32, [i32; 4]) {
    let view = model.get_selected_view();
    (view.row, view.column, view.range)
}

#[test]
fn full_row_and_column_selections_do_not_grow_over_merges() {
    let mut model = new_empty_user_model();
    // merge B2:C3
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();

    // Like in Excel, selecting a row that slices through a merged range
    // selects just that row, it does not drag the other rows of the merge in
    model.set_selected_cell(2, 1).unwrap();
    model.set_selected_range(2, 1, 2, LAST_COLUMN).unwrap();
    assert_eq!(selected(&model).2, [2, 1, 2, LAST_COLUMN]);

    // same for a column crossing the merge
    model.set_selected_cell(1, 2).unwrap();
    model.set_selected_range(1, 2, LAST_ROW, 2).unwrap();
    assert_eq!(selected(&model).2, [1, 2, LAST_ROW, 2]);
}

#[test]
fn selecting_a_covered_cell_selects_the_anchor() {
    let mut model = new_empty_user_model();
    // merge B2:D4
    model.merge_cells(&area(0, 2, 2, 3, 3)).unwrap();

    model.set_selected_cell(3, 3).unwrap();
    assert_eq!(selected(&model), (2, 2, [2, 2, 4, 4]));

    model.set_selected_cell(1, 1).unwrap();
    assert_eq!(selected(&model), (1, 1, [1, 1, 1, 1]));
}

#[test]
fn arrows_skip_over_merges() {
    let mut model = new_empty_user_model();
    // merge B2:D4
    model.merge_cells(&area(0, 2, 2, 3, 3)).unwrap();

    // horizontally through the anchor row
    model.set_selected_cell(2, 1).unwrap();
    model.on_arrow_right().unwrap();
    assert_eq!(selected(&model), (2, 2, [2, 2, 4, 4]));
    model.on_arrow_right().unwrap();
    assert_eq!(selected(&model), (2, 5, [2, 5, 2, 5]));
    model.on_arrow_left().unwrap();
    assert_eq!(selected(&model), (2, 2, [2, 2, 4, 4]));
    model.on_arrow_left().unwrap();
    assert_eq!(selected(&model), (2, 1, [2, 1, 2, 1]));

    // vertically through the anchor column
    model.set_selected_cell(1, 2).unwrap();
    model.on_arrow_down().unwrap();
    assert_eq!(selected(&model), (2, 2, [2, 2, 4, 4]));
    model.on_arrow_down().unwrap();
    assert_eq!(selected(&model), (5, 2, [5, 2, 5, 2]));
    model.on_arrow_up().unwrap();
    assert_eq!(selected(&model), (2, 2, [2, 2, 4, 4]));
    model.on_arrow_up().unwrap();
    assert_eq!(selected(&model), (1, 2, [1, 2, 1, 2]));
}

#[test]
fn arrows_enter_merges_from_covered_rows_and_columns() {
    let mut model = new_empty_user_model();
    // merge B2:D4
    model.merge_cells(&area(0, 2, 2, 3, 3)).unwrap();

    // entering horizontally on a covered row snaps to the anchor;
    // leaving continues from the anchor's row (v1 simplification)
    model.set_selected_cell(3, 1).unwrap();
    model.on_arrow_right().unwrap();
    assert_eq!(selected(&model), (2, 2, [2, 2, 4, 4]));
    model.on_arrow_right().unwrap();
    assert_eq!(selected(&model), (2, 5, [2, 5, 2, 5]));

    // entering vertically on a covered column snaps to the anchor
    model.set_selected_cell(1, 4).unwrap();
    model.on_arrow_down().unwrap();
    assert_eq!(selected(&model), (2, 2, [2, 2, 4, 4]));
}

#[test]
fn expanding_selection_grows_over_merges() {
    let mut model = new_empty_user_model();
    // merge B2:D4
    model.merge_cells(&area(0, 2, 2, 3, 3)).unwrap();

    model.set_selected_cell(1, 1).unwrap();
    model.on_expand_selected_range("ArrowDown").unwrap();
    // A1:A2 does not touch the merge
    assert_eq!(selected(&model).2, [1, 1, 2, 1]);

    model.on_expand_selected_range("ArrowRight").unwrap();
    // A1:B2 touches the merge: grows to contain it fully
    assert_eq!(selected(&model).2, [1, 1, 4, 4]);
}

#[test]
fn area_selecting_grows_over_merges() {
    let mut model = new_empty_user_model();
    // merges B2:C3 and E3:F4: growing over the first one grazes the second
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();
    model.merge_cells(&area(0, 3, 5, 2, 2)).unwrap();

    model.set_selected_cell(1, 1).unwrap();
    model.on_area_selecting(2, 4).unwrap();
    // A1:D2 covers part of B2:C3 -> grows to row 3 -> touches E3:F4? no
    // (column 5 not included): range is A1:D3
    assert_eq!(selected(&model).2, [1, 1, 3, 4]);

    model.on_area_selecting(3, 5).unwrap();
    // now both merges are grazed: fixpoint covers A1:F4
    assert_eq!(selected(&model).2, [1, 1, 4, 6]);
}

#[test]
fn navigate_to_edge_treats_merge_as_one_cell() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 2, 2, "5").unwrap();
    // merge B2:D4, content at the anchor
    model.merge_cells(&area(0, 2, 2, 3, 3)).unwrap();
    model.set_user_input(0, 2, 6, "7").unwrap();

    // ctrl+right from A2 lands on the merge
    model.set_selected_cell(2, 1).unwrap();
    model
        .on_navigate_to_edge_in_direction(crate::worksheet::NavigationDirection::Right)
        .unwrap();
    assert_eq!(selected(&model), (2, 2, [2, 2, 4, 4]));

    // ctrl+right from the merge jumps past it to F2
    model
        .on_navigate_to_edge_in_direction(crate::worksheet::NavigationDirection::Right)
        .unwrap();
    assert_eq!(selected(&model), (2, 6, [2, 6, 2, 6]));
}

// ── Autofill and clipboard ───────────────────────────────────────────────────

#[test]
fn auto_fill_over_merges_is_rejected() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 2, "1").unwrap();
    // merge B3:C4
    model.merge_cells(&area(0, 3, 2, 2, 2)).unwrap();
    model.flush_send_queue();

    // filling B1 down to B4 crosses the merge
    assert_eq!(
        model.auto_fill_rows(&area(0, 1, 2, 1, 1), 4),
        Err("Cannot auto-fill over merged cells".to_string())
    );
    // filling B1 right to C1 is fine (the merge is below)
    model.auto_fill_columns(&area(0, 1, 2, 1, 1), 3).unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 3), Ok("1".to_string()));
}

#[test]
fn paste_over_merges_is_rejected() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "42").unwrap();
    // merge A2:B3
    model.merge_cells(&area(0, 2, 1, 2, 2)).unwrap();

    model.set_selected_cell(1, 1).unwrap();
    let clipboard = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(2, 1).unwrap();
    assert_eq!(
        model.paste_from_clipboard(0, clipboard.range, &clipboard.data, false),
        Err("Cannot paste over merged cells".to_string())
    );
    // the model is untouched
    assert_eq!(model.get_merged_cells(0).unwrap(), vec![merged(2, 1, 2, 2)]);
    assert_eq!(model.get_formatted_cell_value(0, 2, 1), Ok("".to_string()));
}

#[test]
fn copy_paste_recreates_the_merge_at_the_target() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 2, 2, "5").unwrap();
    // merge B2:C3
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();

    // select the merged cell (selection covers the whole merge) and copy it
    model.set_selected_cell(2, 2).unwrap();
    let clipboard = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(10, 10).unwrap();
    model
        .paste_from_clipboard(0, clipboard.range, &clipboard.data, false)
        .unwrap();

    // the source merge is untouched and the target got its own J10:K11 merge
    assert_eq!(
        model.get_merged_cells(0).unwrap(),
        vec![merged(2, 2, 2, 2), merged(10, 10, 2, 2)]
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 10, 10),
        Ok("5".to_string())
    );

    model.undo().unwrap();
    assert_eq!(model.get_merged_cells(0).unwrap(), vec![merged(2, 2, 2, 2)]);
    assert_eq!(
        model.get_formatted_cell_value(0, 10, 10),
        Ok("".to_string())
    );

    model.redo().unwrap();
    assert_eq!(
        model.get_merged_cells(0).unwrap(),
        vec![merged(2, 2, 2, 2), merged(10, 10, 2, 2)]
    );
}

#[test]
fn copy_paste_keeps_merges_at_their_relative_position() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "a").unwrap();
    model.set_user_input(0, 2, 2, "5").unwrap();
    model.set_user_input(0, 4, 4, "x").unwrap();
    // merge B2:C3, in the middle of the copied area
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();

    // copy A1:D4 and paste it at F1
    model.set_selected_range(1, 1, 4, 4).unwrap();
    let clipboard = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(1, 6).unwrap();
    model
        .paste_from_clipboard(0, clipboard.range, &clipboard.data, false)
        .unwrap();

    // the pasted merge keeps its offset inside the area: G2:H3
    assert_eq!(
        model.get_merged_cells(0).unwrap(),
        vec![merged(2, 2, 2, 2), merged(2, 7, 2, 2)]
    );
    assert_eq!(model.get_formatted_cell_value(0, 2, 7), Ok("5".to_string()));
}

#[test]
fn cut_paste_moves_the_merge_to_the_target() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 2, 2, "5").unwrap();
    // merge B2:C3
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();

    // select the merged cell (selection covers the whole merge) and cut it
    model.set_selected_cell(2, 2).unwrap();
    let clipboard = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(10, 10).unwrap();
    model
        .paste_from_clipboard(0, clipboard.range, &clipboard.data, true)
        .unwrap();

    // the merge moved with its content: gone from the source, at J10:K11 now
    assert_eq!(
        model.get_merged_cells(0).unwrap(),
        vec![merged(10, 10, 2, 2)]
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 10, 10),
        Ok("5".to_string())
    );
    assert_eq!(model.get_formatted_cell_value(0, 2, 2), Ok("".to_string()));

    // undo moves the merge and its content back to the source
    model.undo().unwrap();
    assert_eq!(model.get_merged_cells(0).unwrap(), vec![merged(2, 2, 2, 2)]);
    assert_eq!(model.get_formatted_cell_value(0, 2, 2), Ok("5".to_string()));
    assert_eq!(
        model.get_formatted_cell_value(0, 10, 10),
        Ok("".to_string())
    );

    model.redo().unwrap();
    assert_eq!(
        model.get_merged_cells(0).unwrap(),
        vec![merged(10, 10, 2, 2)]
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 10, 10),
        Ok("5".to_string())
    );
}

#[test]
fn paste_csv_over_merges_is_rejected() {
    let mut model = new_empty_user_model();
    // merge B2:C3
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();
    assert_eq!(
        model.paste_csv_string(&area(0, 1, 1, 1, 1), "1\t2\n3\t4"),
        Err("Cannot paste over merged cells".to_string())
    );
    // pasting away from the merge works
    model.set_selected_cell(10, 1).unwrap();
    model
        .paste_csv_string(&area(0, 10, 1, 1, 1), "1\t2\n3\t4")
        .unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 11, 2),
        Ok("4".to_string())
    );
}

// Merging a range whose selected cell is not the anchor must move the
// selection to the anchor: a covered cell can never stay selected (the UI
// draws the selection outline from the selected cell, sized like the merged
// rectangle, so a selected covered cell renders a shifted selection).
#[test]
fn merging_snaps_the_selection_to_the_anchor() {
    let mut model = new_empty_user_model();
    // Select H20 and extend the selection to G19: H20 stays the selected cell
    model.set_selected_cell(20, 8).unwrap();
    model.on_area_selecting(19, 7).unwrap();
    let view = model.get_selected_view();
    assert_eq!(view.range, [19, 7, 20, 8]);
    assert_eq!((view.row, view.column), (20, 8));

    // Merge the selected area, as the toolbar button does
    model.merge_cells(&area(0, 19, 7, 2, 2)).unwrap();

    // The selected cell must be the anchor G19 and the selected range the
    // whole merged range
    let view = model.get_selected_view();
    assert_eq!((view.row, view.column), (19, 7));
    assert_eq!(view.range, [19, 7, 20, 8]);
}
