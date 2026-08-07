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
