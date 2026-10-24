#![allow(clippy::unwrap_used)]

use crate::test::user_model::util::new_empty_user_model;

#[test]
fn create_defined_name() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "42").unwrap();
    model
        .new_defined_name("myName", None, "Sheet1!$A$1", false)
        .unwrap();
    model.set_user_input(0, 5, 7, "=myName").unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 5, 7),
        Ok("42".to_string())
    );

    assert_eq!(
        model.get_defined_name_list(),
        vec![("myName".to_string(), None, "Sheet1!$A$1".to_string(), false)]
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
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "42").unwrap();

    // Global
    model
        .new_defined_name("myName", None, "Sheet1!$A$1", false)
        .unwrap();
    model.set_user_input(0, 5, 7, "=myName").unwrap();

    // Local to Sheet2
    model.new_sheet().unwrap();
    model.set_user_input(1, 2, 1, "145").unwrap();
    model
        .new_defined_name("myName", Some(1), "Sheet2!$A$2", false)
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
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    model
        .set_user_input(0, 2, 1, r#"=CONCATENATE(MyName, " world!")"#)
        .unwrap();
    model.new_sheet().unwrap();
    model
        .new_defined_name("myName", Some(1), "Sheet1!$A$1", false)
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
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    model
        .set_user_input(0, 2, 1, r#"=CONCATENATE(MyName, " world!")"#)
        .unwrap();
    model.new_sheet().unwrap();
    model
        .new_defined_name("myName", Some(1), "Sheet1!$A$1", false)
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("#NAME?".to_string())
    );

    model
        .update_defined_name("myName", Some(1), "myName", None, "Sheet1!$A$1", false)
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("Hello world!".to_string())
    );
}

#[test]
fn rename_defined_name() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    model
        .set_user_input(0, 2, 1, r#"=CONCATENATE(MyName, " world!")"#)
        .unwrap();
    model.new_sheet().unwrap();
    model
        .new_defined_name("myName", None, "Sheet1!$A$1", false)
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("Hello world!".to_string())
    );

    model
        .update_defined_name("myName", None, "newName", None, "Sheet1!$A$1", false)
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
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "42").unwrap();
    model.set_user_input(0, 1, 2, "123").unwrap();

    model
        .new_defined_name("answer", None, "Sheet1!$A$1", false)
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
        .new_defined_name("answer", Some(1), "Sheet1!$A$1", false)
        .unwrap();

    model.set_user_input(1, 3, 1, "=answer").unwrap();

    model
        .update_defined_name("answer", None, "respuesta", None, "Sheet1!$A$1", false)
        .unwrap();

    assert_eq!(
        model.get_cell_content(0, 2, 1),
        Ok("=IF(respuesta<2,respuesta*2,respuesta^2)".to_string())
    );

    assert_eq!(
        model.get_cell_content(0, 3, 1),
        Ok("=baddunction(-respuesta)".to_string())
    );

    // A defined name with the same name but different scope
    assert_eq!(model.get_cell_content(1, 3, 1), Ok("=answer".to_string()));
}

#[test]
fn rename_defined_name_string_operations() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    model.set_user_input(0, 1, 2, "World").unwrap();

    model
        .new_defined_name("hello", None, "Sheet1!$A$1", false)
        .unwrap();

    model
        .new_defined_name("world", None, "Sheet1!$B$1", false)
        .unwrap();

    model.set_user_input(0, 2, 1, "=hello&world").unwrap();

    model
        .update_defined_name("hello", None, "HolaS", None, "Sheet1!$A$1", false)
        .unwrap();

    assert_eq!(
        model.get_cell_content(0, 2, 1),
        Ok("=HolaS&world".to_string())
    );
}

#[test]
fn invalid_names() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    model
        .new_defined_name("MyName", None, "Sheet1!$A$1", false)
        .unwrap();

    // spaces
    assert_eq!(
        model.new_defined_name("A real", None, "Sheet1!$A$1", false),
        Err("Name: Invalid defined name".to_string())
    );

    // Starts with number
    assert_eq!(
        model.new_defined_name("2real", None, "Sheet1!$A$1", false),
        Err("Name: Invalid defined name".to_string())
    );

    // Updating also fails
    assert_eq!(
        model.update_defined_name("MyName", None, "My Name", None, "Sheet1!$A$1", false),
        Err("Name: Invalid defined name".to_string())
    );
}

#[test]
fn already_existing() {
    let mut model = new_empty_user_model();

    model
        .new_defined_name("MyName", None, "Sheet1!$A$1", false)
        .unwrap();
    model
        .new_defined_name("Another", None, "Sheet1!$A$1", false)
        .unwrap();

    // Can't create a new name with the same name
    assert_eq!(
        model.new_defined_name("MyName", None, "Sheet1!$A$2", false),
        Err("Name: Defined name already exists".to_string())
    );

    // Can't update one into an existing
    assert_eq!(
        model.update_defined_name("Another", None, "MyName", None, "Sheet1!$A$1", false),
        Err("Name: Defined name already exists".to_string())
    );
}

#[test]
fn invalid_sheet() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    model
        .new_defined_name("MyName", None, "Sheet1!$A$1", false)
        .unwrap();

    assert_eq!(
        model.new_defined_name("Mything", Some(2), "Sheet1!$A$1", false),
        Err("Scope: Invalid sheet index".to_string())
    );

    assert_eq!(
        model.update_defined_name("MyName", None, "MyName", Some(2), "Sheet1!$A$1", false),
        Err("Scope: Invalid sheet index".to_string())
    );

    assert_eq!(
        model.update_defined_name("MyName", Some(9), "YourName", None, "Sheet1!$A$1", false),
        Err("General: Failed to get old name".to_string())
    );
}

#[test]
fn invalid_formula() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    assert!(model.new_defined_name("MyName", None, "A1", false).is_err());

    model.set_user_input(0, 1, 2, "=MyName").unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("#NAME?".to_string())
    );
}

#[test]
fn undo_redo() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    model.set_user_input(0, 2, 1, "Hola").unwrap();
    model.set_user_input(0, 1, 2, r#"=MyName&"!""#).unwrap();

    model
        .new_defined_name("MyName", None, "Sheet1!$A$1", false)
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
        .update_defined_name("MyName", None, "MyName", None, "Sheet1!$A$2", false)
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

    let mut model2 = new_empty_user_model();
    model2.apply_external_diffs(&send_queue).unwrap();

    assert_eq!(model2.get_defined_name_list().len(), 1);
    assert_eq!(
        model2.get_formatted_cell_value(0, 1, 2),
        Ok("Hola!".to_string())
    );
}

#[test]
fn change_scope_to_first_sheet() {
    let mut model = new_empty_user_model();
    model.new_sheet().unwrap();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    model
        .set_user_input(1, 2, 1, r#"=CONCATENATE(MyName, " world!")"#)
        .unwrap();
    model
        .new_defined_name("myName", None, "Sheet1!$A$1", false)
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(1, 2, 1),
        Ok("Hello world!".to_string())
    );

    model
        .update_defined_name("myName", None, "myName", Some(0), "Sheet1!$A$1", false)
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(1, 2, 1),
        Ok("#NAME?".to_string())
    );
}

#[test]
fn rename_sheet() {
    let mut model = new_empty_user_model();
    model.new_sheet().unwrap();
    model.set_user_input(0, 1, 1, "Hello").unwrap();

    model
        .new_defined_name("myName", None, "Sheet1!$A$1", false)
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

// When a cell referenced by a defined name is cut and pasted, the defined
// name's formula should be updated to the new location.

#[test]
fn cut_paste_updates_defined_name_cell_reference() {
    let mut model = new_empty_user_model();

    // A1 = 11; defined name "MyConst" = Sheet1!$A$1
    model.set_user_input(0, 1, 1, "11").unwrap();
    model
        .new_defined_name("MyConst", None, "Sheet1!$A$1", false)
        .unwrap();
    model.set_user_input(0, 3, 3, "=SEQUENCE(MyConst)").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 3, 3).unwrap(), "1");

    // Cut A1, paste to F11
    model.set_selected_cell(1, 1).unwrap();
    model.set_selected_range(1, 1, 1, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(11, 6).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 1, 1), &cp.data, true)
        .unwrap();

    // "MyConst" must now point to $F$11
    assert_eq!(
        model.get_defined_name_list(),
        vec![(
            "MyConst".to_string(),
            None,
            "Sheet1!$F$11".to_string(),
            false
        )]
    );
    // The formula using the defined name still evaluates correctly
    assert_eq!(model.get_formatted_cell_value(0, 3, 3).unwrap(), "1");
}

#[test]
fn cut_paste_updates_defined_name_range_reference() {
    let mut model = new_empty_user_model();

    // C3:C5 = 1,2,3; defined name "MyRange" = Sheet1!$C$3:$C$5
    model.set_user_input(0, 3, 3, "1").unwrap();
    model.set_user_input(0, 4, 3, "2").unwrap();
    model.set_user_input(0, 5, 3, "3").unwrap();
    model
        .new_defined_name("MyRange", None, "Sheet1!$C$3:$C$5", false)
        .unwrap();
    model.set_user_input(0, 1, 2, "=SUM(MyRange)").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 2).unwrap(), "6");

    // Cut C3:C5, paste to H3:H5
    model.set_selected_cell(3, 3).unwrap();
    model.set_selected_range(3, 3, 5, 3).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(3, 8).unwrap();
    model
        .paste_from_clipboard(0, (3, 3, 5, 3), &cp.data, true)
        .unwrap();

    // "MyRange" must now point to $H$3:$H$5
    assert_eq!(
        model.get_defined_name_list(),
        vec![(
            "MyRange".to_string(),
            None,
            "Sheet1!$H$3:$H$5".to_string(),
            false
        )]
    );
    assert_eq!(model.get_formatted_cell_value(0, 1, 2).unwrap(), "6");
}

#[test]
fn cut_paste_defined_name_no_update_when_outside_moved_area() {
    let mut model = new_empty_user_model();

    // A1 = 5; B2 = 7; "Name1" = Sheet1!$A$1; "Name2" = Sheet1!$B$2
    model.set_user_input(0, 1, 1, "5").unwrap();
    model.set_user_input(0, 2, 2, "7").unwrap();
    model
        .new_defined_name("Name1", None, "Sheet1!$A$1", false)
        .unwrap();
    model
        .new_defined_name("Name2", None, "Sheet1!$B$2", false)
        .unwrap();

    // Cut only A1, paste to D4 — Name2 must not change
    model.set_selected_cell(1, 1).unwrap();
    model.set_selected_range(1, 1, 1, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(4, 4).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 1, 1), &cp.data, true)
        .unwrap();

    let names = model.get_defined_name_list();
    // Name1 moved to D4
    assert!(names.contains(&("Name1".to_string(), None, "Sheet1!$D$4".to_string(), false)));
    // Name2 is unaffected
    assert!(names.contains(&("Name2".to_string(), None, "Sheet1!$B$2".to_string(), false)));
}

#[test]
fn cut_paste_defined_name_undo_restores_formula() {
    let mut model = new_empty_user_model();

    model.set_user_input(0, 1, 1, "42").unwrap();
    model
        .new_defined_name("MyVal", None, "Sheet1!$A$1", false)
        .unwrap();

    // Cut A1, paste to C5
    model.set_selected_cell(1, 1).unwrap();
    model.set_selected_range(1, 1, 1, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(5, 3).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 1, 1), &cp.data, true)
        .unwrap();

    assert_eq!(
        model.get_defined_name_list(),
        vec![("MyVal".to_string(), None, "Sheet1!$C$5".to_string(), false)]
    );

    // Undo restores the original formula
    model.undo().unwrap();
    assert_eq!(
        model.get_defined_name_list(),
        vec![("MyVal".to_string(), None, "Sheet1!$A$1".to_string(), false)]
    );
}

#[test]
fn cut_paste_scoped_defined_name_updates() {
    let mut model = new_empty_user_model();
    model.new_sheet().unwrap();

    // Sheet1!A1 = 10; scoped defined name on Sheet1: "Local" = Sheet1!$A$1
    model.set_user_input(0, 1, 1, "10").unwrap();
    model
        .new_defined_name("Local", Some(0), "Sheet1!$A$1", false)
        .unwrap();

    // Cut A1, paste to B2
    model.set_selected_cell(1, 1).unwrap();
    model.set_selected_range(1, 1, 1, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(2, 2).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 1, 1), &cp.data, true)
        .unwrap();

    // The scoped defined name must also update
    assert!(model.get_defined_name_list().contains(&(
        "Local".to_string(),
        Some(0),
        "Sheet1!$B$2".to_string(),
        false
    )));
}

#[test]
fn cut_paste_range_defined_name_no_update_when_partial_overlap() {
    let mut model = new_empty_user_model();

    // B1:D3 = various values; "BigRange" covers more than what we cut
    for r in 1..=3_i32 {
        for c in 2..=4_i32 {
            model
                .set_user_input(0, r, c, &(r * 10 + c).to_string())
                .unwrap();
        }
    }
    // "BigRange" = Sheet1!$B$1:$D$3 (3x3 area)
    model
        .new_defined_name("BigRange", None, "Sheet1!$B$1:$D$3", false)
        .unwrap();

    // Cut only B1:B3 (subset of BigRange) to F1:F3
    model.set_selected_cell(1, 2).unwrap();
    model.set_selected_range(1, 2, 3, 2).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(1, 6).unwrap();
    model
        .paste_from_clipboard(0, (1, 2, 3, 2), &cp.data, true)
        .unwrap();

    // BigRange only partially overlaps the moved area — it must NOT be updated
    assert_eq!(
        model.get_defined_name_list(),
        vec![(
            "BigRange".to_string(),
            None,
            "Sheet1!$B$1:$D$3".to_string(),
            false
        )]
    );
}

#[test]
fn hidden_defined_name() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "42").unwrap();
    model
        .new_defined_name("Secret", None, "Sheet1!$A$1", true)
        .unwrap();
    // A hidden name resolves like any other
    model.set_user_input(0, 2, 1, "=Secret").unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("42".to_string())
    );
    assert_eq!(
        model.get_defined_name_list(),
        vec![("Secret".to_string(), None, "Sheet1!$A$1".to_string(), true)]
    );

    // Unhide it, keeping everything else
    model
        .update_defined_name("Secret", None, "Secret", None, "Sheet1!$A$1", false)
        .unwrap();
    assert_eq!(
        model.get_defined_name_list(),
        vec![("Secret".to_string(), None, "Sheet1!$A$1".to_string(), false)]
    );

    // Undo/redo restore the flag
    model.undo().unwrap();
    assert_eq!(
        model.get_defined_name_list(),
        vec![("Secret".to_string(), None, "Sheet1!$A$1".to_string(), true)]
    );
    model.redo().unwrap();
    assert_eq!(
        model.get_defined_name_list(),
        vec![("Secret".to_string(), None, "Sheet1!$A$1".to_string(), false)]
    );

    // Hide it again with a rename, then delete: undo of the delete brings back a hidden name
    model
        .update_defined_name("Secret", None, "Hidden", None, "Sheet1!$A$1", true)
        .unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("42".to_string())
    );
    model.delete_defined_name("Hidden", None).unwrap();
    assert_eq!(model.get_defined_name_list().len(), 0);
    model.undo().unwrap();
    assert_eq!(
        model.get_defined_name_list(),
        vec![("Hidden".to_string(), None, "Sheet1!$A$1".to_string(), true)]
    );
    // Undo the rename, the unhide, the "=Secret" input and finally the creation
    model.undo().unwrap();
    model.undo().unwrap();
    model.undo().unwrap();
    assert_eq!(
        model.get_defined_name_list(),
        vec![("Secret".to_string(), None, "Sheet1!$A$1".to_string(), true)]
    );
    model.undo().unwrap();
    assert_eq!(model.get_defined_name_list().len(), 0);
}

#[test]
fn hidden_defined_name_survives_cut_paste() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "42").unwrap();
    model
        .new_defined_name("Secret", None, "Sheet1!$A$1", true)
        .unwrap();

    // Cut A1 and paste it to C5: the name follows and stays hidden
    model.set_selected_cell(1, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(5, 3).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 1, 1), &cp.data, true)
        .unwrap();
    assert_eq!(
        model.get_defined_name_list(),
        vec![("Secret".to_string(), None, "Sheet1!$C$5".to_string(), true)]
    );
    model.undo().unwrap();
    assert_eq!(
        model.get_defined_name_list(),
        vec![("Secret".to_string(), None, "Sheet1!$A$1".to_string(), true)]
    );
}

#[test]
fn hidden_defined_name_survives_sheet_duplicate_and_rename() {
    let mut model = new_empty_user_model();
    model
        .new_defined_name("Secret", Some(0), "Sheet1!$A$1", true)
        .unwrap();
    model
        .new_defined_name("Visible", Some(0), "Sheet1!$A$2", false)
        .unwrap();

    model.rename_sheet(0, "Renamed").unwrap();
    let names = model.get_defined_name_list();
    assert!(names.contains(&(
        "Secret".to_string(),
        Some(0),
        "Renamed!$A$1".to_string(),
        true
    )));
    assert!(names.contains(&(
        "Visible".to_string(),
        Some(0),
        "Renamed!$A$2".to_string(),
        false
    )));

    model.duplicate_sheet(0).unwrap();
    let names = model.get_defined_name_list();
    assert!(names.contains(&(
        "Secret".to_string(),
        Some(1),
        "'Renamed (1)'!$A$1".to_string(),
        true
    )));
    assert!(names.contains(&(
        "Visible".to_string(),
        Some(1),
        "'Renamed (1)'!$A$2".to_string(),
        false
    )));
}
