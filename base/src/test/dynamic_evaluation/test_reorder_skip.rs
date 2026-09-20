#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// A second evaluation of a model whose spill areas are already materialized
// (the imported-workbook shape) must not flag any spill as changed: every
// anchor keeps its dimensions, so the reorder scan is skipped entirely.
#[test]
fn second_evaluation_flags_no_shape_changes() {
    let mut model = new_empty_model();

    model._set("A1", "=B2+D2");
    model._set("B1", "=D1:D2");
    model._set("D1", "=E1:E2");
    model._set("E1", "10");
    model._set("E2", "20");

    model.evaluate();
    // The spill areas now exist with their final dimensions, like a workbook
    // imported from xlsx. Re-evaluating must produce the same values.
    model.evaluate();

    assert_eq!(model._get_text("D1"), "10");
    assert_eq!(model._get_text("D2"), "20");
    assert_eq!(model._get_text("B1"), "10");
    assert_eq!(model._get_text("B2"), "20");
    assert_eq!(model._get_text("A1"), "40");
    // No spill changed shape in the steady-state pass.
    assert!(model.changed_spill_shapes.is_empty());
}

// Genuine reorder case: anchor B1 depends on anchor C1's spill area and is
// listed first in natural order. On the first evaluation both spills change
// shape (from 1x1), so the changed-shape path must still run the reorder scan
// and converge to correct values.
#[test]
fn dependent_anchor_listed_first_still_reorders() {
    let mut model = new_empty_model();

    model._set("B1", "=C1:C3");
    model._set("C1", "=SEQUENCE(3)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "1");
    assert_eq!(model._get_text("B2"), "2");
    assert_eq!(model._get_text("B3"), "3");
    assert_eq!(model._get_text("C1"), "1");
    assert_eq!(model._get_text("C2"), "2");
    assert_eq!(model._get_text("C3"), "3");
}

// Shape grow and shrink across evaluations: the anchor is flagged as changed
// on the passes where its dimensions move, and dependents stay correct.
#[test]
fn shape_grow_and_shrink() {
    let mut model = new_empty_model();

    model._set("B1", "=C1:C4");
    model._set("C1", "=SEQUENCE(E1)");
    model._set("E1", "2");

    model.evaluate();
    assert_eq!(model._get_text("C1"), "1");
    assert_eq!(model._get_text("C2"), "2");
    assert_eq!(model._get_text("C3"), "");
    assert_eq!(model._get_text("B2"), "2");

    // grow 2x1 -> 3x1
    model._set("E1", "3");
    model.evaluate();
    assert_eq!(model._get_text("C3"), "3");
    assert_eq!(model._get_text("B3"), "3");

    // shrink 3x1 -> 1x1
    model._set("E1", "1");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "1");
    assert_eq!(model._get_text("C2"), "");
    assert_eq!(model._get_text("C3"), "");
    assert_eq!(model._get_text("B1"), "1");
    assert_eq!(model._get_text("B2"), "0");

    // steady state after the shrink: nothing changes shape any more
    model.evaluate();
    assert!(model.changed_spill_shapes.is_empty());
}

// A dynamic anchor that collapses from a spilled area to a scalar result
// (here via a #SPILL!-style shrink to an error/scalar) is flagged as changed
// on that pass and dependents see the new state.
#[test]
fn spill_collapse_to_scalar_is_a_shape_change() {
    let mut model = new_empty_model();

    model._set("C1", "=SEQUENCE(E1)");
    model._set("E1", "3");
    model.evaluate();
    assert_eq!(model._get_text("C3"), "3");

    // SEQUENCE(-1) is an error: the anchor collapses to 1x1.
    model._set("E1", "-1");
    model.evaluate();
    assert_eq!(model._get_text("C2"), "");
    assert_eq!(model._get_text("C3"), "");

    model.evaluate();
    assert!(model.changed_spill_shapes.is_empty());
}
