#![allow(clippy::unwrap_used)]

use crate::{test::util::new_empty_model, UserModel};

#[test]
fn basic() {
    let mut model1 = UserModel::from_model(new_empty_model());
    let width = model1.get_column_width(0, 3).unwrap() * 3.0;
    model1.set_columns_width(0, 3, 3, width).unwrap();
    model1.set_user_input(0, 1, 2, "Hello IronCalc!").unwrap();

    let model_bytes = model1.to_bytes();

    let model2 = UserModel::from_bytes(&model_bytes).unwrap();

    assert_eq!(model2.get_column_width(0, 3), Ok(width));
    assert_eq!(
        model2.get_formatted_cell_value(0, 1, 2),
        Ok("Hello IronCalc!".to_string())
    );
}

#[test]
fn errors() {
    let model_bytes = "Early in the morning, late in the century, Cricklewood Broadway.".as_bytes();
    assert_eq!(
        &UserModel::from_bytes(model_bytes).unwrap_err(),
        "Error parsing workbook: invalid packing"
    );
}
