#![allow(clippy::unwrap_used)]

use crate::model::DefinedNameEntry;
use crate::test::user_model::util::new_empty_user_model;

#[test]
fn create_defined_name() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "42").unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("myName", "Sheet1!$A$1"))
        .unwrap();
    model.set_user_input(0, 5, 7, "=myName").unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 5, 7),
        Ok("42".to_string())
    );

    assert_eq!(
        model.get_defined_name_list(),
        vec![DefinedNameEntry {
            name: "myName".to_string(),
            scope: None,
            formula: "Sheet1!$A$1".to_string(),
            hidden: false,
            comment: String::new()
        }]
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
        .new_defined_name(DefinedNameEntry::new("myName", "Sheet1!$A$1"))
        .unwrap();
    model.set_user_input(0, 5, 7, "=myName").unwrap();

    // Local to Sheet2
    model.new_sheet().unwrap();
    model.set_user_input(1, 2, 1, "145").unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("myName", "Sheet2!$A$2").scope(1))
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
        .new_defined_name(DefinedNameEntry::new("myName", "Sheet1!$A$1").scope(1))
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
        .new_defined_name(DefinedNameEntry::new("myName", "Sheet1!$A$1").scope(1))
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("#NAME?".to_string())
    );

    model
        .update_defined_name(
            "myName",
            Some(1),
            DefinedNameEntry::new("myName", "Sheet1!$A$1"),
        )
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
        .new_defined_name(DefinedNameEntry::new("myName", "Sheet1!$A$1"))
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("Hello world!".to_string())
    );

    model
        .update_defined_name(
            "myName",
            None,
            DefinedNameEntry::new("newName", "Sheet1!$A$1"),
        )
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
        .new_defined_name(DefinedNameEntry::new("answer", "Sheet1!$A$1"))
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
        .new_defined_name(DefinedNameEntry::new("answer", "Sheet1!$A$1").scope(1))
        .unwrap();

    model.set_user_input(1, 3, 1, "=answer").unwrap();

    model
        .update_defined_name(
            "answer",
            None,
            DefinedNameEntry::new("respuesta", "Sheet1!$A$1"),
        )
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
        .new_defined_name(DefinedNameEntry::new("hello", "Sheet1!$A$1"))
        .unwrap();

    model
        .new_defined_name(DefinedNameEntry::new("world", "Sheet1!$B$1"))
        .unwrap();

    model.set_user_input(0, 2, 1, "=hello&world").unwrap();

    model
        .update_defined_name("hello", None, DefinedNameEntry::new("HolaS", "Sheet1!$A$1"))
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
        .new_defined_name(DefinedNameEntry::new("MyName", "Sheet1!$A$1"))
        .unwrap();

    // spaces
    assert_eq!(
        model.new_defined_name(DefinedNameEntry::new("A real", "Sheet1!$A$1")),
        Err("Name: Invalid defined name".to_string())
    );

    // Starts with number
    assert_eq!(
        model.new_defined_name(DefinedNameEntry::new("2real", "Sheet1!$A$1")),
        Err("Name: Invalid defined name".to_string())
    );

    // Updating also fails
    assert_eq!(
        model.update_defined_name(
            "MyName",
            None,
            DefinedNameEntry::new("My Name", "Sheet1!$A$1")
        ),
        Err("Name: Invalid defined name".to_string())
    );
}

#[test]
fn already_existing() {
    let mut model = new_empty_user_model();

    model
        .new_defined_name(DefinedNameEntry::new("MyName", "Sheet1!$A$1"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Another", "Sheet1!$A$1"))
        .unwrap();

    // Can't create a new name with the same name
    assert_eq!(
        model.new_defined_name(DefinedNameEntry::new("MyName", "Sheet1!$A$2")),
        Err("Name: Defined name already exists".to_string())
    );

    // Can't update one into an existing
    assert_eq!(
        model.update_defined_name(
            "Another",
            None,
            DefinedNameEntry::new("MyName", "Sheet1!$A$1")
        ),
        Err("Name: Defined name already exists".to_string())
    );
}

#[test]
fn invalid_sheet() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("MyName", "Sheet1!$A$1"))
        .unwrap();

    assert_eq!(
        model.new_defined_name(DefinedNameEntry::new("Mything", "Sheet1!$A$1").scope(2)),
        Err("Scope: Invalid sheet index".to_string())
    );

    assert_eq!(
        model.update_defined_name(
            "MyName",
            None,
            DefinedNameEntry::new("MyName", "Sheet1!$A$1").scope(2)
        ),
        Err("Scope: Invalid sheet index".to_string())
    );

    assert_eq!(
        model.update_defined_name(
            "MyName",
            Some(9),
            DefinedNameEntry::new("YourName", "Sheet1!$A$1")
        ),
        Err("General: Failed to get old name".to_string())
    );
}

#[test]
fn invalid_formula() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "Hello").unwrap();
    assert!(model
        .new_defined_name(DefinedNameEntry::new("MyName", "A1"))
        .is_err());

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
        .new_defined_name(DefinedNameEntry::new("MyName", "Sheet1!$A$1"))
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
        .update_defined_name(
            "MyName",
            None,
            DefinedNameEntry::new("MyName", "Sheet1!$A$2"),
        )
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
        .new_defined_name(DefinedNameEntry::new("myName", "Sheet1!$A$1"))
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(1, 2, 1),
        Ok("Hello world!".to_string())
    );

    model
        .update_defined_name(
            "myName",
            None,
            DefinedNameEntry::new("myName", "Sheet1!$A$1").scope(0),
        )
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
        .new_defined_name(DefinedNameEntry::new("myName", "Sheet1!$A$1"))
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
        .new_defined_name(DefinedNameEntry::new("MyConst", "Sheet1!$A$1"))
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
        vec![DefinedNameEntry {
            name: "MyConst".to_string(),
            scope: None,
            formula: "Sheet1!$F$11".to_string(),
            hidden: false,
            comment: String::new()
        }]
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
        .new_defined_name(DefinedNameEntry::new("MyRange", "Sheet1!$C$3:$C$5"))
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
        vec![DefinedNameEntry {
            name: "MyRange".to_string(),
            scope: None,
            formula: "Sheet1!$H$3:$H$5".to_string(),
            hidden: false,
            comment: String::new()
        }]
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
        .new_defined_name(DefinedNameEntry::new("Name1", "Sheet1!$A$1"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Name2", "Sheet1!$B$2"))
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
    assert!(names.contains(&DefinedNameEntry {
        name: "Name1".to_string(),
        scope: None,
        formula: "Sheet1!$D$4".to_string(),
        hidden: false,
        comment: String::new()
    }));
    // Name2 is unaffected
    assert!(names.contains(&DefinedNameEntry {
        name: "Name2".to_string(),
        scope: None,
        formula: "Sheet1!$B$2".to_string(),
        hidden: false,
        comment: String::new()
    }));
}

#[test]
fn cut_paste_defined_name_undo_restores_formula() {
    let mut model = new_empty_user_model();

    model.set_user_input(0, 1, 1, "42").unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("MyVal", "Sheet1!$A$1"))
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
        vec![DefinedNameEntry {
            name: "MyVal".to_string(),
            scope: None,
            formula: "Sheet1!$C$5".to_string(),
            hidden: false,
            comment: String::new()
        }]
    );

    // Undo restores the original formula
    model.undo().unwrap();
    assert_eq!(
        model.get_defined_name_list(),
        vec![DefinedNameEntry {
            name: "MyVal".to_string(),
            scope: None,
            formula: "Sheet1!$A$1".to_string(),
            hidden: false,
            comment: String::new()
        }]
    );
}

#[test]
fn cut_paste_scoped_defined_name_updates() {
    let mut model = new_empty_user_model();
    model.new_sheet().unwrap();

    // Sheet1!A1 = 10; scoped defined name on Sheet1: "Local" = Sheet1!$A$1
    model.set_user_input(0, 1, 1, "10").unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Local", "Sheet1!$A$1").scope(0))
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
    assert!(model.get_defined_name_list().contains(&DefinedNameEntry {
        name: "Local".to_string(),
        scope: Some(0),
        formula: "Sheet1!$B$2".to_string(),
        hidden: false,
        comment: String::new()
    }));
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
        .new_defined_name(DefinedNameEntry::new("BigRange", "Sheet1!$B$1:$D$3"))
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
        vec![DefinedNameEntry {
            name: "BigRange".to_string(),
            scope: None,
            formula: "Sheet1!$B$1:$D$3".to_string(),
            hidden: false,
            comment: String::new()
        }]
    );
}

#[test]
fn hidden_defined_name() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "42").unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Secret", "Sheet1!$A$1").hidden(true))
        .unwrap();
    // A hidden name resolves like any other
    model.set_user_input(0, 2, 1, "=Secret").unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("42".to_string())
    );
    assert_eq!(
        model.get_defined_name_list(),
        vec![DefinedNameEntry {
            name: "Secret".to_string(),
            scope: None,
            formula: "Sheet1!$A$1".to_string(),
            hidden: true,
            comment: String::new()
        }]
    );

    // Unhide it, keeping everything else
    model
        .update_defined_name(
            "Secret",
            None,
            DefinedNameEntry::new("Secret", "Sheet1!$A$1"),
        )
        .unwrap();
    assert_eq!(
        model.get_defined_name_list(),
        vec![DefinedNameEntry {
            name: "Secret".to_string(),
            scope: None,
            formula: "Sheet1!$A$1".to_string(),
            hidden: false,
            comment: String::new()
        }]
    );

    // Undo/redo restore the flag
    model.undo().unwrap();
    assert_eq!(
        model.get_defined_name_list(),
        vec![DefinedNameEntry {
            name: "Secret".to_string(),
            scope: None,
            formula: "Sheet1!$A$1".to_string(),
            hidden: true,
            comment: String::new()
        }]
    );
    model.redo().unwrap();
    assert_eq!(
        model.get_defined_name_list(),
        vec![DefinedNameEntry {
            name: "Secret".to_string(),
            scope: None,
            formula: "Sheet1!$A$1".to_string(),
            hidden: false,
            comment: String::new()
        }]
    );

    // Hide it again with a rename, then delete: undo of the delete brings back a hidden name
    model
        .update_defined_name(
            "Secret",
            None,
            DefinedNameEntry::new("Hidden", "Sheet1!$A$1").hidden(true),
        )
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
        vec![DefinedNameEntry {
            name: "Hidden".to_string(),
            scope: None,
            formula: "Sheet1!$A$1".to_string(),
            hidden: true,
            comment: String::new()
        }]
    );
    // Undo the rename, the unhide, the "=Secret" input and finally the creation
    model.undo().unwrap();
    model.undo().unwrap();
    model.undo().unwrap();
    assert_eq!(
        model.get_defined_name_list(),
        vec![DefinedNameEntry {
            name: "Secret".to_string(),
            scope: None,
            formula: "Sheet1!$A$1".to_string(),
            hidden: true,
            comment: String::new()
        }]
    );
    model.undo().unwrap();
    assert_eq!(model.get_defined_name_list().len(), 0);
}

#[test]
fn hidden_defined_name_survives_cut_paste() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "42").unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Secret", "Sheet1!$A$1").hidden(true))
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
        vec![DefinedNameEntry {
            name: "Secret".to_string(),
            scope: None,
            formula: "Sheet1!$C$5".to_string(),
            hidden: true,
            comment: String::new()
        }]
    );
    model.undo().unwrap();
    assert_eq!(
        model.get_defined_name_list(),
        vec![DefinedNameEntry {
            name: "Secret".to_string(),
            scope: None,
            formula: "Sheet1!$A$1".to_string(),
            hidden: true,
            comment: String::new()
        }]
    );
}

#[test]
fn hidden_defined_name_survives_sheet_duplicate_and_rename() {
    let mut model = new_empty_user_model();
    model
        .new_defined_name(
            DefinedNameEntry::new("Secret", "Sheet1!$A$1")
                .scope(0)
                .hidden(true),
        )
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Visible", "Sheet1!$A$2").scope(0))
        .unwrap();

    model.rename_sheet(0, "Renamed").unwrap();
    let names = model.get_defined_name_list();
    assert!(names.contains(&DefinedNameEntry {
        name: "Secret".to_string(),
        scope: Some(0),
        formula: "Renamed!$A$1".to_string(),
        hidden: true,
        comment: String::new()
    }));
    assert!(names.contains(&DefinedNameEntry {
        name: "Visible".to_string(),
        scope: Some(0),
        formula: "Renamed!$A$2".to_string(),
        hidden: false,
        comment: String::new()
    }));

    model.duplicate_sheet(0).unwrap();
    let names = model.get_defined_name_list();
    assert!(names.contains(&DefinedNameEntry {
        name: "Secret".to_string(),
        scope: Some(1),
        formula: "'Renamed (1)'!$A$1".to_string(),
        hidden: true,
        comment: String::new()
    }));
    assert!(names.contains(&DefinedNameEntry {
        name: "Visible".to_string(),
        scope: Some(1),
        formula: "'Renamed (1)'!$A$2".to_string(),
        hidden: false,
        comment: String::new()
    }));
}

#[test]
fn defined_name_comment() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "0.21").unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("TaxRate", "Sheet1!$A$1").comment("VAT in 2024"))
        .unwrap();
    assert_eq!(
        model.get_defined_name_list(),
        vec![DefinedNameEntry {
            name: "TaxRate".to_string(),
            scope: None,
            formula: "Sheet1!$A$1".to_string(),
            hidden: false,
            comment: "VAT in 2024".to_string()
        }]
    );

    // Change the comment only
    model
        .update_defined_name(
            "TaxRate",
            None,
            DefinedNameEntry::new("TaxRate", "Sheet1!$A$1").comment("VAT in 2025"),
        )
        .unwrap();
    assert_eq!(model.get_defined_name_list()[0].comment, "VAT in 2025");

    // Undo/redo restore the comment
    model.undo().unwrap();
    assert_eq!(model.get_defined_name_list()[0].comment, "VAT in 2024");
    model.redo().unwrap();
    assert_eq!(model.get_defined_name_list()[0].comment, "VAT in 2025");

    // Undo of a delete brings the comment back
    model.delete_defined_name("TaxRate", None).unwrap();
    model.undo().unwrap();
    assert_eq!(model.get_defined_name_list()[0].comment, "VAT in 2025");
}

#[test]
fn defined_name_comment_max_length() {
    let mut model = new_empty_user_model();
    let ok = "é".repeat(255);
    let too_long = "é".repeat(256);

    // 255 characters (not bytes) are fine
    model
        .new_defined_name(DefinedNameEntry::new("Name1", "Sheet1!$A$1").comment(&ok))
        .unwrap();
    assert_eq!(model.get_defined_name_list()[0].comment, ok);

    // 256 are not, both when creating and updating
    assert_eq!(
        model.new_defined_name(DefinedNameEntry::new("Name2", "Sheet1!$A$2").comment(&too_long)),
        Err("Comment: Comment must be at most 255 characters".to_string())
    );
    assert_eq!(
        model.update_defined_name(
            "Name1",
            None,
            DefinedNameEntry::new("Name1", "Sheet1!$A$1").comment(&too_long)
        ),
        Err("Comment: Comment must be at most 255 characters".to_string())
    );
    // Nothing changed and nothing was pushed to the history
    assert_eq!(model.get_defined_name_list().len(), 1);
    assert_eq!(model.get_defined_name_list()[0].comment, ok);
    model.undo().unwrap();
    assert_eq!(model.get_defined_name_list().len(), 0);
}

#[test]
fn defined_name_comment_survives_cut_paste_rename_and_duplicate() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "42").unwrap();
    model
        .new_defined_name(
            DefinedNameEntry::new("Rate", "Sheet1!$A$1")
                .scope(0)
                .hidden(true)
                .comment("Local rate"),
        )
        .unwrap();

    // Cut A1 to C5
    model.set_selected_cell(1, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(5, 3).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 1, 1), &cp.data, true)
        .unwrap();
    assert_eq!(
        model.get_defined_name_list(),
        vec![DefinedNameEntry {
            name: "Rate".to_string(),
            scope: Some(0),
            formula: "Sheet1!$C$5".to_string(),
            hidden: true,
            comment: "Local rate".to_string()
        }]
    );

    model.rename_sheet(0, "Renamed").unwrap();
    model.duplicate_sheet(0).unwrap();
    let names = model.get_defined_name_list();
    assert!(names.contains(&DefinedNameEntry {
        name: "Rate".to_string(),
        scope: Some(0),
        formula: "Renamed!$C$5".to_string(),
        hidden: true,
        comment: "Local rate".to_string()
    }));
    assert!(names.contains(&DefinedNameEntry {
        name: "Rate".to_string(),
        scope: Some(1),
        formula: "'Renamed (1)'!$C$5".to_string(),
        hidden: true,
        comment: "Local rate".to_string()
    }));
}
