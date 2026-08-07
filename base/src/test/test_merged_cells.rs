#![allow(clippy::unwrap_used)]

use crate::expressions::types::Area;
use crate::merged_cells::MergeStructure;
use crate::test::util::new_empty_model;
use crate::types::{Color, MergedCell};

fn area(sheet: u32, row: i32, column: i32, width: i32, height: i32) -> Area {
    Area {
        sheet,
        row,
        column,
        width,
        height,
    }
}

#[test]
fn merge_keeps_anchor_and_clears_covered() {
    let mut model = new_empty_model();
    model._set("B2", "5");
    model._set("C2", "7");
    model._set("C3", "hello");
    model.evaluate();

    // Merge B2:C3
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();
    model.evaluate();

    assert_eq!(model._get_text("B2"), "5");
    assert_eq!(model._get_text("C2"), "");
    assert_eq!(model._get_text("C3"), "");
    assert_eq!(
        model.get_merged_cells(0).unwrap(),
        &[MergedCell {
            row: 2,
            column: 2,
            width: 2,
            height: 2
        }]
    );
}

#[test]
fn merge_preserves_covered_styles() {
    let mut model = new_empty_model();
    model._set("C2", "7");
    let mut style = model.get_style_for_cell(0, 2, 3).unwrap();
    style.fill.color = Color::Rgb("#FF0000".to_string());
    model.set_cell_style(0, 2, 3, &style).unwrap();

    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();
    model.evaluate();

    assert_eq!(model._get_text("C2"), "");
    let style_after = model.get_style_for_cell(0, 2, 3).unwrap();
    assert_eq!(style_after.fill.color, Color::Rgb("#FF0000".to_string()));
}

#[test]
fn merge_invalid_ranges() {
    let mut model = new_empty_model();
    // single cell
    assert_eq!(
        model.merge_cells(&area(0, 2, 2, 1, 1)),
        Err("Cannot merge a single cell".to_string())
    );
    // invalid coordinates
    assert_eq!(
        model.merge_cells(&area(0, 0, 1, 2, 2)),
        Err("Invalid range".to_string())
    );
    assert_eq!(
        model.merge_cells(&area(0, 1, 1, 0, 2)),
        Err("Invalid range".to_string())
    );
    // out of bounds
    assert_eq!(
        model.merge_cells(&area(0, crate::constants::LAST_ROW, 1, 1, 2)),
        Err("Range is out of bounds".to_string())
    );
    // invalid sheet
    assert_eq!(
        model.merge_cells(&area(13, 1, 1, 2, 2)),
        Err("Invalid sheet index".to_string())
    );
    assert!(model.get_merged_cells(0).unwrap().is_empty());
}

#[test]
fn merge_overlap_rejected() {
    let mut model = new_empty_model();
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();
    // partial overlap
    assert!(model.merge_cells(&area(0, 3, 3, 2, 2)).is_err());
    // superset
    assert!(model.merge_cells(&area(0, 1, 1, 6, 6)).is_err());
    // disjoint is fine
    model.merge_cells(&area(0, 4, 4, 2, 2)).unwrap();
    assert_eq!(model.get_merged_cells(0).unwrap().len(), 2);
}

#[test]
fn merge_over_cse_array_formula_rejected() {
    let mut model = new_empty_model();
    // A1:B2 is a CSE array formula
    model.set_user_array_formula(0, 1, 1, 2, 2, "=1").unwrap();
    model.evaluate();

    assert_eq!(
        model.merge_cells(&area(0, 1, 1, 2, 2)),
        Err("Cannot merge cells that intersect an array formula".to_string())
    );
    // partial intersection is also rejected
    assert_eq!(
        model.merge_cells(&area(0, 2, 2, 2, 2)),
        Err("Cannot merge cells that intersect an array formula".to_string())
    );
}

#[test]
fn covered_cell_write_rejected() {
    let mut model = new_empty_model();
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();

    assert_eq!(
        model.set_user_input(0, 3, 3, "42".to_string()),
        Err("Cannot edit a cell that is part of a merged cell".to_string())
    );
    // the anchor is editable
    model.set_user_input(0, 2, 2, "42".to_string()).unwrap();
    model.evaluate();
    assert_eq!(model._get_text("B2"), "42");

    // after unmerging, the covered cell is editable again
    model.unmerge_cells(&area(0, 2, 2, 2, 2)).unwrap();
    model.set_user_input(0, 3, 3, "43".to_string()).unwrap();
    model.evaluate();
    assert_eq!(model._get_text("C3"), "43");
}

#[test]
fn array_formula_over_merge_rejected() {
    let mut model = new_empty_model();
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();

    assert_eq!(
        model.set_user_array_formula(0, 2, 2, 2, 2, "=1"),
        Err("Cannot set an array formula over merged cells".to_string())
    );
    // partial intersection is also rejected
    assert_eq!(
        model.set_user_array_formula(0, 1, 1, 2, 2, "=1"),
        Err("Cannot set an array formula over merged cells".to_string())
    );
}

#[test]
fn spill_into_merge_is_spill_error() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    // C2:D3 merged blocks the spill of C1
    model.merge_cells(&area(0, 2, 3, 2, 2)).unwrap();
    model._set("C1", "=A1:A3");
    model.evaluate();

    assert_eq!(model._get_text("C1"), "#SPILL!");

    // unmerging lets the formula spill again
    model.unmerge_cells(&area(0, 2, 3, 2, 2)).unwrap();
    model.evaluate();
    assert_eq!(model._get_text("C1"), "10");
    assert_eq!(model._get_text("C2"), "20");
    assert_eq!(model._get_text("C3"), "30");
}

#[test]
fn merge_over_dynamic_spill_children() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("C1", "=A1:A3");
    model.evaluate();
    assert_eq!(model._get_text("C3"), "30");

    // Merging over the spill children (not the anchor) resets the anchor,
    // which then re-evaluates to #SPILL! because the merge blocks it.
    model.merge_cells(&area(0, 2, 3, 2, 2)).unwrap();
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#SPILL!");

    model.unmerge_cells(&area(0, 2, 3, 2, 2)).unwrap();
    model.evaluate();
    assert_eq!(model._get_text("C1"), "10");
    assert_eq!(model._get_text("C3"), "30");
}

#[test]
fn merge_over_dynamic_anchor_clears_formula() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    // C2 spills C2:C4
    model._set("C2", "=A1:A3");
    model.evaluate();
    assert_eq!(model._get_text("C4"), "30");

    // B2:C3 covers the anchor C2 as a covered cell: the formula is deleted
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();
    model.evaluate();
    assert_eq!(model._get_text("C2"), "");
    assert_eq!(model._get_text("C4"), "");
    assert!(!model._has_formula("C2"));
}

#[test]
fn reference_to_covered_cell_is_zero() {
    let mut model = new_empty_model();
    model._set("B2", "7");
    // merge B2:C2
    model.merge_cells(&area(0, 2, 2, 2, 1)).unwrap();
    model._set("E1", "=C2");
    model._set("E2", "=SUM(B2:C2)");
    model.evaluate();

    assert_eq!(model._get_text("E1"), "0");
    assert_eq!(model._get_text("E2"), "7");
}

#[test]
fn unmerge_intersecting_removes_all_touched() {
    let mut model = new_empty_model();
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();
    model.merge_cells(&area(0, 2, 5, 2, 2)).unwrap();

    // A range touching only the second merge
    model.unmerge_cells(&area(0, 3, 6, 1, 1)).unwrap();
    assert_eq!(
        model.get_merged_cells(0).unwrap(),
        &[MergedCell {
            row: 2,
            column: 2,
            width: 2,
            height: 2
        }]
    );

    // A range covering everything
    model.unmerge_cells(&area(0, 1, 1, 20, 20)).unwrap();
    assert!(model.get_merged_cells(0).unwrap().is_empty());
}

#[test]
fn unmerge_without_merges_is_ok() {
    let mut model = new_empty_model();
    model.unmerge_cells(&area(0, 1, 1, 5, 5)).unwrap();
    assert!(model.get_merged_cells(0).unwrap().is_empty());
    // an invalid sheet is still an error
    assert!(model.unmerge_cells(&area(13, 1, 1, 5, 5)).is_err());
}

#[test]
fn merge_structure() {
    let mut model = new_empty_model();
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();

    assert_eq!(
        model.get_merge_structure(0, 2, 2).unwrap(),
        MergeStructure::Anchor {
            width: 2,
            height: 2
        }
    );
    assert_eq!(
        model.get_merge_structure(0, 3, 3).unwrap(),
        MergeStructure::Covered {
            anchor_row: 2,
            anchor_column: 2
        }
    );
    assert_eq!(
        model.get_merge_structure(0, 1, 1).unwrap(),
        MergeStructure::None
    );
}

#[test]
fn duplicate_sheet_copies_merges() {
    let mut model = new_empty_model();
    model._set("B2", "5");
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();

    let (_, new_sheet) = model.duplicate_sheet(0).unwrap();
    assert_eq!(
        model.get_merged_cells(new_sheet).unwrap(),
        &[MergedCell {
            row: 2,
            column: 2,
            width: 2,
            height: 2
        }]
    );
    // the copies are independent
    model.unmerge_cells(&area(new_sheet, 2, 2, 2, 2)).unwrap();
    assert_eq!(model.get_merged_cells(0).unwrap().len(), 1);
    assert!(model.get_merged_cells(new_sheet).unwrap().is_empty());
}

// ── Structural operations ────────────────────────────────────────────────────

fn merged(row: i32, column: i32, width: i32, height: i32) -> MergedCell {
    MergedCell {
        row,
        column,
        width,
        height,
    }
}

#[test]
fn insert_rows_displaces_merges() {
    // merge B2:C4 in every case
    let cases = [
        // (insert at row, expected merge after)
        (1, merged(3, 2, 2, 3)), // above: shifts down
        (2, merged(3, 2, 2, 3)), // at the anchor row: shifts down
        (3, merged(2, 2, 2, 4)), // strictly inside: grows
        (4, merged(2, 2, 2, 4)), // at the last row: grows
        (5, merged(2, 2, 2, 3)), // below: untouched
    ];
    for (insert_at, expected) in cases {
        let mut model = new_empty_model();
        model._set("B2", "5");
        model.merge_cells(&area(0, 2, 2, 2, 3)).unwrap();
        model.insert_rows(0, insert_at, 1).unwrap();
        assert_eq!(
            model.get_merged_cells(0).unwrap(),
            &[expected],
            "inserting a row at {insert_at}"
        );
    }
}

#[test]
fn delete_rows_displaces_merges() {
    // merge B2:C4 in every case
    let cases = [
        // (first deleted row, count, expected merges after)
        (1, 1, vec![merged(1, 2, 2, 3)]), // above: shifts up
        (1, 2, vec![merged(1, 2, 2, 2)]), // overlaps the top: shrinks + shifts
        (3, 1, vec![merged(2, 2, 2, 2)]), // strictly inside: shrinks
        (4, 2, vec![merged(2, 2, 2, 2)]), // overlaps the bottom: shrinks
        (2, 3, vec![]),                   // exactly the merge: removed
        (1, 5, vec![]),                   // superset: removed
        (5, 1, vec![merged(2, 2, 2, 3)]), // below: untouched
    ];
    for (delete_at, count, expected) in cases {
        let mut model = new_empty_model();
        model._set("B2", "5");
        model.merge_cells(&area(0, 2, 2, 2, 3)).unwrap();
        model.delete_rows(0, delete_at, count).unwrap();
        assert_eq!(
            model.get_merged_cells(0).unwrap(),
            &expected,
            "deleting {count} row(s) at {delete_at}"
        );
    }
}

#[test]
fn insert_and_delete_columns_displace_merges() {
    // merge B2:D3 (columns 2-4)
    let mut model = new_empty_model();
    model._set("B2", "5");
    model.merge_cells(&area(0, 2, 2, 3, 2)).unwrap();

    // insert a column inside: grows
    model.insert_columns(0, 3, 1).unwrap();
    assert_eq!(model.get_merged_cells(0).unwrap(), &[merged(2, 2, 4, 2)]);
    // insert a column before: shifts right
    model.insert_columns(0, 1, 1).unwrap();
    assert_eq!(model.get_merged_cells(0).unwrap(), &[merged(2, 3, 4, 2)]);
    // delete a column overlapping the left edge: shrinks + shifts
    model.delete_columns(0, 2, 2).unwrap();
    assert_eq!(model.get_merged_cells(0).unwrap(), &[merged(2, 2, 3, 2)]);
    // delete all its columns: removed
    model.delete_columns(0, 2, 3).unwrap();
    assert!(model.get_merged_cells(0).unwrap().is_empty());
}

#[test]
fn merge_shrunk_to_single_cell_is_removed() {
    let mut model = new_empty_model();
    // vertical merge B2:B3
    model.merge_cells(&area(0, 2, 2, 1, 2)).unwrap();
    model.delete_rows(0, 3, 1).unwrap();
    assert!(model.get_merged_cells(0).unwrap().is_empty());

    // horizontal merge B2:C2
    model.merge_cells(&area(0, 2, 2, 2, 1)).unwrap();
    model.delete_columns(0, 3, 1).unwrap();
    assert!(model.get_merged_cells(0).unwrap().is_empty());
}

#[test]
fn insert_rows_clamps_merges_at_the_bottom() {
    let mut model = new_empty_model();
    let last_row = crate::constants::LAST_ROW;
    // vertical merge in the last two rows of column B
    model.merge_cells(&area(0, last_row - 1, 2, 1, 2)).unwrap();
    // the merge is pushed against the edge and collapses to a single cell
    model.insert_rows(0, 1, 1).unwrap();
    assert!(model.get_merged_cells(0).unwrap().is_empty());
}

#[test]
fn move_columns_with_merges() {
    let mut model = new_empty_model();
    model._set("B2", "5");
    // merge B2:C3 (columns 2-3)
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();

    // moving only column C would split the merge
    assert_eq!(
        model.move_columns_action(0, 3, 1, 2),
        Err("Cannot move columns because that would split a merged cell".to_string())
    );

    // moving both columns two to the right carries the merge along
    model.move_columns_action(0, 2, 2, 2).unwrap();
    model.evaluate();
    assert_eq!(model.get_merged_cells(0).unwrap(), &[merged(2, 4, 2, 2)]);
    assert_eq!(model._get_text("D2"), "5");
    assert_eq!(model._get_text("B2"), "");
}

#[test]
fn move_columns_displaced_zone_shifts_merge() {
    let mut model = new_empty_model();
    model._set("D2", "7");
    // merge D2:E3 (columns 4-5)
    model.merge_cells(&area(0, 2, 4, 2, 2)).unwrap();

    // move column B (2) to column F (delta 4): the displaced zone is C..F,
    // the merge is fully inside it and shifts one to the left
    model.move_columns_action(0, 2, 1, 4).unwrap();
    model.evaluate();
    assert_eq!(model.get_merged_cells(0).unwrap(), &[merged(2, 3, 2, 2)]);
    assert_eq!(model._get_text("C2"), "7");
}

#[test]
fn move_rows_with_merges() {
    let mut model = new_empty_model();
    model._set("B2", "5");
    // merge B2:C3 (rows 2-3)
    model.merge_cells(&area(0, 2, 2, 2, 2)).unwrap();

    // moving only row 3 would split the merge
    assert_eq!(
        model.move_rows_action(0, 3, 1, 2),
        Err("Cannot move rows because that would split a merged cell".to_string())
    );

    // moving both rows down carries the merge along
    model.move_rows_action(0, 2, 2, 2).unwrap();
    model.evaluate();
    assert_eq!(model.get_merged_cells(0).unwrap(), &[merged(4, 2, 2, 2)]);
    assert_eq!(model._get_text("B4"), "5");
}
