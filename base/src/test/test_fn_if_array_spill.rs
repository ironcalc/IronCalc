#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Group/value model shared by the tests, shaped like a pivot source:
//   A1:A6 = A,A,B,B,C,A   (group / criteria range)
//   B1:B6 = 10,20,5,5,100,30 (value range)
// so the groups aggregate to: A → {10,20,30}, B → {5,5}, C → {100}.
fn group_model() -> crate::model::Model<'static> {
    let mut model = new_empty_model();
    for (i, (a, b)) in [
        ("A", 10),
        ("A", 20),
        ("B", 5),
        ("B", 5),
        ("C", 100),
        ("A", 30),
    ]
    .iter()
    .enumerate()
    {
        let row = i + 1;
        model._set(&format!("A{row}"), a);
        model._set(&format!("B{row}"), &b.to_string());
    }
    model
}

// ── COUNTIF ───────────────────────────────────────────────────────────────────

#[test]
fn countif_scalar_criterion_is_unchanged() {
    let mut model = group_model();
    model._set("D1", r#"=COUNTIF(A1:A6,"A")"#);
    model.evaluate();
    assert_eq!(model._get_text("D1"), "3");
}

#[test]
fn countif_inline_array_criteria_spill_horizontally() {
    let mut model = group_model();
    model._set("D1", r#"=COUNTIF(A1:A6,{"A","B","C"})"#);
    model.evaluate();
    assert_eq!(model._get_text("D1"), "3"); // A
    assert_eq!(model._get_text("E1"), "2"); // B
    assert_eq!(model._get_text("F1"), "1"); // C
}

#[test]
fn countif_range_criteria_spill_like_a_pivot() {
    // The pivot shape: the groups sit in a (spilled) range and COUNTIF spills one
    // count per group, aligned to that range.
    let mut model = group_model();
    model._set("G1", "A");
    model._set("G2", "B");
    model._set("G3", "C");
    model._set("H1", "=COUNTIF(A1:A6,G1:G3)");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "3");
    assert_eq!(model._get_text("H2"), "2");
    assert_eq!(model._get_text("H3"), "1");
}

#[test]
fn countif_array_criteria_with_no_match_is_zero() {
    let mut model = group_model();
    model._set("D1", r#"=COUNTIF(A1:A6,{"A","Z"})"#);
    model.evaluate();
    assert_eq!(model._get_text("D1"), "3");
    assert_eq!(model._get_text("E1"), "0");
}

// ── AVERAGEIF ─────────────────────────────────────────────────────────────────

#[test]
fn averageif_scalar_criterion_is_unchanged() {
    let mut model = group_model();
    model._set("D1", r#"=AVERAGEIF(A1:A6,"A",B1:B6)"#);
    model.evaluate();
    assert_eq!(model._get_text("D1"), "20"); // (10+20+30)/3
}

#[test]
fn averageif_array_criteria_spill() {
    let mut model = group_model();
    model._set("D1", r#"=AVERAGEIF(A1:A6,{"A","B","C"},B1:B6)"#);
    model.evaluate();
    assert_eq!(model._get_text("D1"), "20"); // A → 60/3
    assert_eq!(model._get_text("E1"), "5"); //  B → 10/2
    assert_eq!(model._get_text("F1"), "100"); // C → 100/1
}

#[test]
fn averageif_array_criteria_no_match_is_div_error() {
    let mut model = group_model();
    model._set("D1", r#"=AVERAGEIF(A1:A6,{"A","Z"},B1:B6)"#);
    model.evaluate();
    assert_eq!(model._get_text("D1"), "20");
    assert_eq!(model._get_text("E1"), "#DIV/0!");
}

// ── MINIFS ────────────────────────────────────────────────────────────────────

#[test]
fn minifs_scalar_criterion_is_unchanged() {
    let mut model = group_model();
    model._set("D1", r#"=MINIFS(B1:B6,A1:A6,"A")"#);
    model.evaluate();
    assert_eq!(model._get_text("D1"), "10");
}

#[test]
fn minifs_array_criteria_spill() {
    let mut model = group_model();
    model._set("D1", r#"=MINIFS(B1:B6,A1:A6,{"A","B","C"})"#);
    model.evaluate();
    assert_eq!(model._get_text("D1"), "10"); // A
    assert_eq!(model._get_text("E1"), "5"); //  B
    assert_eq!(model._get_text("F1"), "100"); // C
}

#[test]
fn minifs_array_criteria_no_match_is_zero() {
    let mut model = group_model();
    model._set("D1", r#"=MINIFS(B1:B6,A1:A6,{"A","Z"})"#);
    model.evaluate();
    assert_eq!(model._get_text("D1"), "10");
    assert_eq!(model._get_text("E1"), "0");
}

// ── MAXIFS ────────────────────────────────────────────────────────────────────

#[test]
fn maxifs_scalar_criterion_is_unchanged() {
    let mut model = group_model();
    model._set("D1", r#"=MAXIFS(B1:B6,A1:A6,"A")"#);
    model.evaluate();
    assert_eq!(model._get_text("D1"), "30");
}

#[test]
fn maxifs_array_criteria_spill() {
    let mut model = group_model();
    model._set("D1", r#"=MAXIFS(B1:B6,A1:A6,{"A","B","C"})"#);
    model.evaluate();
    assert_eq!(model._get_text("D1"), "30"); // A
    assert_eq!(model._get_text("E1"), "5"); //  B
    assert_eq!(model._get_text("F1"), "100"); // C
}

#[test]
fn maxifs_array_criteria_no_match_is_zero() {
    let mut model = group_model();
    model._set("D1", r#"=MAXIFS(B1:B6,A1:A6,{"A","Z"})"#);
    model.evaluate();
    assert_eq!(model._get_text("D1"), "30");
    assert_eq!(model._get_text("E1"), "0");
}

// ── Open-column criteria range ────────────────────────────────────────────────
// COUNTIF counts through the COUNTIFS walk, clipped to the used area, so a whole
// column is not read once per criterion. The counts still match.

#[test]
fn countif_whole_column_array_criteria_spill() {
    let mut model = group_model(); // A1:A6 = A,A,B,B,C,A
    model._set("D1", r#"=COUNTIF(A:A,{"A","B","C"})"#);
    model.evaluate();
    assert_eq!(model._get_text("D1"), "3"); // A
    assert_eq!(model._get_text("E1"), "2"); // B
    assert_eq!(model._get_text("F1"), "1"); // C
}
