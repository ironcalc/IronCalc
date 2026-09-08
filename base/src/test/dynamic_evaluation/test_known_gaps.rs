#![allow(clippy::unwrap_used)]

// Known gaps of the two-phase ordering algorithm.

use crate::cell::CellValue;
use crate::test::util::new_empty_model;
use crate::Model;

//         ║    A    |    B    |      C       |
// ════════╬═════════╪═════════╪══════════════╪
//    1    ║         | =A5:A6  | =SEQUENCE(3) |
// ────────╫─────────┼─────────┼──────────────┼
//    2    ║         |         |              |
// ────────╫─────────┼─────────┼──────────────┼
//    3    ║         |         |              |
// ────────╫─────────┼─────────┼──────────────┼
//    5    ║ =C2*2   |         |              |
// ────────╫─────────┼─────────┼──────────────┼
//    6    ║ =C3*2   |         |              |
// ────────╫─────────┼─────────┼──────────────┼
//
// B1 is a spill that depends on C1's spill area only *through* the regular
// cells A5 and A6. Phase 1 evaluates B1 before C1 and pulls in A5/A6, which
// read empty C2/C3. C1's area is not in B1's direct support (only A5:A6 is),
// so the reorder check never fires.
#[test]
fn transitive_dependency_through_regular_cell() {
    let mut model = new_empty_model();

    model._set("B1", "=A5:A6");
    model._set("A5", "=C2*2");
    model._set("A6", "=C3*2");
    model._set("C1", "=SEQUENCE(3)");

    model.evaluate();

    assert_eq!(model._get_text("C2"), "2");
    assert_eq!(model._get_text("C3"), "3");
    assert_eq!(model._get_text("A5"), "4");
    assert_eq!(model._get_text("A6"), "6");
    assert_eq!(model._get_text("B1"), "4");
    assert_eq!(model._get_text("B2"), "6");
}

//         ║    B    |      C       |
// ════════╬═════════╪══════════════╪
//    1    ║ =C1#    | =SEQUENCE(3) |
// ────────╫─────────┼──────────────┼
//    2    ║         |              |
// ────────╫─────────┼──────────────┼
//    3    ║         |              |
// ────────╫─────────┼──────────────┼
//
// The spill range operator reads the `r` stored in the anchor without
// evaluating it first. B1 runs before C1, sees r = (1, 1) and copies a single
// cell. `#` is evaluated through `evaluate_node_with_reference`, which records
// nothing in `support`, so no reorder happens either.
#[test]
fn spill_range_operator_before_anchor() {
    let mut model = new_empty_model();

    model._set("B1", "=C1#");
    model._set("C1", "=SEQUENCE(3)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "1");
    assert_eq!(model._get_text("B2"), "2");
    assert_eq!(model._get_text("B3"), "3");
}

//         ║    A    |            B            |      D       |
// ════════╬═════════╪═════════════════════════╪══════════════╪
//    1    ║ =B1:B1  | =SUM(OFFSET(D1,1,0,2,1)) | =SEQUENCE(3) |
// ────────╫─────────┼─────────────────────────┼──────────────┼
//    2    ║         |                         |              |
// ────────╫─────────┼─────────────────────────┼──────────────┼
//    3    ║         |                         |              |
// ────────╫─────────┼─────────────────────────┼──────────────┼
//
// A1 is a (trivial) spill evaluated in Phase 1 before D1; it pulls in B1,
// whose only reference into D's spill area is computed by OFFSET. Computed
// references are never recorded in `support`, so D1's area is never matched
// against anything and B1 keeps the value it read from empty cells.
#[test]
fn computed_reference_into_spill_area() {
    let mut model = new_empty_model();

    model._set("A1", "=B1:B1");
    model._set("B1", "=SUM(OFFSET(D1,1,0,2,1))");
    model._set("D1", "=SEQUENCE(3)");

    model.evaluate();

    assert_eq!(model._get_text("D2"), "2");
    assert_eq!(model._get_text("D3"), "3");
    assert_eq!(model._get_text("B1"), "5");
    assert_eq!(model._get_text("A1"), "5");
}

//         ║    A    |    B    |        C        |
// ════════╬═════════╪═════════╪═════════════════╪
//    1    ║         | =A5:A6  | =RANDARRAY(3,1) |
// ────────╫─────────┼─────────┼─────────────────┼
//    5    ║ =C2*2   |         |                 |
// ────────╫─────────┼─────────┼─────────────────┼
//    6    ║ =C3*2   |         |                 |
// ────────╫─────────┼─────────┼─────────────────┼
//
// Same shape as the transitive case but with a volatile source: after the
// retraction the readers must see exactly the values that ended up in C2:C3.
#[test]
fn transitive_dependency_on_volatile_spill_is_consistent() {
    let mut model = new_empty_model();

    model._set("B1", "=A5:A6");
    model._set("A5", "=C2*2");
    model._set("A6", "=C3*2");
    model._set("C1", "=RANDARRAY(3,1)");

    model.evaluate();

    // Compare the stored numbers, not their formatted text (which is rounded).
    let number = |model: &Model, cell: &str| match model.get_cell_value_by_ref(cell) {
        Ok(CellValue::Number(n)) => n,
        other => panic!("{cell} is not a number: {other:?}"),
    };
    let c2 = number(&model, "Sheet1!C2");
    let c3 = number(&model, "Sheet1!C3");
    assert_eq!(number(&model, "Sheet1!A5"), c2 * 2.0);
    assert_eq!(number(&model, "Sheet1!A6"), c3 * 2.0);
    assert_eq!(number(&model, "Sheet1!B1"), c2 * 2.0);
    assert_eq!(number(&model, "Sheet1!B2"), c3 * 2.0);
}

//         ║    A          |      C       |
// ════════╬═══════════════╪══════════════╪
//    1    ║ =SUM(C1#)     | =SEQUENCE(3) |
// ────────╫───────────────┼──────────────┼
//
// A scalar consumer of the spill range operator, evaluated before the anchor.
#[test]
fn sum_of_spill_range_before_anchor() {
    let mut model = new_empty_model();

    model._set("A1", "=SUM(C1#)");
    model._set("C1", "=SEQUENCE(3)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), "6");
}

// The spill range operator on a cell whose shape depends on itself is a cycle.
#[test]
fn spill_range_operator_on_itself_is_circular() {
    let mut model = new_empty_model();

    model._set("C1", "=SEQUENCE(ROWS(C1#)+1)");

    model.evaluate();

    assert_eq!(model._get_text("C1"), "#CIRC!");
}
