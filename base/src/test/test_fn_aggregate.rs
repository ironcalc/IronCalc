#![allow(clippy::unwrap_used)]

use crate::model::DefinedNameEntry;
use crate::test::util::new_empty_model;

#[test]
fn functions_1_to_11() {
    let mut model = new_empty_model();
    // three numbers, an error, a text and an empty cell
    model._set("A1", "3");
    model._set("A2", "5");
    model._set("A3", "=1/0");
    model._set("A4", "x");
    model._set("A6", "10");
    // function_num, with option 6 (errors are left out), with option 4 (nothing is)
    let cases = [
        (1, "6", "#DIV/0!"),
        (2, "3", "3"),
        (3, "4", "5"),
        (4, "10", "#DIV/0!"),
        (5, "3", "#DIV/0!"),
        (6, "150", "#DIV/0!"),
        (7, "3.605551275", "#DIV/0!"),
        (8, "2.943920289", "#DIV/0!"),
        (9, "18", "#DIV/0!"),
        (10, "13", "#DIV/0!"),
        (11, "8.666666667", "#DIV/0!"),
    ];
    for (function, _, _) in cases {
        model._set(
            &format!("B{function}"),
            &format!("=AGGREGATE({function},6,A1:A6)"),
        );
        model._set(
            &format!("C{function}"),
            &format!("=AGGREGATE({function},4,A1:A6)"),
        );
    }
    model.evaluate();
    for (function, without_errors, with_errors) in cases {
        assert_eq!(
            model._get_text(&format!("B{function}")),
            without_errors,
            "function {function}, option 6"
        );
        assert_eq!(
            model._get_text(&format!("C{function}")),
            with_errors,
            "function {function}, option 4"
        );
    }
    assert_eq!(model._get_formula("B1"), "=AGGREGATE(1,6,A1:A6)");
}

// The example in Excel's documentation of AGGREGATE, with its results
#[test]
fn example_in_the_excel_documentation() {
    let mut model = new_empty_model();
    let column_a = [
        "=1/0",
        "72",
        "30",
        "=SQRT(-1)",
        "31",
        "96",
        "32",
        "81",
        "33",
        "53",
        "34",
    ];
    let column_b = [
        "82", "65", "95", "63", "53", "71", "55", "83", "100", "91", "89",
    ];
    for (i, (a, b)) in column_a.iter().zip(column_b).enumerate() {
        model._set(&format!("A{}", i + 1), a);
        model._set(&format!("B{}", i + 1), b);
    }
    model._set("C1", "=AGGREGATE(4,6,A1:A11)");
    model._set("C2", "=MAX(A1:A2)");
    model.evaluate();
    assert_eq!(model._get_text("A4"), "#NUM!");
    assert_eq!(model._get_text("C1"), "96");
    assert_eq!(model._get_text("C2"), "#DIV/0!");
}

#[test]
fn hidden_rows_and_nested_totals() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "20");
    model._set("A3", "=SUBTOTAL(9,A1:A2)");
    model._set("A4", "=AGGREGATE(9,4,A1:A2)");
    model._set("A5", "300");
    model.set_row_hidden(0, 2, true).unwrap();
    // options 0 to 3 leave out the totals in A3 and A4, the odd ones leave out
    // the hidden row. SUM, COUNT and COUNTA each have their own loop over the cells
    let cases = [
        (
            9,
            "B",
            ["321", "301", "321", "301", "363", "343", "363", "343"],
        ),
        (2, "C", ["3", "2", "3", "2", "5", "4", "5", "4"]),
        (3, "D", ["3", "2", "3", "2", "5", "4", "5", "4"]),
    ];
    for (function, column, _) in cases {
        for options in 0..8 {
            model._set(
                &format!("{column}{}", options + 1),
                &format!("=AGGREGATE({function},{options},A1:A5)"),
            );
        }
    }
    // no option is option 0
    model._set("E1", "=AGGREGATE(9,,A1:A5)");
    // SUBTOTAL leaves out the AGGREGATE in A4 as well
    model._set("E2", "=SUBTOTAL(9,A1:A5)");
    model.evaluate();
    for (function, column, expected) in cases {
        for (options, value) in expected.iter().enumerate() {
            assert_eq!(
                model._get_text(&format!("{column}{}", options + 1)),
                *value,
                "function {function}, option {options}"
            );
        }
    }
    assert_eq!(model._get_text("E1"), "321");
    assert_eq!(model._get_text("E2"), "321");
}

#[test]
fn wrong_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "3");
    model._set("A2", "5");
    model.set_row_hidden(0, 2, true).unwrap();
    model._set("B1", "=AGGREGATE(9,6)");
    model._set("B2", "=AGGREGATE(0,6,A1:A2)");
    model._set("B3", "=AGGREGATE(20,6,A1:A2)");
    model._set("B4", "=AGGREGATE(9,8,A1:A2)");
    model._set("B5", "=AGGREGATE(9,-1,A1:A2)");
    // MEDIAN: not yet
    model._set("B6", "=AGGREGATE(12,6,A1:A2)");
    // the two numbers are truncated: option 7 would leave out the hidden row
    model._set("B7", "=AGGREGATE(9.9,6.9,A1:A2)");
    // What is not a reference is #VALUE!, as in Excel. SUM, COUNT and COUNTA each read
    // their arguments with their own loop
    let not_references = [
        "=AGGREGATE(9,6,5)",
        "=AGGREGATE(2,6,5)",
        "=AGGREGATE(3,6,5)",
        "=AGGREGATE(9,6,A1*2)",
        "=AGGREGATE(9,6,A1:A2,5)",
        "=AGGREGATE(9,6,A1:A2*2)",
        "=AGGREGATE(2,6,A1:A2*2)",
        "=AGGREGATE(3,6,A1:A2,A1:A2*2)",
        "=AGGREGATE(9,6,\"5\")",
        "=AGGREGATE(9,6,TRUE)",
        "=AGGREGATE(3,6,\"x\")",
        "=AGGREGATE(9,6,A1:A2,)",
        // a value from a function that could give a cell
        "=AGGREGATE(9,6,IF(TRUE,5))",
        // the condition of IF is a cell, its result is not (Excel was not asked about
        // this one)
        "=AGGREGATE(9,6,IF(A1,5))",
        "=AGGREGATE(9,6,CHOOSE(1,5))",
        "=AGGREGATE(9,6,INDEX({1,2,3},2))",
        // what CHOOSE is asked with is a cell, its result is not (Excel was not asked
        // about this one)
        "=AGGREGATE(9,6,CHOOSE(A1,7,8,9))",
        // an array from such a function
        "=AGGREGATE(2,6,IF(A1:A2>2,A1:A2))",
        "=AGGREGATE(9,6,INDEX(A1:A2*2,0))",
        // a total written as an argument
        "=AGGREGATE(9,6,SUBTOTAL(9,A1:A2))",
        "=AGGREGATE(2,6,SUBTOTAL(9,A1:A2))",
        "=AGGREGATE(3,6,SUBTOTAL(9,A1:A2))",
        "=AGGREGATE(9,6,AGGREGATE(9,4,A1:A2))",
    ];
    for (row, formula) in not_references.iter().enumerate() {
        model._set(&format!("D{}", row + 1), formula);
    }
    // An argument that is an error gives that error, whatever the option, as in Excel
    let errors = [
        ("=AGGREGATE(9,6,#REF!)", "#REF!"),
        ("=AGGREGATE(2,6,#REF!)", "#REF!"),
        ("=AGGREGATE(3,6,#REF!)", "#REF!"),
        ("=AGGREGATE(9,6,1/0)", "#DIV/0!"),
        ("=AGGREGATE(2,6,1/0)", "#DIV/0!"),
        ("=AGGREGATE(3,6,1/0)", "#DIV/0!"),
    ];
    for (row, (formula, _)) in errors.iter().enumerate() {
        model._set(&format!("E{}", row + 1), formula);
    }
    // SUBTOTAL still takes a number
    model._set("F1", "=SUBTOTAL(9,A1:A2,5)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#ERROR!");
    assert_eq!(model._get_text("B2"), "#VALUE!");
    assert_eq!(model._get_text("B3"), "#VALUE!");
    assert_eq!(model._get_text("B4"), "#VALUE!");
    assert_eq!(model._get_text("B5"), "#VALUE!");
    assert_eq!(model._get_text("B6"), "#N/IMPL!");
    assert_eq!(model._get_text("B7"), "8");
    for (row, formula) in not_references.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("D{}", row + 1)),
            "#VALUE!",
            "{formula}"
        );
    }
    for (row, (formula, error)) in errors.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("E{}", row + 1)),
            *error,
            "{formula}"
        );
    }
    assert_eq!(model._get_text("F1"), "13");
}

#[test]
fn what_gives_a_reference_is_a_reference() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "3");
    model._set("B3", "5");
    model._set("B4", "8");
    model._set("G1", "=1/0");
    model._set("G2", "4");
    // a spill of three cells that hold 17, the total of B1:B4
    model._set("H1", "=SEQUENCE(3)*0+SUBTOTAL(9,B1:B4)");
    model
        .new_defined_name(DefinedNameEntry::new(
            "OneCell",
            "INDEX(Sheet1!$B$1:$B$4,2)",
        ))
        .unwrap();
    // The values are those of Excel
    let cases = [
        ("=AGGREGATE(9,6,IF(TRUE,B1:B4))", "17"),
        ("=AGGREGATE(9,6,OFFSET(B1,0,0,4,1))", "17"),
        ("=AGGREGATE(9,6,INDEX(B1:B4,0))", "17"),
        ("=AGGREGATE(9,6,(B1:B4))", "17"),
        ("=AGGREGATE(9,6,CHOOSE(1,B1:B4))", "17"),
        // one cell
        ("=AGGREGATE(9,6,INDEX(B1:B4,2))", "3"),
        ("=AGGREGATE(2,6,INDEX(B1:B4,2))", "1"),
        ("=AGGREGATE(3,6,INDEX(B1:B4,2))", "1"),
        ("=AGGREGATE(9,6,IF(TRUE,B2))", "3"),
        ("=AGGREGATE(9,6,CHOOSE(1,B2))", "3"),
        ("=AGGREGATE(9,6,XLOOKUP(3,B1:B4,B1:B4))", "3"),
        ("=AGGREGATE(9,6,CHOOSE(1,INDEX(B1:B4,2)))", "3"),
        ("=AGGREGATE(9,6,OneCell)", "3"),
        // an empty cell is not a number (C1:C4 is empty)
        ("=AGGREGATE(1,6,B1:B4,INDEX(C1:C4,1))", "4.25"),
        ("=AGGREGATE(5,6,B1:B4,INDEX(C1:C4,1))", "1"),
        ("=AGGREGATE(1,6,INDEX(C1:C4,1))", "#DIV/0!"),
        // an error in that one cell, when the option keeps errors
        ("=AGGREGATE(9,4,INDEX(G1:G2,1))", "#DIV/0!"),
        // Excel was not asked about the rest.
        // The cells that a total spills into are nested totals, as the cell of its
        // formula. That is Excel's rule, with other numbers
        ("=AGGREGATE(9,0,H1:H3)", "0"),
        ("=AGGREGATE(9,4,H1:H3)", "51"),
        // What a function picks from says if it can give a cell: the branches of IF,
        // the choices of CHOOSE, what OFFSET and INDIRECT give
        ("=AGGREGATE(9,6,IF(B1<0,5,B2))", "3"),
        ("=AGGREGATE(9,6,CHOOSE(2,5,B2))", "3"),
        ("=AGGREGATE(9,6,CHOOSE(2,5,INDEX(B1:B4,2)))", "3"),
        ("=AGGREGATE(9,6,INDEX(OFFSET(B1,0,0,4,1),2))", "3"),
        ("=AGGREGATE(9,6,INDEX(INDIRECT(\"B1:B4\"),2))", "3"),
        // An error of the function itself is the result, whatever the option
        ("=AGGREGATE(9,6,XLOOKUP(99,B1:B4,B1:B4))", "#N/A"),
        ("=AGGREGATE(9,6,INDEX(B1:B4,9))", "#REF!"),
        ("=AGGREGATE(2,6,INDEX(B1:B4,9))", "#REF!"),
        ("=AGGREGATE(3,6,XLOOKUP(99,B1:B4,B1:B4))", "#N/A"),
    ];
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("D{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, value)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("D{}", row + 1)),
            *value,
            "{formula}"
        );
    }
}

#[test]
fn a_total_inside_another_formula_is_a_nested_total() {
    // Row 4 holds a total inside another formula. The values are those of Excel
    let mut model = new_empty_model();
    for column in ["A", "B"] {
        model._set(&format!("{column}1"), "1");
        model._set(&format!("{column}2"), "2");
        model._set(&format!("{column}3"), "3");
    }
    model._set("A4", "=AGGREGATE(9,4,A1:A3)+1");
    model._set("B4", "=IF(FALSE,SUBTOTAL(9,B1:B3),10)");
    for (row, column) in [(1, "A"), (2, "B")] {
        model._set(
            &format!("D{row}"),
            &format!("=SUBTOTAL(9,{column}1:{column}4)"),
        );
        model._set(
            &format!("E{row}"),
            &format!("=AGGREGATE(9,0,{column}1:{column}4)"),
        );
        model._set(
            &format!("F{row}"),
            &format!("=AGGREGATE(9,4,{column}1:{column}4)"),
        );
    }
    model.evaluate();
    // SUBTOTAL and option 0 leave the cell out, option 4 does not
    assert_eq!(model._get_text("D1"), "6");
    assert_eq!(model._get_text("E1"), "6");
    assert_eq!(model._get_text("F1"), "13");
    assert_eq!(model._get_text("D2"), "6");
    assert_eq!(model._get_text("E2"), "6");
    assert_eq!(model._get_text("F2"), "16");
}
