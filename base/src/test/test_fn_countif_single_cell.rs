#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Excel treats every reference handed to the range parameters of the *IF/*IFS
// family as a range, including a reference to a single cell: COUNTIF(A1,"x") is
// a count over the 1x1 range A1 and answers 0 or 1, never #VALUE!.

#[test]
fn test_fn_countif_single_cell_reference() {
    let mut model = new_empty_model();
    model.new_sheet();

    model._set("A1", "x");
    // B1 is left empty
    model._set("Sheet2!A1", "y");

    model._set("D1", "=COUNTIF(A1,\"x\")");
    model._set("D2", "=COUNTIF(B1,\"x\")");
    model._set("D3", "=COUNTIF($A$1,\"x\")");
    model._set("D4", "=COUNTIF(Sheet1!A1,\"x\")");
    model._set("D5", "=COUNTIF(Sheet2!A1,\"x\")");
    model._set("D6", "=COUNTIF(Sheet2!A1,\"y\")");

    model.evaluate();

    assert_eq!(model._get_text("D1"), *"1");
    assert_eq!(model._get_text("D2"), *"0");
    assert_eq!(model._get_text("D3"), *"1");
    assert_eq!(model._get_text("D4"), *"1");
    assert_eq!(model._get_text("D5"), *"0");
    assert_eq!(model._get_text("D6"), *"1");
}

// A cell holding an error is counted when the criterion asks for that error and
// skipped otherwise.
#[test]
fn test_fn_countif_single_cell_with_an_error() {
    let mut model = new_empty_model();

    model._set("A1", "=1/0");

    model._set("D1", "=COUNTIF(A1,\"x\")");
    model._set("D2", "=COUNTIF(A1,\"#DIV/0!\")");

    model.evaluate();

    assert_eq!(model._get_text("D1"), *"0");
    assert_eq!(model._get_text("D2"), *"1");
}

#[test]
fn test_fn_countif_single_cell_reference_numeric_criteria() {
    let mut model = new_empty_model();

    model._set("A1", "42");

    model._set("D1", "=COUNTIF(A1,42)");
    model._set("D2", "=COUNTIF(A1,\">40\")");
    model._set("D3", "=COUNTIF(A1,\"<40\")");

    model.evaluate();

    assert_eq!(model._get_text("D1"), *"1");
    assert_eq!(model._get_text("D2"), *"1");
    assert_eq!(model._get_text("D3"), *"0");
}

#[test]
fn test_fn_countif_single_cell_defined_name() {
    let mut model = new_empty_model();

    model
        .new_defined_name("MyName", None, "Sheet1!$P$293")
        .unwrap();
    model
        .new_defined_name("EmptyCell", None, "Sheet1!$P$294")
        .unwrap();

    model._set("P293", "COVERED");

    model._set("A1", "=COUNTIF(MyName,\"COVERED\")");
    model._set("A2", "=COUNTIF(EmptyCell,\"COVERED\")");
    model._set("A3", "=IF(COUNTIF(MyName,\"COVERED\")>0,\"yes\",\"no\")");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"0");
    assert_eq!(model._get_text("A3"), *"yes");
}

#[test]
fn test_fn_countif_single_cell_let_variable_and_intersection() {
    let mut model = new_empty_model();

    model._set("A1", "x");
    model._set("A2", "y");
    model._set("A3", "x");

    model._set("D1", "=LET(r,A1,COUNTIF(r,\"x\"))");
    // @A1:A3 in row 2 is the single cell A2
    model._set("D2", "=COUNTIF(@A1:A3,\"y\")");

    model.evaluate();

    assert_eq!(model._get_text("D1"), *"1");
    assert_eq!(model._get_text("D2"), *"1");
}

// Multi-cell ranges were never affected and must answer the same.
#[test]
fn test_fn_countif_multi_cell_range_is_unchanged() {
    let mut model = new_empty_model();

    model._set("A1", "x");
    model._set("A2", "y");
    model._set("A3", "x");
    model
        .new_defined_name("Many", None, "Sheet1!$A$1:$A$3")
        .unwrap();

    model._set("D1", "=COUNTIF(A1:A3,\"x\")");
    model._set("D2", "=COUNTIF(A1:A3,\"z\")");
    model._set("D3", "=COUNTIF(A:A,\"x\")");
    model._set("D4", "=COUNTIF(Many,\"x\")");

    model.evaluate();

    assert_eq!(model._get_text("D1"), *"2");
    assert_eq!(model._get_text("D2"), *"0");
    assert_eq!(model._get_text("D3"), *"2");
    assert_eq!(model._get_text("D4"), *"2");
}

// A formula parsed at one cell and evaluated at another can give a range whose
// corners are swapped. It still covers every cell between them.
#[test]
fn test_fn_ifs_range_inverted_at_evaluation_shared_formula() {
    let mut model = new_empty_model();

    for row in 1..=10 {
        model._set(&format!("A{row}"), "x");
        model._set(&format!("B{row}"), "10");
    }

    // In D5 and E5 the ranges are A1:$A$3 and B1:$B$3
    model._set("D5", "=COUNTIF(A1:$A$3,\"x\")");
    model._set("E5", "=SUMIF(A1:$A$3,\"x\",B1:$B$3)");
    model._set("F5", "=COUNTIFS(A1:$A$3,\"x\")");

    // As in an xlsx shared formula, D10, E10 and F10 hold the same formulas
    // without parsing them again: there the ranges are A6:$A$3 and B6:$B$3.
    for column in 4..=6 {
        let index = model
            .workbook
            .worksheet(0)
            .unwrap()
            .cell(5, column)
            .unwrap()
            .get_formula()
            .unwrap();
        model
            .workbook
            .worksheet_mut(0)
            .unwrap()
            .set_cell_with_formula(10, column, index, 0)
            .unwrap();
    }

    model.evaluate();

    assert_eq!(model._get_text("D5"), *"3");
    assert_eq!(model._get_text("E5"), *"30");
    assert_eq!(model._get_text("F5"), *"3");
    assert_eq!(model._get_text("D10"), *"4");
    assert_eq!(model._get_text("E10"), *"40");
    assert_eq!(model._get_text("F10"), *"4");
}

#[test]
fn test_fn_ifs_range_inverted_at_evaluation_lambda() {
    let mut model = new_empty_model();

    model._set("A5", "x");
    model._set("B5", "5");
    // The body of a LAMBDA defined name is parsed at A1 and evaluated where
    // it is called: in H10 its range runs from $A10 up to $A$5.
    model
        .new_defined_name("CountX", None, "LAMBDA(n,COUNTIF($A$5:$A1,\"x\"))")
        .unwrap();
    model
        .new_defined_name("SumX", None, "LAMBDA(n,SUMIF($A$5:$A1,\"x\",$B$5:$B1))")
        .unwrap();

    model._set("H10", "=CountX(1)");
    model._set("H11", "=SumX(1)");

    model.evaluate();

    assert_eq!(model._get_text("H10"), *"1");
    assert_eq!(model._get_text("H11"), *"5");
}

// An argument that is not a reference is still an error.
#[test]
fn test_fn_countif_non_reference_range_is_still_an_error() {
    let mut model = new_empty_model();

    model._set("A1", "=COUNTIF(5,5)");
    model._set("A2", "=COUNTIF(\"x\",\"x\")");
    model._set("A3", "=COUNTIF(NoSuchName,\"x\")");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
    assert_eq!(model._get_text("A3"), *"#NAME?");
}

#[test]
fn test_fn_countifs_single_cell_reference() {
    let mut model = new_empty_model();

    model._set("A1", "x");
    model._set("B1", "10");

    model._set("D1", "=COUNTIFS(A1,\"x\")");
    model._set("D2", "=COUNTIFS(A1,\"x\",B1,\">5\")");
    model._set("D3", "=COUNTIFS(A1,\"x\",B1,\">50\")");
    model._set("D4", "=COUNTIFS(A1:A1,\"x\",B1,\">5\")");

    model.evaluate();

    assert_eq!(model._get_text("D1"), *"1");
    assert_eq!(model._get_text("D2"), *"1");
    assert_eq!(model._get_text("D3"), *"0");
    assert_eq!(model._get_text("D4"), *"1");
}

#[test]
fn test_fn_sumif_single_cell_reference() {
    let mut model = new_empty_model();

    model._set("A1", "x");
    model._set("B1", "10");

    // Two argument form: the criteria range is also the sum range
    model._set("D1", "=SUMIF(B1,\">5\")");
    model._set("D2", "=SUMIF(A1,\"x\",B1)");
    model._set("D3", "=SUMIF(A1,\"y\",B1)");
    // An array criterion spills one sum per criterion
    model._set("D4", "=SUMIF(A1,{\"x\",\"y\"},B1)");

    model.evaluate();

    assert_eq!(model._get_text("D1"), *"10");
    assert_eq!(model._get_text("D2"), *"10");
    assert_eq!(model._get_text("D3"), *"0");
    assert_eq!(model._get_text("D4"), *"10");
    assert_eq!(model._get_text("E4"), *"0");
}

#[test]
fn test_fn_ifs_family_single_cell_reference() {
    let mut model = new_empty_model();

    model._set("A1", "x");
    model._set("B1", "10");

    model._set("D1", "=SUMIFS(B1,A1,\"x\")");
    model._set("D2", "=AVERAGEIFS(B1,A1,\"x\")");
    model._set("D3", "=MAXIFS(B1,A1,\"x\")");
    model._set("D4", "=MINIFS(B1,A1,\"x\")");
    model._set("D5", "=AVERAGEIF(A1,\"x\",B1)");

    model.evaluate();

    assert_eq!(model._get_text("D1"), *"10");
    assert_eq!(model._get_text("D2"), *"10");
    assert_eq!(model._get_text("D3"), *"10");
    assert_eq!(model._get_text("D4"), *"10");
    assert_eq!(model._get_text("D5"), *"10");
}

#[test]
fn test_fn_sumif_single_cell_defined_names() {
    let mut model = new_empty_model();

    model
        .new_defined_name("Flag", None, "Sheet1!$P$293")
        .unwrap();
    model
        .new_defined_name("Amount", None, "Sheet1!$Q$293")
        .unwrap();

    model._set("P293", "COVERED");
    model._set("Q293", "125");

    model._set("A1", "=SUMIF(Flag,\"COVERED\",Amount)");
    model._set("A2", "=SUMIFS(Amount,Flag,\"COVERED\")");
    model._set("A3", "=SUMIFS(Amount,Flag,\"NOT COVERED\")");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"125");
    assert_eq!(model._get_text("A2"), *"125");
    assert_eq!(model._get_text("A3"), *"0");
}
