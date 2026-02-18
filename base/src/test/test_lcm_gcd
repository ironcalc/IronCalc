#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

fn create_formula_with_args(func_name: &str, count: usize) -> String {
    let args = (0..count).map(|_| "1".to_string()).collect::<Vec<_>>().join(", ");
    format!("={}({})", func_name, args)
}

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=LCM()");
    model._set("A2", "=LCM(2, 3)");

    model._set("A3", "=GCD()");
    model._set("A4", "=GCD(10, 25)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"6");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"5");
}

#[test]
fn test_with_50_arguments() {
    let mut model = new_empty_model();
    model._set("A1", &create_formula_with_args("LCM", 50));
    model._set("A2", &create_formula_with_args("GCD", 50));


    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"1");
}
