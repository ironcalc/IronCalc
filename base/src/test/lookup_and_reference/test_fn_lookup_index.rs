#![allow(clippy::unwrap_used)]

//! Lookups answered from the per-pass exact-match index, and sorted lookups
//! that read only the cells they probe, must give the same results as a scan.

use crate::test::util::new_empty_model;

fn table(model: &mut crate::Model<'_>) {
    // A2:B8, keys in A, values in B. Duplicate key 10 and mixed case text.
    let rows = [
        ("10", "first ten"),
        ("Łódź", "city"),
        ("25", "twenty five"),
        ("10", "second ten"),
        ("apple", "fruit"),
        ("1", "one"),
        ("0.5", "half"),
    ];
    for (i, (key, value)) in rows.iter().enumerate() {
        let row = i + 2;
        model._set(&format!("A{row}"), key);
        model._set(&format!("B{row}"), value);
    }
}

#[test]
fn vlookup_exact_many_formulas_same_table() {
    let mut model = new_empty_model();
    table(&mut model);
    model._set("D1", "=VLOOKUP(10,$A$2:$B$8,2,FALSE)");
    model._set("D2", "=VLOOKUP(25,$A$2:$B$8,2,FALSE)");
    model._set("D3", r#"=VLOOKUP("łÓDŹ",$A$2:$B$8,2,FALSE)"#);
    model._set("D4", "=VLOOKUP(99,$A$2:$B$8,2,FALSE)");
    model._set("D5", r#"=VLOOKUP("APPLE",$A$2:$B$8,2,FALSE)"#);
    model.evaluate();
    assert_eq!(model._get_text("D1"), "first ten");
    assert_eq!(model._get_text("D2"), "twenty five");
    assert_eq!(model._get_text("D3"), "city");
    assert_eq!(model._get_text("D4"), "#N/A");
    assert_eq!(model._get_text("D5"), "fruit");
}

#[test]
fn vlookup_exact_values_left_to_the_scan() {
    let mut model = new_empty_model();
    table(&mut model);
    model._set("D1", "=VLOOKUP(1,$A$2:$B$8,2,FALSE)");
    model._set("D2", "=VLOOKUP(0.5,$A$2:$B$8,2,FALSE)");
    model._set("D3", r#"=VLOOKUP("ap*",$A$2:$B$8,2,FALSE)"#);
    model._set("D4", r#"=VLOOKUP("?ódź",$A$2:$B$8,2,FALSE)"#);
    model.evaluate();
    assert_eq!(model._get_text("D1"), "one");
    assert_eq!(model._get_text("D2"), "half");
    assert_eq!(model._get_text("D3"), "fruit");
    assert_eq!(model._get_text("D4"), "city");
}

#[test]
fn vlookup_whole_columns_beyond_one_chunk() {
    let mut model = new_empty_model();
    for row in 1..=2100 {
        model._set(&format!("A{row}"), &format!("{}", row * 2));
        model._set(&format!("B{row}"), &format!("v{row}"));
    }
    model._set("D1", "=VLOOKUP(4000,A:B,2,FALSE)");
    model._set("D3", "=VLOOKUP(4200,A:B,2,FALSE)");
    model._set("D4", "=VLOOKUP(9999,A:B,2,FALSE)");
    model._set("D5", "=MATCH(3000,A:A,0)");
    model._set("D6", "=XLOOKUP(2048,A:A,B:B)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "v2000");
    assert_eq!(model._get_text("D3"), "v2100");
    assert_eq!(model._get_text("D4"), "#N/A");
    assert_eq!(model._get_text("D5"), "1500");
    assert_eq!(model._get_text("D6"), "v1024");
}

#[test]
fn vlookup_sorted_lazy_same_as_before() {
    let mut model = new_empty_model();
    for (i, value) in [10, 20, 30, 40, 50].iter().enumerate() {
        model._set(&format!("A{}", i + 1), &value.to_string());
        model._set(&format!("B{}", i + 1), &format!("r{}", i + 1));
    }
    model._set("D1", "=VLOOKUP(35,A1:B5,2)");
    model._set("D2", "=VLOOKUP(10,A1:B5,2,TRUE)");
    model._set("D3", "=VLOOKUP(5,A1:B5,2,TRUE)");
    model._set("D4", "=VLOOKUP(99,A1:B5,2,TRUE)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "r3");
    assert_eq!(model._get_text("D2"), "r1");
    assert_eq!(model._get_text("D3"), "#N/A");
    assert_eq!(model._get_text("D4"), "r5");
}

#[test]
fn hlookup_exact_and_sorted() {
    let mut model = new_empty_model();
    for (i, value) in [10, 20, 30, 40].iter().enumerate() {
        let column = (b'A' + i as u8) as char;
        model._set(&format!("{column}1"), &value.to_string());
        model._set(&format!("{column}2"), &format!("c{}", i + 1));
    }
    model._set("A4", "=HLOOKUP(30,A1:D2,2,FALSE)");
    model._set("B4", "=HLOOKUP(25,A1:D2,2,TRUE)");
    model._set("C4", "=HLOOKUP(31,A1:D2,2,FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("A4"), "c3");
    assert_eq!(model._get_text("B4"), "c2");
    assert_eq!(model._get_text("C4"), "#N/A");
}

#[test]
fn match_exact_first_duplicate_and_tolerance() {
    let mut model = new_empty_model();
    table(&mut model);
    model._set("C1", "0.3");
    model._set("D1", "=MATCH(10,A2:A8,0)");
    model._set("D2", r#"=MATCH("APPLE",A2:A8,0)"#);
    model._set("D3", "=MATCH(0.1+0.2,C1:C1,0)");
    model._set("D4", "=MATCH(11,A2:A8,0)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "1");
    assert_eq!(model._get_text("D2"), "5");
    assert_eq!(model._get_text("D3"), "1");
    assert_eq!(model._get_text("D4"), "#N/A");
}

#[test]
fn xlookup_exact_first_and_last() {
    let mut model = new_empty_model();
    table(&mut model);
    model._set("D1", "=XLOOKUP(10,A2:A8,B2:B8)");
    model._set("D2", "=XLOOKUP(10,A2:A8,B2:B8,,0,-1)");
    model._set("D3", r#"=XLOOKUP("łódź",A2:A8,B2:B8)"#);
    model._set("D4", r#"=XLOOKUP(77,A2:A8,B2:B8,"none")"#);
    model.evaluate();
    assert_eq!(model._get_text("D1"), "first ten");
    assert_eq!(model._get_text("D2"), "second ten");
    assert_eq!(model._get_text("D3"), "city");
    assert_eq!(model._get_text("D4"), "none");
}

#[test]
fn index_rebuilt_after_a_change() {
    let mut model = new_empty_model();
    table(&mut model);
    model._set("D1", "=VLOOKUP(25,$A$2:$B$8,2,FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "twenty five");
    model._set("A4", "26");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "#N/A");
    model._set("A6", "25");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "fruit");
}

#[test]
fn lookup_range_of_formulas_and_spills() {
    let mut model = new_empty_model();
    model._set("A1", "=10*2");
    model._set("A2", "=SEQUENCE(3,1,100,10)");
    model._set("B1", "x");
    model._set("B2", "a");
    model._set("B3", "b");
    model._set("B4", "c");
    model._set("D1", "=VLOOKUP(20,A1:B4,2,FALSE)");
    model._set("D2", "=VLOOKUP(110,A1:B4,2,FALSE)");
    model._set("D3", "=MATCH(120,A1:A4,0)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "x");
    assert_eq!(model._get_text("D2"), "b");
    assert_eq!(model._get_text("D3"), "4");
}
