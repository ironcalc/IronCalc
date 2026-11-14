#![allow(clippy::unwrap_used)]

use crate::{test::util::new_empty_model, types::DateSystem, UserModel};

#[test]
fn ironcalc_by_default() {
    let model = UserModel::from_model(new_empty_model());
    assert_eq!(model.get_date_system(), DateSystem::IronCalc);
}

#[test]
fn changing_the_system_keeps_the_values_and_reevaluates() {
    let mut model = UserModel::from_model(new_empty_model());
    model.set_user_input(0, 1, 1, "=YEAR(0)").unwrap();
    model.set_user_input(0, 2, 1, "=DATE(1904,1,1)").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "1899");
    assert_eq!(model.get_formatted_cell_value(0, 2, 1).unwrap(), "1/1/1904");

    model.set_date_system(DateSystem::Excel1904);
    assert_eq!(model.get_date_system(), DateSystem::Excel1904);
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "1904");
    assert_eq!(model.get_formatted_cell_value(0, 2, 1).unwrap(), "1/1/1904");

    model.set_date_system(DateSystem::Excel1900);
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "1900");
}

#[test]
fn undo_and_redo() {
    let mut model = UserModel::from_model(new_empty_model());
    model.set_user_input(0, 1, 1, "=YEAR(0)").unwrap();
    model.set_date_system(DateSystem::Excel1904);
    // Setting the same system again is not an action
    model.set_date_system(DateSystem::Excel1904);

    model.undo().unwrap();
    assert_eq!(model.get_date_system(), DateSystem::IronCalc);
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "1899");
    // The undo went straight to the cell input
    model.undo().unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "");

    model.redo().unwrap();
    model.redo().unwrap();
    assert_eq!(model.get_date_system(), DateSystem::Excel1904);
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "1904");
}

#[test]
fn names() {
    for system in [
        DateSystem::IronCalc,
        DateSystem::Excel1900,
        DateSystem::Excel1904,
    ] {
        assert_eq!(system.to_string().parse::<DateSystem>(), Ok(system));
    }
    assert!("Excel2000".parse::<DateSystem>().is_err());
}
