#![allow(clippy::unwrap_used, clippy::panic)]
//! Regression tests for #1126 at the
//! `UserModel` level. They make sure the undo/redo diff machinery keeps working
//! when conditional-formatting rules and defined names are created in a
//! non-English language: the diffs hold the internal (English) formula, so they
//! replay correctly regardless of the active language.

use crate::{
    cf_types::{CfRule, CfRuleInput},
    test::user_model::util::new_empty_user_model,
    types::{Color, Dxf, Fill},
    UserModel,
};

fn red_fill() -> Dxf {
    Dxf {
        fill: Some(Fill {
            color: Color::Rgb("#FF0000".to_string()),
        }),
        font: None,
        border: None,
        num_fmt: None,
        alignment: None,
    }
}

fn stored_cf_formula(model: &UserModel<'static>) -> String {
    match &model.model.workbook.worksheets[0].conditional_formatting[0].cf_rule {
        CfRule::Formula { formula, .. } => formula.clone(),
        other => panic!("expected a Formula rule, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Defined names
// ---------------------------------------------------------------------------
#[test]
fn defined_name_created_in_spanish_undo_redo() {
    let mut model = new_empty_user_model();
    model.set_language("es").unwrap();
    model
        .new_defined_name("MiSi", None, "=LAMBDA(x, SI(x>0, 1, 2))")
        .unwrap();
    model.set_user_input(0, 1, 1, "=MiSi(5)").unwrap();
    assert_eq!(model.model._get_text("A1"), *"1");

    // Stored in English, displayed in Spanish.
    assert_eq!(
        model.model.workbook.defined_names[0].formula,
        "=LAMBDA(x,IF(x>0,1,2))"
    );
    assert_eq!(model.get_defined_name_list()[0].2, "=LAMBDA(x,SI(x>0,1,2))");

    // Switching language must not break undo/redo: the diff carries English.
    model.set_language("en").unwrap();

    model.undo().unwrap(); // undo the A1 = "=MiSi(5)" input
    model.undo().unwrap(); // undo the defined-name creation
    assert!(model.get_defined_name_list().is_empty());

    model.redo().unwrap(); // redo the defined-name creation
    assert_eq!(
        model.model.workbook.defined_names[0].formula,
        "=LAMBDA(x,IF(x>0,1,2))"
    );
    model.redo().unwrap(); // redo the A1 input
    assert_eq!(model.model._get_text("A1"), *"1");
}

// ---------------------------------------------------------------------------
// Conditional formatting
// ---------------------------------------------------------------------------
#[test]
fn cf_formula_created_in_spanish_undo_redo() {
    let mut model = new_empty_user_model();
    for row in 1..=10 {
        model.set_user_input(0, row, 1, &row.to_string()).unwrap();
    }
    model.set_language("es").unwrap();
    model
        .add_conditional_formatting(
            0,
            "A1:A10",
            CfRuleInput::Formula {
                formula: "=SI($A1>5,VERDADERO,FALSO)".to_string(),
                format: red_fill(),
                stop_if_true: false,
            },
        )
        .unwrap();

    // Stored in English, displayed in Spanish.
    assert_eq!(stored_cf_formula(&model), "=IF($A1>5,TRUE,FALSE)");
    let list = model.get_conditional_formatting_list(0).unwrap();
    let CfRule::Formula { formula, .. } = &list[0].cf_rule else {
        panic!("expected a Formula rule");
    };
    assert_eq!(formula, "=SI($A1>5,VERDADERO,FALSO)");

    let is_red = |m: &UserModel<'static>, row: i32| {
        m.model
            .get_extended_style_for_cell(0, row, 1)
            .unwrap()
            .style
            .fill
            .color
            == Color::Rgb("#FF0000".to_string())
    };
    assert!(is_red(&model, 10));

    // Switch to English then undo/redo.
    model.set_language("en").unwrap();

    model.undo().unwrap();
    assert!(model.get_conditional_formatting_list(0).unwrap().is_empty());

    model.redo().unwrap();
    assert_eq!(stored_cf_formula(&model), "=IF($A1>5,TRUE,FALSE)");
    assert!(is_red(&model, 10));
    assert!(!is_red(&model, 1));
}
