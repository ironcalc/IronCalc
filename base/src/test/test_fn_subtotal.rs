#![allow(clippy::unwrap_used)]

use crate::model::DefinedNameEntry;
use crate::test::util::new_empty_model;

#[test]
fn var_leaves_out_hidden_rows() {
    let mut model = new_empty_model();
    model._set("A1", "2");
    model._set("A2", "8");
    model._set("A3", "4");
    model.set_row_hidden(0, 2, true).unwrap();
    model._set("B1", "=SUBTOTAL(10,A1:A3)");
    model._set("B2", "=SUBTOTAL(11,A1:A3)");
    model._set("B3", "=SUBTOTAL(110,A1:A3)");
    model._set("B4", "=SUBTOTAL(111,A1:A3)");
    model.evaluate();
    // 10 and 11 take the three numbers
    assert_eq!(model._get_text("B1"), "9.333333333");
    assert_eq!(model._get_text("B2"), "6.222222222");
    // 110 and 111 only 2 and 4
    assert_eq!(model._get_text("B3"), "2");
    assert_eq!(model._get_text("B4"), "1");
}

#[test]
fn product_of_no_numbers() {
    let mut model = new_empty_model();
    model._set("A1", "2");
    model._set("A2", "8");
    // B1:B2 holds no number
    model._set("B1", "x");
    model._set("C1", "=SUBTOTAL(6,A1:A2)");
    model._set("C2", "=SUBTOTAL(6,B1:B2)");
    model._set("C3", "=SUBTOTAL(106,B1:B2)");
    model._set("C4", "=PRODUCT(B1:B2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "16");
    assert_eq!(model._get_text("C2"), "0");
    assert_eq!(model._get_text("C3"), "0");
    assert_eq!(model._get_text("C4"), "0");
}

#[test]
fn a_total_inside_another_formula_is_a_nested_total() {
    // Each column holds 1, 2 and 3 and, in row 4, the formula of the case.
    // Row 6 is a SUBTOTAL over rows 1 to 4: 6 when the cell of row 4 is left out.
    // Row 7 counts the cells that are not empty: 3 when it is left out, 4 when not.
    let cases = [
        // (the formula of row 4, with # for the column; its value; the SUBTOTAL)
        // These are as Excel gives them
        ("=SUBTOTAL(9,#1:#3)", "6", "6"),
        ("=SUBTOTAL(9,#1:#3)+1", "7", "6"),
        ("=SUBTOTAL(9,#1:#3)*2", "12", "6"),
        ("=SUBTOTAL(9,#1:#3)&\"\"", "6", "6"),
        ("=SUBTOTAL(9,#1:#3)>0", "TRUE", "6"),
        ("=-SUBTOTAL(9,#1:#3)", "-6", "6"),
        ("=IFERROR(SUBTOTAL(9,#1:#3),0)", "6", "6"),
        ("=SUM(SUBTOTAL(9,#1:#3),1)", "7", "6"),
        ("=IF(TRUE,SUBTOTAL(9,#1:#3),0)", "6", "6"),
        // also when the SUBTOTAL is in the branch that is not taken
        ("=IF(FALSE,SUBTOTAL(9,#1:#3),10)", "10", "6"),
        ("=SUBTOTAL(9,#1:#2)+SUBTOTAL(9,#3)", "6", "6"),
        ("=LET(t,SUBTOTAL(9,#1:#3),t)", "6", "6"),
        ("=LAMBDA(x,SUBTOTAL(9,x))(#1:#3)", "6", "6"),
        // not through a defined name, and not through a reference
        ("=NestedTotal", "6", "12"),
        ("=A4", "6", "12"),
        ("=SUM(#1:#3)+1", "7", "13"),
        ("=1+SUBTOTAL(9,#1:#3)", "7", "6"),
        ("=SUBTOTAL(9,#1:#3)^2", "36", "6"),
        ("=IF(TRUE,IFERROR(-SUBTOTAL(9,#1:#3),0),0)", "-6", "6"),
        ("=LAMBDA(x,x*2)(SUBTOTAL(9,#1:#3))", "12", "6"),
        // These follow from the same rule. Excel was not asked
        ("=@SUBTOTAL(9,#1:#3)", "6", "6"),
        // as the argument of a LAMBDA that has a name
        ("=Twice(SUBTOTAL(9,#1:#3))", "12", "6"),
    ];
    let mut model = new_empty_model();
    model
        .new_defined_name(DefinedNameEntry::new(
            "NestedTotal",
            "SUBTOTAL(9,Sheet1!$A$1:$A$3)",
        ))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Twice", "LAMBDA(x,x*2)"))
        .unwrap();
    let columns = [
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R",
        "S", "T", "U", "V",
    ];
    assert_eq!(columns.len(), cases.len());
    for (column, (formula, _, _)) in columns.iter().zip(cases) {
        model._set(&format!("{column}1"), "1");
        model._set(&format!("{column}2"), "2");
        model._set(&format!("{column}3"), "3");
        model._set(&format!("{column}4"), &formula.replace('#', column));
        model._set(
            &format!("{column}6"),
            &format!("=SUBTOTAL(9,{column}1:{column}4)"),
        );
        model._set(
            &format!("{column}7"),
            &format!("=SUBTOTAL(3,{column}1:{column}4)"),
        );
    }
    // COUNT reads the cells with a loop of its own
    model._set("X1", "=SUBTOTAL(2,B1:B4)");
    model.evaluate();

    for (column, (formula, value, total)) in columns.iter().zip(cases) {
        assert_eq!(model._get_text(&format!("{column}4")), value, "{formula}");
        assert_eq!(model._get_text(&format!("{column}6")), total, "{formula}");
        let count = if total == "6" { "3" } else { "4" };
        assert_eq!(model._get_text(&format!("{column}7")), count, "{formula}");
    }
    assert_eq!(model._get_text("X1"), "3");
}

#[test]
fn the_cells_that_a_total_spills_into_are_left_out() {
    // The values are those of Excel
    let mut model = new_empty_model();
    model._set("G1", "1");
    model._set("G2", "2");
    model._set("G3", "3");
    // 6 in H1, H2 and H3
    model._set("H1", "=SEQUENCE(3)*0+SUBTOTAL(9,G1:G3)");
    model._set("I1", "=SUBTOTAL(9,H1:H3)");
    model._set("I2", "=SUBTOTAL(3,H1:H3)");
    model._set("I3", "=SUBTOTAL(9,H2:H3)");
    // a spill without a SUBTOTAL is counted
    model._set("K1", "=SEQUENCE(3)*0+SUM(G1:G3)");
    model._set("L1", "=SUBTOTAL(9,K1:K3)");
    // A1 is evaluated before the formula of H1
    model._set("A1", "=SUBTOTAL(9,H1:H3)");
    model.evaluate();

    assert_eq!(model._get_text("H3"), "6");
    assert_eq!(model._get_text("I1"), "0");
    assert_eq!(model._get_text("I2"), "0");
    assert_eq!(model._get_text("I3"), "0");
    assert_eq!(model._get_text("L1"), "18");
    assert_eq!(model._get_text("A1"), "0");
}
