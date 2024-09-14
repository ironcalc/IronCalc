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
