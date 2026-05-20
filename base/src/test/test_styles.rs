#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;
use crate::types::Style;

#[test]
fn test_model_set_cells_with_values_styles() {
    let mut model = new_empty_model();
    // Inputs
    model.set_user_input(0, 1, 1, "21".to_string()).unwrap(); // A1
    model.set_user_input(0, 2, 1, "42".to_string()).unwrap(); // A2

    let style_base = model.get_style_for_cell(0, 1, 1).unwrap();
    let mut style = style_base.clone();
    style.font.b = true;
    style.num_fmt = "#,##0.00".to_string();
    assert!(model.set_cell_style(0, 1, 1, &style).is_ok());

    let mut style = style_base;
    style.num_fmt = "#,##0.00".to_string();
    assert!(model.set_cell_style(0, 2, 1, &style).is_ok());
    let style: Style = model.get_style_for_cell(0, 2, 1).unwrap();
    assert_eq!(style.num_fmt, "#,##0.00".to_string());
}

#[test]
fn test_named_styles() {
    let mut model = new_empty_model();
    model._set("A1", "42");
    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.font.b = true;
    assert!(model.set_cell_style(0, 1, 1, &style).is_ok());
    let bold_style_index = model.get_cell_style_index(0, 1, 1).unwrap();
    let e = model
        .workbook
        .styles
        .add_named_cell_style("bold", bold_style_index);
    assert!(e.is_ok());
    model._set("A2", "420");
    let a2_style_index = model.get_cell_style_index(0, 2, 1).unwrap();
    assert!(a2_style_index != bold_style_index);
    let e = model.set_cell_style_by_name(0, 2, 1, "bold");
    assert!(e.is_ok());
    assert_eq!(model.get_cell_style_index(0, 2, 1), Ok(bold_style_index));
}

#[test]
fn test_create_named_style() {
    let mut model = new_empty_model();
    model._set("A1", "42");

    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    assert!(!style.font.b);

    style.font.b = true;
    let e = model.workbook.styles.create_named_style("bold", &style);
    assert!(e.is_ok());

    let e = model.set_cell_style_by_name(0, 1, 1, "bold");
    assert!(e.is_ok());

    let style = model.get_style_for_cell(0, 1, 1).unwrap();
    assert!(style.font.b);
}

#[test]
fn test_get_named_style_list() {
    let model = new_empty_model();
    let list = model.get_named_style_list();
    // Default model always includes "Normal" plus all built-in styles
    assert!(list.iter().any(|n| n.eq_ignore_ascii_case("normal")));
    assert!(list.iter().any(|n| n == "Good"));
    assert!(list.iter().any(|n| n == "Bad"));
    assert!(list.iter().any(|n| n == "Heading 1"));
    assert!(list.iter().any(|n| n == "20% - Accent1"));
    assert!(list.len() > 10);
}

#[test]
fn test_get_named_style() {
    let mut model = new_empty_model();
    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.font.b = true;
    model.create_named_style("bold", &style).unwrap();
    let retrieved = model.get_named_style("bold").unwrap();
    assert!(retrieved.font.b);
}

#[test]
fn test_create_named_style_via_model() {
    let mut model = new_empty_model();
    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.font.b = true;
    assert!(model.create_named_style("bold", &style).is_ok());
    // Duplicate name fails
    assert!(model.create_named_style("bold", &style).is_err());
    // Can apply the style to a cell
    model.set_cell_style_by_name(0, 1, 1, "bold").unwrap();
    assert!(model.get_style_for_cell(0, 1, 1).unwrap().font.b);
}

#[test]
fn test_delete_named_style() {
    let mut model = new_empty_model();
    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.font.b = true;
    model.create_named_style("bold", &style).unwrap();
    // Apply it to a cell
    model.set_cell_style_by_name(0, 1, 1, "bold").unwrap();
    assert!(model.get_style_for_cell(0, 1, 1).unwrap().font.b);
    // Delete the named style
    assert!(model.delete_named_style("bold").is_ok());
    // Named style is gone
    assert!(model.get_named_style("bold").is_err());
    // Cell retains its formatting
    assert!(model.get_style_for_cell(0, 1, 1).unwrap().font.b);
    // Built-in "normal" cannot be deleted
    assert!(model.delete_named_style("normal").is_err());
    // Deleting a non-existent style fails
    assert!(model.delete_named_style("nonexistent").is_err());
}

#[test]
fn test_update_named_style_updates_cells() {
    let mut model = new_empty_model();
    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.font.b = true;
    model.create_named_style("bold", &style).unwrap();
    // Apply to A1 and A2
    model.set_cell_style_by_name(0, 1, 1, "bold").unwrap();
    model.set_cell_style_by_name(0, 2, 1, "bold").unwrap();
    // Update the style to also be italic
    let mut new_style = model.get_named_style("bold").unwrap();
    new_style.font.i = true;
    model.update_named_style("bold", "bold", &new_style).unwrap();
    // Both cells now have the updated style
    assert!(model.get_style_for_cell(0, 1, 1).unwrap().font.i);
    assert!(model.get_style_for_cell(0, 2, 1).unwrap().font.i);
    // The named style reflects the new formatting
    assert!(model.get_named_style("bold").unwrap().font.i);
}

#[test]
fn test_update_named_style_rename() {
    let mut model = new_empty_model();
    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.font.b = true;
    model.create_named_style("bold", &style).unwrap();
    // Rename it
    model.update_named_style("bold", "bold2", &style).unwrap();
    assert!(model.get_named_style("bold").is_err());
    assert!(model.get_named_style("bold2").is_ok());
}

#[test]
fn test_update_named_style_rejects_builtin() {
    let mut model = new_empty_model();
    let style = model.get_style_for_cell(0, 1, 1).unwrap();
    assert!(model.update_named_style("normal", "normal", &style).is_err());
    assert!(model.delete_named_style("normal").is_err());
}

#[test]
fn empty_models_have_builtin_fills() {
    let model = new_empty_model();
    // The first two fills are always none and gray125 (Excel requirements)
    assert_eq!(
        model.workbook.styles.fills[0].pattern_type,
        "none".to_string()
    );
    assert_eq!(
        model.workbook.styles.fills[1].pattern_type,
        "gray125".to_string()
    );
    // Built-in named styles add many more fills
    assert!(model.workbook.styles.fills.len() > 2);
}
