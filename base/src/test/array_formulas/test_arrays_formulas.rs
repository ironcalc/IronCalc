#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn set_values() {
    let mut model = new_empty_model();
    // set a 2x2 array formula with value "Hello, world!" in A5:B6
    model
        .set_user_array_formula(0, 5, 1, 2, 2, "Hello, world!".to_string())
        .unwrap();
    model.evaluate();

    // check that the value is correctly set in the 4 cells
    assert_eq!(model._get_text_at(0, 5, 1), "Hello, world!");
    assert_eq!(model._get_text_at(0, 5, 2), "Hello, world!");
    assert_eq!(model._get_text_at(0, 6, 1), "Hello, world!");
    assert_eq!(model._get_text_at(0, 6, 2), "Hello, world!");

    // check there is no formula in A5
    assert!(!model._has_formula("A5"));
    assert_eq!(model._get_text("A5"), "Hello, world!");

    // check we can delete A6
    model.set_user_input(0, 6, 1, "".to_string()).unwrap();
    model.evaluate();
    assert_eq!(model._get_text_at(0, 6, 1), "");
}

#[test]
fn set_simple_formula() {
    let mut model = new_empty_model();
    // set a 2x2 array formula with value "=1+1" in A5:B6
    model
        .set_user_array_formula(0, 5, 1, 2, 2, "=1+1".to_string())
        .unwrap();
    model.evaluate();

    // check that the value is correctly set in the 4 cells
    assert_eq!(model._get_text_at(0, 5, 1), "2");
    assert_eq!(model._get_text_at(0, 5, 2), "2");
    assert_eq!(model._get_text_at(0, 6, 1), "2");
    assert_eq!(model._get_text_at(0, 6, 2), "2");

    // check there is a formula in A5
    assert!(model._has_formula("A5"));
    assert_eq!(model._get_formula("A5"), "=1+1");

    // check we cannot delete A6
    assert!(model.set_user_input(0, 6, 1, "".to_string()).is_err());
    assert!(model.cell_clear_contents(0, 6, 1).is_err());
    assert!(model.cell_clear_all(0, 6, 1).is_err());

    // assert we can delete A5 and that it deletes the whole array formula
    model.set_user_input(0, 5, 1, "".to_string()).unwrap();
    model.evaluate();
    assert_eq!(model._get_text_at(0, 5, 1), "");
    assert_eq!(model._get_text_at(0, 5, 2), "");
    assert_eq!(model._get_text_at(0, 6, 1), "");
    assert_eq!(model._get_text_at(0, 6, 2), "");
}

#[test]
fn overwrite_quote_index() {
    let mut model = new_empty_model();
    // set a 2x2 array formula with value "=1+1" in A5:B6
    model
        .set_user_array_formula(0, 5, 1, 2, 2, "=1+1".to_string())
        .unwrap();
    model.evaluate();

    // // set a quote prefix style in A5
    // let quote_prefix_style_index = model.workbook.styles.add_quote_prefix_style();
    // model.set_cell_style_index(0, 5, 1, quote_prefix_style_index).unwrap();

    // // check that the quote prefix style is not lost in the spillover cells
    // assert!(model.workbook.styles.style_is_quote_prefix(
    //     model.get_cell_style_index(0, 5, 1).unwrap()
    // ));
    // assert!(model.workbook.styles.style_is_quote_prefix(
    //     model.get_cell_style_index(0, 5, 2).unwrap()
    // ));
    // assert!(model.workbook.styles.style_is_quote_prefix(
    //     model.get_cell_style_index(0, 6, 1).unwrap()
    // ));
    // assert!(model.workbook.styles.style_is_quote_prefix(
    //     model.get_cell_style_index(0, 6, 2).unwrap()
    // ));
}
