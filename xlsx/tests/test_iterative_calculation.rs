#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use std::io::Cursor;

use ironcalc::export::save_xlsx_to_writer;
use ironcalc::import::{load_from_xlsx, load_from_xlsx_bytes};
use ironcalc_base::cell::CellValue;
use ironcalc_base::types::IterativeCalculation;
use ironcalc_base::Model;

fn round_trip(model: &Model) -> Model<'static> {
    let cursor = save_xlsx_to_writer(model, Cursor::new(Vec::new())).unwrap();
    let workbook = load_from_xlsx_bytes(&cursor.into_inner(), "model", "en", "UTC").unwrap();
    Model::from_workbook(workbook, "en").unwrap()
}

fn number(model: &Model, cell: &str) -> f64 {
    match model.get_cell_value_by_ref(cell).unwrap() {
        CellValue::Number(n) => n,
        other => panic!("expected a number in {cell}, got {other:?}"),
    }
}

#[test]
fn settings_survive_saving_and_loading() {
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    model.set_iterative_calculation(true, 50, 0.0001);
    let loaded = round_trip(&model);
    assert_eq!(
        loaded.get_iterative_calculation(),
        IterativeCalculation {
            enabled: true,
            maximum_iterations: 50,
            maximum_change: 0.0001,
        }
    );
}

#[test]
fn default_settings_survive_saving_and_loading() {
    let model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    let loaded = round_trip(&model);
    assert_eq!(
        loaded.get_iterative_calculation(),
        IterativeCalculation::default()
    );
}

#[test]
fn a_file_without_iteration_settings_loads_with_the_defaults() {
    let model = load_from_xlsx("tests/example.xlsx", "en", "UTC", "en").unwrap();
    assert_eq!(
        model.get_iterative_calculation(),
        IterativeCalculation::default()
    );
}

#[test]
fn a_saved_cycle_settles_after_loading() {
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    model
        .set_user_input(0, 1, 1, "=B1/2+10".to_string())
        .unwrap();
    model.set_user_input(0, 1, 2, "=A1+5".to_string()).unwrap();
    model.set_iterative_calculation(true, 100, 1e-9);
    let mut loaded = round_trip(&model);
    loaded.evaluate();
    assert!((number(&loaded, "Sheet1!A1") - 25.0).abs() < 1e-6);
    assert!((number(&loaded, "Sheet1!B1") - 30.0).abs() < 1e-6);
}
