#![allow(clippy::unwrap_used)]

use crate::{
    cf_types::{CfRule, Cfvo, ConditionalFormatting, ValueOperator},
    test::user_model::util::new_empty_user_model,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn model_with_values() -> crate::UserModel<'static> {
    let mut model = new_empty_user_model();
    for i in 1i32..=5 {
        model.set_user_input(0, i, 1, &i.to_string()).unwrap();
    }
    model.evaluate();
    model
}

fn color_scale_rule() -> CfRule {
    CfRule::ColorScale {
        cfvo: vec![Cfvo::Min, Cfvo::Max],
        colors: vec!["#FF0000".to_string(), "#00FF00".to_string()],
    }
}

fn data_bar_rule() -> CfRule {
    CfRule::DataBar {
        cfvo: vec![Cfvo::Min, Cfvo::Max],
        color: "#0000FF".to_string(),
        show_value: true,
    }
}

fn cell_is_greater_than(threshold: &str) -> CfRule {
    CfRule::CellIs {
        operator: ValueOperator::GreaterThan,
        formula: threshold.to_string(),
        formula2: None,
        dxf_id: 0,
    }
}

// ---------------------------------------------------------------------------
// Add
// ---------------------------------------------------------------------------

#[test]
fn test_add_rule_appears_in_list() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].range, "A1:A5");
    assert!(matches!(list[0].cf_rule, CfRule::ColorScale { .. }));
}

#[test]
fn test_add_multiple_rules() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    model
        .add_conditional_formatting(0, "B1:B10", data_bar_rule())
        .unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].range, "A1:A5");
    assert_eq!(list[1].range, "B1:B10");
}

#[test]
fn test_add_invalid_sheet_errors() {
    let mut model = new_empty_user_model();
    let result = model.add_conditional_formatting(99, "A1:A5", color_scale_rule());
    assert!(result.is_err());
}

#[test]
fn test_add_invalid_range_errors() {
    let mut model = new_empty_user_model();
    let result = model.add_conditional_formatting(0, "not_a_range", color_scale_rule());
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------

#[test]
fn test_delete_rule() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    assert_eq!(model.get_conditional_formatting_list(0).unwrap().len(), 1);

    model.delete_conditional_formatting(0, 0).unwrap();
    assert_eq!(model.get_conditional_formatting_list(0).unwrap().len(), 0);
}

#[test]
fn test_delete_one_of_two() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    model
        .add_conditional_formatting(0, "B1:B5", data_bar_rule())
        .unwrap();

    model.delete_conditional_formatting(0, 0).unwrap();

    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].range, "B1:B5");
}

#[test]
fn test_delete_out_of_bounds_errors() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    let result = model.delete_conditional_formatting(0, 5);
    assert!(result.is_err());
}

#[test]
fn test_delete_invalid_sheet_errors() {
    let mut model = new_empty_user_model();
    let result = model.delete_conditional_formatting(99, 0);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------

#[test]
fn test_update_rule_type() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();

    model
        .update_conditional_formatting(0, 0, "A1:A5", data_bar_rule())
        .unwrap();

    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 1);
    assert!(matches!(list[0].cf_rule, CfRule::DataBar { .. }));
}

#[test]
fn test_update_range() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();

    model
        .update_conditional_formatting(0, 0, "C1:C10", color_scale_rule())
        .unwrap();

    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list[0].range, "C1:C10");
}

#[test]
fn test_update_out_of_bounds_errors() {
    let mut model = new_empty_user_model();
    let result = model.update_conditional_formatting(0, 99, "A1:A5", color_scale_rule());
    assert!(result.is_err());
}

#[test]
fn test_update_invalid_range_errors() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    let result = model.update_conditional_formatting(0, 0, "!!!!", color_scale_rule());
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Effect on cell styles
// ---------------------------------------------------------------------------

#[test]
fn test_color_scale_applies_fill() {
    let mut model = model_with_values();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    model.evaluate();

    // A1 = 1 (min) → red (#FF0000), A5 = 5 (max) → green (#00FF00)
    let style_a1 = model.get_extended_cell_style(0, 1, 1).unwrap();
    let style_a5 = model.get_extended_cell_style(0, 5, 1).unwrap();

    assert_eq!(
        style_a1.style.fill.bg_color,
        Some("#FF0000".to_string()),
        "A1 should be red (min color)"
    );
    assert_eq!(
        style_a5.style.fill.bg_color,
        Some("#00FF00".to_string()),
        "A5 should be green (max color)"
    );
}

#[test]
fn test_data_bar_applies() {
    let mut model = model_with_values();
    model
        .add_conditional_formatting(0, "A1:A5", data_bar_rule())
        .unwrap();
    model.evaluate();

    let style_a1 = model.get_extended_cell_style(0, 1, 1).unwrap();
    let style_a5 = model.get_extended_cell_style(0, 5, 1).unwrap();

    let bar_a1 = style_a1.data_bar.expect("A1 should have a data bar");
    let bar_a5 = style_a5.data_bar.expect("A5 should have a data bar");

    assert_eq!(bar_a1.value, 0.0, "A1 (min) bar should be 0");
    assert_eq!(bar_a5.value, 1.0, "A5 (max) bar should be 1");
}

#[test]
fn test_delete_removes_applied_style() {
    let mut model = model_with_values();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    model.evaluate();

    // Verify it applies
    let before = model.get_extended_cell_style(0, 1, 1).unwrap();
    assert!(before.style.fill.bg_color.is_some());

    // Delete and re-evaluate
    model.delete_conditional_formatting(0, 0).unwrap();
    model.evaluate();

    let after = model.get_extended_cell_style(0, 1, 1).unwrap();
    assert!(
        after.style.fill.bg_color.is_none(),
        "bg_color should be gone after rule deleted"
    );
}

#[test]
fn test_rule_only_applies_in_range() {
    let mut model = model_with_values();
    // A1:A3 only — A4 and A5 should not be affected
    model
        .add_conditional_formatting(
            0,
            "A1:A3",
            CfRule::ColorScale {
                cfvo: vec![Cfvo::Min, Cfvo::Max],
                colors: vec!["#FF0000".to_string(), "#00FF00".to_string()],
            },
        )
        .unwrap();
    model.evaluate();

    let style_a4 = model.get_extended_cell_style(0, 4, 1).unwrap();
    assert!(
        style_a4.style.fill.bg_color.is_none(),
        "A4 is outside the CF range and should not have a fill"
    );
}

// ---------------------------------------------------------------------------
// Priority ordering
// ---------------------------------------------------------------------------

#[test]
fn test_lower_priority_number_wins() {
    let mut model = model_with_values();

    // Rule with lower priority number (= higher priority) wins.
    // We add two color scales; the one inserted first gets priority 1.
    // After adding both, we manually check priorities and confirm the first rule's
    // color is applied to the overlapping cells.
    model
        .add_conditional_formatting(
            0,
            "A1:A5",
            CfRule::ColorScale {
                cfvo: vec![Cfvo::Min, Cfvo::Max],
                colors: vec!["#FF0000".to_string(), "#00FF00".to_string()],
            },
        )
        .unwrap();
    model
        .add_conditional_formatting(
            0,
            "A1:A5",
            CfRule::ColorScale {
                cfvo: vec![Cfvo::Min, Cfvo::Max],
                colors: vec!["#0000FF".to_string(), "#FFFF00".to_string()],
            },
        )
        .unwrap();

    let list = model.get_conditional_formatting_list(0).unwrap();
    // First rule has lower priority number (=higher priority)
    assert!(list[0].priority < list[1].priority);

    model.evaluate();

    // A1 = min: should show first rule's min color (#FF0000), not second's (#0000FF)
    let style = model.get_extended_cell_style(0, 1, 1).unwrap();
    assert_eq!(style.style.fill.bg_color, Some("#FF0000".to_string()));
}

// ---------------------------------------------------------------------------
// Undo / redo
// ---------------------------------------------------------------------------

#[test]
fn test_undo_add() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    assert_eq!(model.get_conditional_formatting_list(0).unwrap().len(), 1);

    model.undo().unwrap();
    assert_eq!(
        model.get_conditional_formatting_list(0).unwrap().len(),
        0,
        "rule should be gone after undo"
    );
}

#[test]
fn test_redo_add() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    model.undo().unwrap();
    assert_eq!(model.get_conditional_formatting_list(0).unwrap().len(), 0);

    model.redo().unwrap();
    assert_eq!(
        model.get_conditional_formatting_list(0).unwrap().len(),
        1,
        "rule should be back after redo"
    );
    assert_eq!(
        model.get_conditional_formatting_list(0).unwrap()[0].range,
        "A1:A5"
    );
}

#[test]
fn test_undo_delete() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    model.delete_conditional_formatting(0, 0).unwrap();
    assert_eq!(model.get_conditional_formatting_list(0).unwrap().len(), 0);

    model.undo().unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 1, "rule should be restored after undo");
    assert_eq!(list[0].range, "A1:A5");
    assert!(matches!(list[0].cf_rule, CfRule::ColorScale { .. }));
}

#[test]
fn test_undo_update() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    model
        .update_conditional_formatting(0, 0, "C1:C10", data_bar_rule())
        .unwrap();

    model.undo().unwrap();

    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list[0].range, "A1:A5", "range should revert after undo");
    assert!(
        matches!(list[0].cf_rule, CfRule::ColorScale { .. }),
        "rule type should revert after undo"
    );
}

#[test]
fn test_redo_update() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    model
        .update_conditional_formatting(0, 0, "C1:C10", data_bar_rule())
        .unwrap();
    model.undo().unwrap();

    model.redo().unwrap();

    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list[0].range, "C1:C10");
    assert!(matches!(list[0].cf_rule, CfRule::DataBar { .. }));
}

#[test]
fn test_cell_is_rule_matching() {
    // CellIs GreaterThan 3 on A1:A5. Values 4 and 5 should match.
    // We can't verify a visual style change without a dxf, but we can verify the
    // rule is stored correctly and the model doesn't crash.
    let mut model = model_with_values();
    model
        .add_conditional_formatting(0, "A1:A5", cell_is_greater_than("3"))
        .unwrap();
    model.evaluate(); // should not panic

    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 1);
    assert!(matches!(
        list[0].cf_rule,
        CfRule::CellIs {
            operator: ValueOperator::GreaterThan,
            ..
        }
    ));
}

#[test]
fn test_get_list_empty_initially() {
    let model = new_empty_user_model();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert!(list.is_empty());
}

#[test]
fn test_multi_sheet_rules_are_independent() {
    let mut model = new_empty_user_model();
    model.new_sheet().unwrap();

    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();

    // Sheet 1 should be empty
    let list1 = model.get_conditional_formatting_list(1).unwrap();
    assert!(list1.is_empty(), "sheet 1 should have no CF rules");

    // Sheet 0 should have one
    let list0 = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list0.len(), 1);
}
