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

    assert_eq!(
        model.get_defined_name_list(),
        vec![("myName".to_string(), None, "Sheet1!$A$1".to_string())]
    );

    // delete it
    model.delete_defined_name("myName", None).unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 5, 7),
        Ok("#NAME?".to_string())
    );

    assert_eq!(model.get_defined_name_list().len(), 0);

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

#[test]
fn rename_defined_name_operations() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "42").unwrap();
    model.set_user_input(0, 1, 2, "123").unwrap();

    model
        .new_defined_name("answer", None, "Sheet1!$A$1")
        .unwrap();

    model
        .set_user_input(0, 2, 1, "=IF(answer<2, answer*2, answer^2)")
        .unwrap();

    model
        .set_user_input(0, 3, 1, "=badDunction(-answer)")
        .unwrap();

    model.new_sheet().unwrap();
    model.set_user_input(1, 1, 1, "78").unwrap();
    model
        .new_defined_name("answer", Some(1), "Sheet1!$A$1")
        .unwrap();

    model.set_user_input(1, 3, 1, "=answer").unwrap();

    model
        .update_defined_name("answer", None, "respuesta", None, "Sheet1!$A$1")
        .unwrap();

    assert_eq!(
        model.get_cell_content(0, 2, 1),
        Ok("=IF(respuesta<2,respuesta*2,respuesta^2)".to_string())
    );

    assert_eq!(
        model.get_cell_content(0, 3, 1),
        Ok("=badDunction(-respuesta)".to_string())
    );

    // A defined name with the same name but different scope
    assert_eq!(model.get_cell_content(1, 3, 1), Ok("=answer".to_string()));
}

#[test]
fn rename_defined_name_string_operations() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    model.set_user_input(0, 1, 2, "World").unwrap();

    model
        .new_defined_name("hello", None, "Sheet1!$A$1")
        .unwrap();

    model
        .new_defined_name("world", None, "Sheet1!$B$1")
        .unwrap();

    model.set_user_input(0, 2, 1, "=hello&world").unwrap();

    model
        .update_defined_name("hello", None, "HolaS", None, "Sheet1!$A$1")
        .unwrap();

    assert_eq!(
        model.get_cell_content(0, 2, 1),
        Ok("=HolaS&world".to_string())
    );
}

#[test]
fn invalid_names() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    model
        .new_defined_name("MyName", None, "Sheet1!$A$1")
        .unwrap();

    // spaces
    assert_eq!(
        model.new_defined_name("A real", None, "Sheet1!$A$1"),
        Err("Invalid defined name".to_string())
    );

    // Starts with number
    assert_eq!(
        model.new_defined_name("2real", None, "Sheet1!$A$1"),
        Err("Invalid defined name".to_string())
    );

    // Updating also fails
    assert_eq!(
        model.update_defined_name("MyName", None, "My Name", None, "Sheet1!$A$1"),
        Err("Invalid defined name".to_string())
    );
}

#[test]
fn already_existing() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();

    model
        .new_defined_name("MyName", None, "Sheet1!$A$1")
        .unwrap();
    model
        .new_defined_name("Another", None, "Sheet1!$A$1")
        .unwrap();

    // Can't create a new name with the same name
    assert_eq!(
        model.new_defined_name("MyName", None, "Sheet1!$A$2"),
        Err("Defined name already exists".to_string())
    );

    // Can't update one into an existing
    assert_eq!(
        model.update_defined_name("Another", None, "MyName", None, "Sheet1!$A$1"),
        Err("Defined name already exists".to_string())
    );
}

#[test]
fn invalid_sheet() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    model
        .new_defined_name("MyName", None, "Sheet1!$A$1")
        .unwrap();

    assert_eq!(
        model.new_defined_name("Mything", Some(2), "Sheet1!$A$1"),
        Err("Invalid sheet index".to_string())
    );

    assert_eq!(
        model.update_defined_name("MyName", None, "MyName", Some(2), "Sheet1!$A$1"),
        Err("Invalid sheet index".to_string())
    );

    assert_eq!(
        model.update_defined_name("MyName", Some(9), "YourName", None, "Sheet1!$A$1"),
        Err("Invalid sheet index".to_string())
    );
}

#[test]
fn invalid_formula() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    model.new_defined_name("MyName", None, "A1").unwrap();

    model.set_user_input(0, 1, 2, "=MyName").unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("#NAME?".to_string())
    );
}

#[test]
fn undo_redo() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    model.set_user_input(0, 2, 1, "Hola").unwrap();
    model.set_user_input(0, 1, 2, r#"=MyName&"!""#).unwrap();

    model
        .new_defined_name("MyName", None, "Sheet1!$A$1")
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("Hello!".to_string())
    );
    model.undo().unwrap();
    assert_eq!(model.get_defined_name_list().len(), 0);
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("#NAME?".to_string())
    );
    model.redo().unwrap();

    assert_eq!(model.get_defined_name_list().len(), 1);
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("Hello!".to_string())
    );

    model
        .update_defined_name("MyName", None, "MyName", None, "Sheet1!$A$2")
        .unwrap();
    assert_eq!(model.get_defined_name_list().len(), 1);
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("Hola!".to_string())
    );
    model.undo().unwrap();

    assert_eq!(model.get_defined_name_list().len(), 1);
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("Hello!".to_string())
    );

    model.redo().unwrap();

    assert_eq!(model.get_defined_name_list().len(), 1);
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("Hola!".to_string())
    );

    let send_queue = model.flush_send_queue();

    let mut model2 = UserModel::new_empty("model", "en", "UTC").unwrap();
    model2.apply_external_diffs(&send_queue).unwrap();

    assert_eq!(model2.get_defined_name_list().len(), 1);
    assert_eq!(
        model2.get_formatted_cell_value(0, 1, 2),
        Ok("Hola!".to_string())
    );
}

#[test]
fn change_scope_to_first_sheet() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.new_sheet().unwrap();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    model
        .set_user_input(1, 2, 1, r#"=CONCATENATE(MyName, " world!")"#)
        .unwrap();
    model
        .new_defined_name("myName", None, "Sheet1!$A$1")
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(1, 2, 1),
        Ok("Hello world!".to_string())
    );

    model
        .update_defined_name("myName", None, "myName", Some(0), "Sheet1!$A$1")
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(1, 2, 1),
        Ok("#NAME?".to_string())
    );
}

#[test]
fn rename_sheet() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.new_sheet().unwrap();
    model.set_user_input(0, 1, 1, "Hello").unwrap();

    model
        .new_defined_name("myName", None, "Sheet1!$A$1")
        .unwrap();

    model
        .set_user_input(0, 2, 1, r#"=CONCATENATE(MyName, " world!")"#)
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("Hello world!".to_string())
    );

    model.rename_sheet(0, "AnotherName").unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("Hello world!".to_string())
    );
}
