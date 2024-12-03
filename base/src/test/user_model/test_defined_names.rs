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

    // delete it
    model.delete_defined_name("myName", None).unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 5, 7),
        Ok("#NAME?".to_string())
    );

    model.undo().unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 5, 7),
        Ok("42".to_string())
    );
}

#[test]
fn scopes() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "42").unwrap();

    // Global
    model
        .new_defined_name("myName", None, "Sheet1!$A$1")
        .unwrap();
    model.set_user_input(0, 5, 7, "=myName").unwrap();

    // Local to Sheet2
    model.new_sheet().unwrap();
    model.set_user_input(1, 2, 1, "145").unwrap();
    model
        .new_defined_name("myName", Some(1), "Sheet2!$A$2")
        .unwrap();
    model.set_user_input(1, 8, 8, "=myName").unwrap();

    // Sheet 3
    model.new_sheet().unwrap();
    model.set_user_input(2, 2, 2, "=myName").unwrap();

    // Global
    assert_eq!(
        model.get_formatted_cell_value(0, 5, 7),
        Ok("42".to_string())
    );

    assert_eq!(
        model.get_formatted_cell_value(1, 8, 8),
        Ok("145".to_string())
    );

    assert_eq!(
        model.get_formatted_cell_value(2, 2, 2),
        Ok("42".to_string())
    );
}

#[test]
fn delete_sheet() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    model
        .set_user_input(0, 2, 1, r#"=CONCATENATE(MyName, " world!")"#)
        .unwrap();
    model.new_sheet().unwrap();
    model
        .new_defined_name("myName", Some(1), "Sheet1!$A$1")
        .unwrap();
    model
        .set_user_input(1, 2, 1, r#"=CONCATENATE(MyName, " my world!")"#)
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("#NAME?".to_string())
    );

    assert_eq!(
        model.get_formatted_cell_value(1, 2, 1),
        Ok("Hello my world!".to_string())
    );

    model.delete_sheet(0).unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("#NAME?".to_string())
    );

    assert_eq!(
        model.get_cell_content(0, 2, 1),
        Ok(r#"=CONCATENATE(MyName," my world!")"#.to_string())
    );
}

#[test]
fn change_scope() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    model
        .set_user_input(0, 2, 1, r#"=CONCATENATE(MyName, " world!")"#)
        .unwrap();
    model.new_sheet().unwrap();
    model
        .new_defined_name("myName", Some(1), "Sheet1!$A$1")
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("#NAME?".to_string())
    );

    model
        .update_defined_name("myName", Some(1), "myName", None, "Sheet1!$A$1")
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("Hello world!".to_string())
    );
}

#[test]
fn rename_defined_name() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    model
        .set_user_input(0, 2, 1, r#"=CONCATENATE(MyName, " world!")"#)
        .unwrap();
    model.new_sheet().unwrap();
    model
        .new_defined_name("myName", None, "Sheet1!$A$1")
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("Hello world!".to_string())
    );

    model
        .update_defined_name("myName", None, "newName", None, "Sheet1!$A$1")
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("Hello world!".to_string())
    );

    assert_eq!(
        model.get_cell_content(0, 2, 1),
        Ok(r#"=CONCATENATE(newName," world!")"#.to_string())
    );
}
