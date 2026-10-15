#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn adds_names_that_formulas_use() {
    let mut model = new_empty_model();
    model._set("A1", "5");
    model
        .new_defined_names(&[
            ("Base".to_string(), None, "Sheet1!$A$1".to_string()),
            ("Local".to_string(), Some(0), "Sheet1!$A$1:$A$2".to_string()),
            ("Add".to_string(), None, "=LAMBDA(x, y, x + y)".to_string()),
        ])
        .unwrap();
    model._set("B1", "=Base*10");
    model._set("C1", "=SUM(Local)");
    model._set("D1", "=Add(Base, 1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), *"50");
    assert_eq!(model._get_text("C1"), *"5");
    assert_eq!(model._get_text("D1"), *"6");
}

#[test]
fn names_added_after_formulas_resolve_on_the_next_evaluation() {
    let mut model = new_empty_model();
    model._set("A1", "7");
    model._set("B1", "=Late*2");
    model
        .new_defined_names(&[("Late".to_string(), None, "Sheet1!$A$1".to_string())])
        .unwrap();
    model.evaluate();
    assert_eq!(model._get_text("B1"), *"14");
}

#[test]
fn an_invalid_entry_adds_none_of_them() {
    let mut model = new_empty_model();
    for bad in [
        ("Twice".to_string(), None, "Sheet1!$A$2".to_string()),
        ("1bad".to_string(), None, "Sheet1!$A$1".to_string()),
        ("Scoped".to_string(), Some(9), "Sheet1!$A$1".to_string()),
        ("NotARef".to_string(), None, "1+1".to_string()),
    ] {
        let err = model
            .new_defined_names(&[
                ("Twice".to_string(), None, "Sheet1!$A$1".to_string()),
                bad.clone(),
            ])
            .unwrap_err();
        assert!(err.contains(&bad.0), "{err}");
        assert!(model.workbook.defined_names.is_empty(), "{bad:?}");
    }
}

#[test]
fn many_names_in_one_call() {
    let mut model = new_empty_model();
    let names: Vec<(String, Option<u32>, String)> = (1..=5000)
        .map(|i| (format!("Nm_{i}"), None, format!("Sheet1!$A${i}")))
        .collect();
    model.new_defined_names(&names).unwrap();
    assert_eq!(model.workbook.defined_names.len(), 5000);
    model._set("A5000", "3");
    model._set("B1", "=Nm_5000");
    model.evaluate();
    assert_eq!(model._get_text("B1"), *"3");
}
