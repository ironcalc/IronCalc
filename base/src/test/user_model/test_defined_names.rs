#![allow(clippy::unwrap_used)]

use crate::UserModel;

#[test]
fn create_defined_name() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "42").unwrap();
    model
        .new_defined_name("myName", None, "Sheet1!$A$1")
        .unwrap();
    model.set_user_input(0, 5, 7, "=myName").unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 5, 7),
        Ok("42".to_string())
    );

    // rename it
    model
        .update_defined_name("myName", None, "myName", None, "$A$1*2")
        .unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 5, 7),
        Ok("42".to_string())
    );

    // delete it
    model.delete_defined_name("myName", None).unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 5, 7),
        Ok("#NAME?".to_string())
    );
}

#[test]
fn rename_defined_name() {}

#[test]
fn delete_sheet() {}

#[test]
fn change_scope() {}
