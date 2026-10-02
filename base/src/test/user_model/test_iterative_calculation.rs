#![allow(clippy::unwrap_used)]

use crate::test::user_model::util::new_empty_user_model;
use crate::types::IterativeCalculation;
use crate::UserModel;

fn value(model: &UserModel, row: i32, column: i32) -> f64 {
    model
        .get_formatted_cell_value(0, row, column)
        .unwrap()
        .parse()
        .unwrap()
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-6,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn set_and_get() {
    let mut model = new_empty_user_model();
    assert_eq!(
        model.get_iterative_calculation(),
        IterativeCalculation::default()
    );
    model.set_iterative_calculation(true, 30, 0.01).unwrap();
    assert_eq!(
        model.get_iterative_calculation(),
        IterativeCalculation {
            enabled: true,
            maximum_iterations: 30,
            maximum_change: 0.01,
        }
    );
}

#[test]
fn undo_and_redo() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "=B1/2+10").unwrap();
    model.set_user_input(0, 1, 2, "=A1+5").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "#CIRC!");

    model.set_iterative_calculation(true, 100, 1e-9).unwrap();
    assert_close(value(&model, 1, 1), 25.0);
    assert_close(value(&model, 1, 2), 30.0);

    model.undo().unwrap();
    assert_eq!(
        model.get_iterative_calculation(),
        IterativeCalculation::default()
    );
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "#CIRC!");

    model.redo().unwrap();
    assert_eq!(
        model.get_iterative_calculation(),
        IterativeCalculation {
            enabled: true,
            maximum_iterations: 100,
            maximum_change: 1e-9,
        }
    );
    assert_close(value(&model, 1, 1), 25.0);
}

#[test]
fn survives_a_round_trip_through_bytes() {
    let mut model = new_empty_user_model();
    model.set_iterative_calculation(true, 12, 0.25).unwrap();
    let bytes = model.to_bytes();
    let restored = UserModel::from_bytes(&bytes, "en").unwrap();
    assert_eq!(
        restored.get_iterative_calculation(),
        IterativeCalculation {
            enabled: true,
            maximum_iterations: 12,
            maximum_change: 0.25,
        }
    );
}
