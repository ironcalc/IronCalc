#![allow(clippy::unwrap_used, clippy::panic)]

use crate::cell::CellValue;
use crate::test::util::new_empty_model;
use crate::types::IterativeCalculation;
use crate::Model;

fn number(model: &Model, cell: &str) -> f64 {
    match model
        .get_cell_value_by_ref(&format!("Sheet1!{cell}"))
        .unwrap()
    {
        CellValue::Number(n) => n,
        other => panic!("expected a number in {cell}, got {other:?}"),
    }
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-6,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn defaults() {
    let model = new_empty_model();
    assert_eq!(
        model.get_iterative_calculation(),
        IterativeCalculation {
            enabled: false,
            maximum_iterations: 100,
            maximum_change: 0.001,
        }
    );
}

#[test]
fn set_and_get() {
    let mut model = new_empty_model();
    model.set_iterative_calculation(true, 30, 0.5);
    assert_eq!(
        model.get_iterative_calculation(),
        IterativeCalculation {
            enabled: true,
            maximum_iterations: 30,
            maximum_change: 0.5,
        }
    );
}

#[test]
fn circular_reference_is_an_error_when_disabled() {
    let mut model = new_empty_model();
    model._set("A1", "=B1/2+10");
    model._set("B1", "=A1+5");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#CIRC!");
    assert_eq!(model._get_text("B1"), "#CIRC!");
}

#[test]
fn disabling_brings_back_the_error() {
    let mut model = new_empty_model();
    model._set("A1", "=B1/2+10");
    model._set("B1", "=A1+5");
    model.set_iterative_calculation(true, 100, 1e-9);
    assert_close(number(&model, "A1"), 25.0);
    model.set_iterative_calculation(false, 100, 1e-9);
    assert_eq!(model._get_text("A1"), "#CIRC!");
    assert_eq!(model._get_text("B1"), "#CIRC!");
}

#[test]
fn two_cell_cycle_settles() {
    let mut model = new_empty_model();
    model._set("A1", "=B1/2+10");
    model._set("B1", "=A1+5");
    model.set_iterative_calculation(true, 100, 1e-9);
    assert_close(number(&model, "A1"), 25.0);
    assert_close(number(&model, "B1"), 30.0);
}

#[test]
fn self_reference_starts_from_zero() {
    let mut model = new_empty_model();
    model._set("A1", "=A1*0.5+10");
    model.set_iterative_calculation(true, 100, 1e-9);
    assert_close(number(&model, "A1"), 20.0);
}

#[test]
fn cells_reading_a_cycle_see_the_settled_values() {
    let mut model = new_empty_model();
    model._set("A1", "=B1/2+10");
    model._set("B1", "=A1+5");
    model._set("C1", "=A1*2");
    model._set("D1", "=C1+B1");
    model.set_iterative_calculation(true, 100, 1e-9);
    assert_close(number(&model, "C1"), 50.0);
    assert_close(number(&model, "D1"), 80.0);
}

#[test]
fn cycle_across_sheets() {
    let mut model = new_empty_model();
    model.add_sheet("Other").unwrap();
    model._set("Sheet1!A1", "=Other!A1/2+10");
    model._set("Other!A1", "=Sheet1!A1+5");
    model.set_iterative_calculation(true, 100, 1e-9);
    assert_close(number(&model, "A1"), 25.0);
    assert_close(
        match model.get_cell_value_by_ref("Other!A1").unwrap() {
            CellValue::Number(n) => n,
            other => panic!("expected a number, got {other:?}"),
        },
        30.0,
    );
}

#[test]
fn growing_reference_stops_at_the_maximum_iterations() {
    let mut model = new_empty_model();
    model._set("A1", "=A1+1");
    model.set_iterative_calculation(true, 100, 1e-9);
    assert_close(number(&model, "A1"), 100.0);
    model.set_iterative_calculation(true, 7, 1e-9);
    assert_close(number(&model, "A1"), 7.0);
}

#[test]
fn evaluating_again_gives_the_same_values() {
    let mut model = new_empty_model();
    model._set("A1", "=A1+1");
    model.set_iterative_calculation(true, 12, 1e-9);
    model.evaluate();
    model.evaluate();
    assert_close(number(&model, "A1"), 12.0);
}

#[test]
fn stops_once_the_change_is_below_the_maximum_change() {
    let mut model = new_empty_model();
    model._set("A1", "=A1*0.5+10");
    model.set_iterative_calculation(true, 100, 1.0);
    let value = number(&model, "A1");
    assert!(value < 20.0 && value > 18.0, "got {value}");
    model.set_iterative_calculation(true, 100, 1e-9);
    assert_close(number(&model, "A1"), 20.0);
}

#[test]
fn a_workbook_without_cycles_is_not_affected() {
    let mut model = new_empty_model();
    model._set("A1", "=5");
    model._set("B1", "=A1+1");
    model._set("C1", "=B1*2");
    model.set_iterative_calculation(true, 100, 1e-9);
    assert_close(number(&model, "A1"), 5.0);
    assert_close(number(&model, "B1"), 6.0);
    assert_close(number(&model, "C1"), 12.0);
}

#[test]
fn a_workbook_without_cycles_is_evaluated_once() {
    let mut model = new_empty_model();
    model._set("A1", "=RAND()");
    model._set("B1", "=A1");
    model.set_iterative_calculation(false, 100, 1e-9);
    let without = number(&model, "B1");
    assert!((0.0..1.0).contains(&without));
    model.set_iterative_calculation(true, 100, 1e-9);
    assert_close(number(&model, "B1"), number(&model, "A1"));
}

#[test]
fn settled_values_do_not_depend_on_the_order_of_entry() {
    let formulas = [
        ("A1", "=B1/2+10"),
        ("B1", "=C1+5"),
        ("C1", "=A1*0.25+B1*0.25"),
        ("D1", "=C1+A1"),
    ];
    let mut forward = new_empty_model();
    for (cell, formula) in formulas {
        forward._set(cell, formula);
    }
    forward.set_iterative_calculation(true, 200, 1e-12);
    let mut backward = new_empty_model();
    for (cell, formula) in formulas.iter().rev() {
        backward._set(cell, formula);
    }
    backward.set_iterative_calculation(true, 200, 1e-12);
    for (cell, _) in formulas {
        assert_close(number(&forward, cell), number(&backward, cell));
    }
}

#[test]
fn a_cycle_that_started_as_an_error_recovers_when_enabled() {
    let mut model = new_empty_model();
    model._set("A1", "=A1/2+4");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#CIRC!");
    model.set_iterative_calculation(true, 100, 1e-9);
    assert_close(number(&model, "A1"), 8.0);
}

#[test]
fn a_cell_outside_the_cycle_can_be_edited() {
    let mut model = new_empty_model();
    model._set("A1", "=B1/2+C1");
    model._set("B1", "=A1+5");
    model._set("C1", "10");
    model.set_iterative_calculation(true, 100, 1e-9);
    assert_close(number(&model, "A1"), 25.0);
    model._set("C1", "20");
    model.evaluate();
    assert_close(number(&model, "A1"), 45.0);
    assert_close(number(&model, "B1"), 50.0);
}
