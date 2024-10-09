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
fn test_model_style_set_fns_in_merge_cell_context() {
    let mut model = new_empty_model();

    //creating a merge cell of D1:F2
    model.update_merge_cell(0, "D1:F2").unwrap();
    model.set_user_input(0, 1, 4, "Hello".to_string()).unwrap();

    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    assert!(!style.font.b);
    style.font.b = true;

    //Updating the mother cell of Merge cells and expecting the update to go through
    // This should make the text "Hello" in bold format
    assert_eq!(model.set_cell_style(0, 1, 4, &style), Ok(()));

    // 1: testing with set_cell_style()
    let original_style: Style = model.get_style_for_cell(0, 1, 5).unwrap();
    assert_eq!(
        model
            .set_cell_style(0, 1, 5, &style),
        Err("Cell row : 1, col : 5 is part of merged cell block, so singular update to the cell is not possible".to_string())
    );
    assert_eq!(model.get_style_for_cell(0, 1, 5), Ok(original_style));

    // 2: testing with set_cell_style_by_name
    let mut style = model.get_style_for_cell(0, 1, 4).unwrap();
    style.font.b = true;
    assert_eq!(
        model.workbook.styles.create_named_style("bold", &style),
        Ok(())
    );

    let original_style: Style = model.get_style_for_cell(0, 1, 5).unwrap();
    assert_eq!(
        model
            .set_cell_style_by_name(0, 1, 5, "bold"),
        Err("Cell row : 1, col : 5 is part of merged cell block, so singular update to the cell is not possible".to_string())
    );
    assert_eq!(model.get_style_for_cell(0, 1, 5), Ok(original_style));
}
