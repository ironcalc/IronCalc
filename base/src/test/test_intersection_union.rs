#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

fn fill(model: &mut crate::Model) {
    for (i, v) in ["1", "2", "3", "4", "5"].iter().enumerate() {
        model._set(&format!("A{}", i + 1), v);
        model._set(&format!("B{}", i + 1), &format!("{}", (i + 1) * 10));
        model._set(&format!("C{}", i + 1), &format!("{}", (i + 1) * 100));
    }
}

// The space operator gives the cells two references share.
#[test]
fn intersection_operator() {
    let mut model = new_empty_model();
    fill(&mut model);
    model._set("E1", "=SUM(A1:A5 A3:C3)");
    model._set("E2", "=A1:C5 B2:B2");
    model._set("E3", "=SUM(A1:A2 C1:C2)");
    model._set("E4", "=SUM(A1:C5 B2:C3)");
    model._set("E5", "=SUM(A1:C5 B:B 2:3)");
    model._set("E6", "=SUM(Sheet1!A1:A5 Sheet1!A3:C3)");
    // Spaces that are not the operator
    model._set("F1", "=A1 +A2");
    model._set("F2", "=SUM(A1, A2 )");
    model._set("F3", "= A1 * 2");
    model._set("F4", "=IF(A1 =1, \"one\" , \"other\")");
    model.evaluate();

    assert_eq!(model._get_text("E1"), "3");
    assert_eq!(model._get_text("E2"), "20");
    assert_eq!(model._get_text("E3"), "#NULL!");
    assert_eq!(model._get_text("E4"), "550");
    assert_eq!(model._get_text("E5"), "50");
    assert_eq!(model._get_text("E6"), "3");
    assert_eq!(model._get_text("F1"), "3");
    assert_eq!(model._get_text("F2"), "3");
    assert_eq!(model._get_text("F3"), "2");
    assert_eq!(model._get_text("F4"), "one");

    assert_eq!(model._get_formula("E1"), "=SUM(A1:A5 A3:C3)");
    assert_eq!(model._get_formula("E5"), "=SUM(A1:C5 B:B 2:3)");
}

// References in an intersection move when rows are inserted.
#[test]
fn intersection_moves_with_rows() {
    let mut model = new_empty_model();
    fill(&mut model);
    model._set("E1", "=SUM(A1:A5 A3:C3)");
    model.evaluate();
    model.insert_rows(0, 2, 1).unwrap();
    model.evaluate();
    assert_eq!(model._get_formula("E1"), "=SUM(A1:A6 A4:C4)");
    assert_eq!(model._get_text("E1"), "3");
}

// A union of references in brackets: (A1,C1:C3)
#[test]
fn union_operator() {
    let mut model = new_empty_model();
    fill(&mut model);
    model._set("E1", "=SUM((A1,C1))");
    model._set("E2", "=SUM((A1:A2,A4:A5))");
    model._set("E3", "=AREAS((A1,B2))");
    model._set("E4", "=AREAS((A1,B2,C3:D4))");
    model._set("E5", "=AREAS(A1:B2)");
    model._set("E6", "=COUNT((A1:A5,B1))");
    model._set("E7", "=INDEX((A1:A5,C1:C5),2,1,2)");
    model._set("E8", "=(A1,B1)");
    model._set("E9", "=SUM(A1,(A2,A3))");
    model._set("E10", "=MAX((A1:A5,B1:B2))");
    model._set("E11", "=SUM((A1:A5 A2:C2,C5))");
    model._set("E12", "=ISREF((A1,B1))");
    model.evaluate();

    assert_eq!(model._get_text("E1"), "101");
    assert_eq!(model._get_text("E2"), "12");
    assert_eq!(model._get_text("E3"), "2");
    assert_eq!(model._get_text("E4"), "3");
    assert_eq!(model._get_text("E5"), "1");
    assert_eq!(model._get_text("E6"), "6");
    assert_eq!(model._get_text("E7"), "200");
    assert_eq!(model._get_text("E8"), "#VALUE!");
    assert_eq!(model._get_text("E9"), "6");
    assert_eq!(model._get_text("E10"), "20");
    assert_eq!(model._get_text("E11"), "502");
    assert_eq!(model._get_text("E12"), "TRUE");

    assert_eq!(model._get_formula("E1"), "=SUM((A1,C1))");
    assert_eq!(model._get_formula("E11"), "=SUM((A1:A5 A2:C2,C5))");
}
