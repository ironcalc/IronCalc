#![allow(clippy::unwrap_used)]

use crate::test::user_model::util::new_empty_user_model;

#[test]
fn create_named_style() {
    let mut model = new_empty_user_model();
    let mut style = model.get_cell_style(0, 1, 1).unwrap();
    style.font.b = true;
    model.create_named_style("bold", &style).unwrap();

    assert!(model.get_named_style_list().contains(&"bold".to_string()));
    assert!(model.get_named_style("bold").unwrap().font.b);

    // Duplicate name fails
    assert!(model.create_named_style("bold", &style).is_err());
}

#[test]
fn create_named_style_undo_redo() {
    let mut model = new_empty_user_model();
    let mut style = model.get_cell_style(0, 1, 1).unwrap();
    style.font.b = true;
    model.create_named_style("bold", &style).unwrap();
    assert!(model.get_named_style_list().contains(&"bold".to_string()));

    // Undo removes the named style
    model.undo().unwrap();
    assert!(!model.get_named_style_list().contains(&"bold".to_string()));

    // Redo restores it
    model.redo().unwrap();
    assert!(model.get_named_style_list().contains(&"bold".to_string()));
    assert!(model.get_named_style("bold").unwrap().font.b);
}

#[test]
fn delete_named_style() {
    let mut model = new_empty_user_model();
    let mut style = model.get_cell_style(0, 1, 1).unwrap();
    style.font.b = true;
    model.create_named_style("bold", &style).unwrap();
    // Apply to a cell via the raw model API
    model
        .model
        .set_cell_style_by_name(0, 1, 1, "bold")
        .unwrap();
    assert!(model.get_cell_style(0, 1, 1).unwrap().font.b);

    // Delete the named style
    model.delete_named_style("bold").unwrap();
    assert!(!model.get_named_style_list().contains(&"bold".to_string()));
    // Cell retains its formatting after the named style is deleted
    assert!(model.get_cell_style(0, 1, 1).unwrap().font.b);

    // Built-in "normal" cannot be deleted
    assert!(model.delete_named_style("normal").is_err());
    // Non-existent style fails
    assert!(model.delete_named_style("nonexistent").is_err());
}

#[test]
fn delete_named_style_undo_redo() {
    let mut model = new_empty_user_model();
    let mut style = model.get_cell_style(0, 1, 1).unwrap();
    style.font.b = true;
    model.create_named_style("bold", &style).unwrap();
    model.delete_named_style("bold").unwrap();
    assert!(!model.get_named_style_list().contains(&"bold".to_string()));

    // Undo restores the named style
    model.undo().unwrap();
    assert!(model.get_named_style_list().contains(&"bold".to_string()));
    assert!(model.get_named_style("bold").unwrap().font.b);

    // Redo deletes it again
    model.redo().unwrap();
    assert!(!model.get_named_style_list().contains(&"bold".to_string()));
}

#[test]
fn update_named_style_updates_cells() {
    let mut model = new_empty_user_model();
    let mut style = model.get_cell_style(0, 1, 1).unwrap();
    style.font.b = true;
    model.create_named_style("bold", &style).unwrap();

    // Apply the named style to A1 by setting the style directly
    model
        .model
        .set_cell_style_by_name(0, 1, 1, "bold")
        .unwrap();
    model
        .model
        .set_cell_style_by_name(0, 2, 1, "bold")
        .unwrap();

    // Update the style to also be italic
    let mut new_style = model.get_named_style("bold").unwrap();
    new_style.font.i = true;
    model.update_named_style("bold", "bold", &new_style).unwrap();

    // Both cells should reflect the new formatting
    assert!(model.get_cell_style(0, 1, 1).unwrap().font.i);
    assert!(model.get_cell_style(0, 2, 1).unwrap().font.i);
    assert!(model.get_named_style("bold").unwrap().font.i);
}

#[test]
fn update_named_style_undo_redo() {
    let mut model = new_empty_user_model();
    let mut style = model.get_cell_style(0, 1, 1).unwrap();
    style.font.b = true;
    model.create_named_style("bold", &style).unwrap();
    model
        .model
        .set_cell_style_by_name(0, 1, 1, "bold")
        .unwrap();

    let mut new_style = model.get_named_style("bold").unwrap();
    new_style.font.i = true;
    model.update_named_style("bold", "bold", &new_style).unwrap();

    assert!(model.get_cell_style(0, 1, 1).unwrap().font.i);

    // Undo: cell should lose the italic
    model.undo().unwrap();
    assert!(!model.get_cell_style(0, 1, 1).unwrap().font.i);
    assert!(!model.get_named_style("bold").unwrap().font.i);

    // Redo: cell gets italic back
    model.redo().unwrap();
    assert!(model.get_cell_style(0, 1, 1).unwrap().font.i);
    assert!(model.get_named_style("bold").unwrap().font.i);
}

#[test]
fn update_named_style_rename() {
    let mut model = new_empty_user_model();
    let mut style = model.get_cell_style(0, 1, 1).unwrap();
    style.font.b = true;
    model.create_named_style("bold", &style).unwrap();

    // Rename without changing the style
    model.update_named_style("bold", "bold2", &style).unwrap();
    assert!(!model.get_named_style_list().contains(&"bold".to_string()));
    assert!(model.get_named_style_list().contains(&"bold2".to_string()));

    // Undo restores original name
    model.undo().unwrap();
    assert!(model.get_named_style_list().contains(&"bold".to_string()));
    assert!(!model.get_named_style_list().contains(&"bold2".to_string()));

    // Redo renames again
    model.redo().unwrap();
    assert!(model.get_named_style_list().contains(&"bold2".to_string()));
}

#[test]
fn update_named_style_rejects_builtin() {
    let mut model = new_empty_user_model();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert!(model.update_named_style("normal", "normal", &style).is_err());
    assert!(model.delete_named_style("normal").is_err());
}

#[test]
fn get_named_style_list_includes_default() {
    let model = new_empty_user_model();
    let list = model.get_named_style_list();
    // Default model has "normal"
    assert!(list.iter().any(|n| n.eq_ignore_ascii_case("normal")));
}
