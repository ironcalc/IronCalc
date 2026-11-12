#![allow(clippy::unwrap_used)]

//! Excel sets a formula whose last operation is a sum or a difference to 0
//! when it cancels out to within 2⁻⁵⁰ of its left operand. Expected values are
//! Excel 16's (the threshold measured on both sides of it).

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

fn number(model: &crate::model::Model, cell: &str) -> f64 {
    match model
        .get_cell_value_by_ref(&format!("Sheet1!{cell}"))
        .unwrap()
    {
        CellValue::Number(n) => n,
        other => unreachable!("{cell} is not a number: {other:?}"),
    }
}

#[test]
fn last_sum_of_the_formula() {
    let mut model = new_empty_model();
    model._set("A1", "1.5");
    let cases = [
        ("=0.1+0.2-0.3", 0.0),
        ("=1-0.9-0.1", 0.0),
        ("=1.1*3-3.3", 0.0),
        ("=SUM(0.1,0.2)-0.3", 0.0),
        ("=A1-1.4-0.1", 0.0),
        ("=(0.1+0.2)-0.3", 0.0),
        ("=0.1+(0.2-0.3)", 0.0),
        ("=+0.1+0.2-0.3", 0.0),
        ("=1E+15+0.3-1E+15", 0.0),
        ("=2^-50+1-1", 0.0),
        // Microsoft's example (KB 78113, "Example when a value reaches zero")
        ("=1.333+1.225-1.333-1.225", 0.0),
        // not the last operation, or not cancelling
        ("=1*(0.5-0.4-0.1)", -2.7755575615628914e-17),
        ("=-(0.1+0.2-0.3)", -5.551115123125783e-17),
        ("=0.1+0.2-0.3+0", 5.551115123125783e-17),
        ("=IF(TRUE,0.1+0.2-0.3)", 5.551115123125783e-17),
        ("=LET(x,0.1+0.2,x-0.3)", 5.551115123125783e-17),
        ("=1-1+2^-50", 8.881784197001252e-16),
        ("=1-(1-2^-30)", 9.313225746154785e-10),
        // the threshold: 2^-50 of the left operand
        ("=1-PRODUCT(1,1-POWER(2,-50))", 0.0),
        ("=1-PRODUCT(1,1-POWER(2,-49))", 1.7763568394002505e-15),
        ("=PRODUCT(-1,1-POWER(2,-50))+1", 8.881784197001252e-16),
        ("=1000-PRODUCT(1000,1-POWER(2,-50))", 9.094947017729282e-13),
        ("=1000-PRODUCT(1000,1-POWER(2,-51))", 0.0),
    ];
    for (i, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("C{}", i + 1), formula);
    }
    model.evaluate();
    for (i, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            number(&model, &format!("C{}", i + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn element_by_element_in_arrays() {
    let mut model = new_empty_model();
    model._set("A1", "={0.1,0.5}+0.2-0.3");
    model.evaluate();
    assert_eq!(number(&model, "A1"), 0.0);
    assert_eq!(number(&model, "B1"), 0.39999999999999997);
}
