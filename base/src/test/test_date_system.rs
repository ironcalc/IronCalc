#![allow(clippy::unwrap_used)]

use crate::{model::Model, test::util::new_empty_model, types::DateSystem};

#[test]
fn new_workbooks_use_the_ironcalc_date_system() {
    let model = new_empty_model();
    assert_eq!(model.workbook.settings.date_system, DateSystem::IronCalc);
}

#[test]
fn date_system_default_is_ironcalc() {
    assert_eq!(DateSystem::default(), DateSystem::IronCalc);
}

#[test]
fn date_system_survives_a_round_trip_through_bytes() {
    let mut model = new_empty_model();
    model.workbook.settings.date_system = DateSystem::Excel1904;
    let bytes = model.to_bytes();
    let model2 = Model::from_bytes(&bytes, "en").unwrap();
    assert_eq!(model2.workbook.settings.date_system, DateSystem::Excel1904);
}
