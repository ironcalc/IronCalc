#![allow(clippy::unwrap_used)]

use crate::{
    cf_types::{CfRule, Cfvo, ValueOperator},
    test::util::new_empty_model,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn model_with_values() -> crate::Model<'static> {
    let mut model = new_empty_model();
    for i in 1i32..=5 {
        model.set_user_input(0, i, 1, i.to_string()).unwrap();
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

fn cell_is_gt(threshold: &str) -> CfRule {
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
    let mut model = new_empty_model();
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
    let mut model = new_empty_model();
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
    let mut model = new_empty_model();
    assert!(model
        .add_conditional_formatting(99, "A1:A5", color_scale_rule())
        .is_err());
}

#[test]
fn test_add_invalid_range_errors() {
    let mut model = new_empty_model();
    assert!(model
        .add_conditional_formatting(0, "not_a_range", color_scale_rule())
        .is_err());
}

#[test]
fn test_add_assigns_increasing_priorities() {
    let mut model = new_empty_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    model
        .add_conditional_formatting(0, "B1:B5", data_bar_rule())
        .unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert!(
        list[0].priority < list[1].priority,
        "first-added rule should have lower (higher-priority) number"
    );
}

// ---------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------

#[test]
fn test_delete_rule() {
    let mut model = new_empty_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    model.delete_conditional_formatting(0, 0).unwrap();
    assert_eq!(model.get_conditional_formatting_list(0).unwrap().len(), 0);
}

#[test]
fn test_delete_first_of_two() {
    let mut model = new_empty_model();
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
    let mut model = new_empty_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    assert!(model.delete_conditional_formatting(0, 5).is_err());
}

#[test]
fn test_delete_invalid_sheet_errors() {
    let mut model = new_empty_model();
    assert!(model.delete_conditional_formatting(99, 0).is_err());
}

// ---------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------

#[test]
fn test_update_rule_type() {
    let mut model = new_empty_model();
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
    let mut model = new_empty_model();
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
fn test_update_preserves_priority() {
    let mut model = new_empty_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    let priority_before = model.get_conditional_formatting_list(0).unwrap()[0].priority;
    model
        .update_conditional_formatting(0, 0, "C1:C10", data_bar_rule())
        .unwrap();
    let priority_after = model.get_conditional_formatting_list(0).unwrap()[0].priority;
    assert_eq!(priority_before, priority_after);
}

#[test]
fn test_update_out_of_bounds_errors() {
    let mut model = new_empty_model();
    assert!(model
        .update_conditional_formatting(0, 99, "A1:A5", color_scale_rule())
        .is_err());
}

#[test]
fn test_update_invalid_range_errors() {
    let mut model = new_empty_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    assert!(model
        .update_conditional_formatting(0, 0, "!!!!", color_scale_rule())
        .is_err());
}

// ---------------------------------------------------------------------------
// Effect on evaluated cell styles
// ---------------------------------------------------------------------------

#[test]
fn test_color_scale_applies_fill() {
    let mut model = model_with_values();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    model.evaluate();

    let style_a1 = model.get_extended_style_for_cell(0, 1, 1).unwrap();
    let style_a5 = model.get_extended_style_for_cell(0, 5, 1).unwrap();
    assert_eq!(style_a1.style.fill.bg_color, Some("#FF0000".to_string()));
    assert_eq!(style_a5.style.fill.bg_color, Some("#00FF00".to_string()));
}

#[test]
fn test_data_bar_applies() {
    let mut model = model_with_values();
    model
        .add_conditional_formatting(0, "A1:A5", data_bar_rule())
        .unwrap();
    model.evaluate();

    let style_a1 = model.get_extended_style_for_cell(0, 1, 1).unwrap();
    let style_a5 = model.get_extended_style_for_cell(0, 5, 1).unwrap();
    assert_eq!(style_a1.data_bar.unwrap().value, 0.0);
    assert_eq!(style_a5.data_bar.unwrap().value, 1.0);
}

#[test]
fn test_delete_removes_applied_style() {
    let mut model = model_with_values();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    model.evaluate();
    assert!(model
        .get_extended_style_for_cell(0, 1, 1)
        .unwrap()
        .style
        .fill
        .bg_color
        .is_some());

    model.delete_conditional_formatting(0, 0).unwrap();
    model.evaluate();
    assert!(model
        .get_extended_style_for_cell(0, 1, 1)
        .unwrap()
        .style
        .fill
        .bg_color
        .is_none());
}

#[test]
fn test_rule_only_applies_inside_range() {
    let mut model = model_with_values();
    model
        .add_conditional_formatting(0, "A1:A3", color_scale_rule())
        .unwrap();
    model.evaluate();
    // A4 is outside the range
    assert!(model
        .get_extended_style_for_cell(0, 4, 1)
        .unwrap()
        .style
        .fill
        .bg_color
        .is_none());
}

#[test]
fn test_lower_priority_number_wins() {
    let mut model = model_with_values();
    // First rule (lower priority number = higher priority): red→green scale
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    // Second rule: blue→yellow scale
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
    model.evaluate();

    // A1 = min: first rule's min color wins
    let style = model.get_extended_style_for_cell(0, 1, 1).unwrap();
    assert_eq!(style.style.fill.bg_color, Some("#FF0000".to_string()));
}

#[test]
fn test_cell_is_rule_stored_correctly() {
    let mut model = model_with_values();
    model
        .add_conditional_formatting(0, "A1:A5", cell_is_gt("3"))
        .unwrap();
    model.evaluate();

    let list = model.get_conditional_formatting_list(0).unwrap();
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
    let model = new_empty_model();
    assert!(model.get_conditional_formatting_list(0).unwrap().is_empty());
}

#[test]
fn test_rules_on_different_sheets_are_independent() {
    let mut model = new_empty_model();
    model.new_sheet();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    assert!(model
        .get_conditional_formatting_list(1)
        .unwrap()
        .is_empty());
    assert_eq!(
        model.get_conditional_formatting_list(0).unwrap().len(),
        1
    );
}
