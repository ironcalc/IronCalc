#![allow(clippy::unwrap_used)]

use crate::{
    cf_types::{CfRule, Cfvo, ValueOperator},
    test::user_model::util::new_empty_user_model,
};

fn color_scale() -> CfRule {
    CfRule::ColorScale {
        cfvo: vec![Cfvo::Min, Cfvo::Max],
        colors: vec!["#FF0000".to_string(), "#00FF00".to_string()],
    }
}

fn data_bar() -> CfRule {
    CfRule::DataBar {
        cfvo: vec![Cfvo::Min, Cfvo::Max],
        color: "#0000FF".to_string(),
        show_value: true,
    }
}

// ---------------------------------------------------------------------------
// Basic CRUD
// ---------------------------------------------------------------------------

#[test]
fn test_add_and_list() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].range, "A1:A5");
}

#[test]
fn test_delete() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();
    model.delete_conditional_formatting(0, 0).unwrap();
    assert!(model
        .get_conditional_formatting_list(0)
        .unwrap()
        .is_empty());
}

#[test]
fn test_update() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();
    model
        .update_conditional_formatting(0, 0, "B1:B10", data_bar())
        .unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list[0].range, "B1:B10");
    assert!(matches!(list[0].cf_rule, CfRule::DataBar { .. }));
}

#[test]
fn test_errors_propagate() {
    let mut model = new_empty_user_model();
    assert!(model
        .add_conditional_formatting(99, "A1:A5", color_scale())
        .is_err());
    assert!(model
        .add_conditional_formatting(0, "not_a_range", color_scale())
        .is_err());
    assert!(model.delete_conditional_formatting(0, 0).is_err());
    assert!(model
        .update_conditional_formatting(0, 0, "A1:A5", color_scale())
        .is_err());
}

// ---------------------------------------------------------------------------
// Auto-evaluation: style changes visible immediately after mutation
// ---------------------------------------------------------------------------

#[test]
fn test_add_triggers_evaluation() {
    let mut model = new_empty_user_model();
    for i in 1i32..=5 {
        model.set_user_input(0, i, 1, &i.to_string()).unwrap();
    }
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();

    // No explicit evaluate() needed — UserModel auto-evaluates
    let style = model.get_extended_cell_style(0, 1, 1).unwrap();
    assert_eq!(style.style.fill.bg_color, Some("#FF0000".to_string()));
}

#[test]
fn test_delete_triggers_evaluation() {
    let mut model = new_empty_user_model();
    for i in 1i32..=5 {
        model.set_user_input(0, i, 1, &i.to_string()).unwrap();
    }
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();
    assert!(model
        .get_extended_cell_style(0, 1, 1)
        .unwrap()
        .style
        .fill
        .bg_color
        .is_some());

    model.delete_conditional_formatting(0, 0).unwrap();
    assert!(model
        .get_extended_cell_style(0, 1, 1)
        .unwrap()
        .style
        .fill
        .bg_color
        .is_none());
}

#[test]
fn test_update_triggers_evaluation() {
    let mut model = new_empty_user_model();
    for i in 1i32..=5 {
        model.set_user_input(0, i, 1, &i.to_string()).unwrap();
    }
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();

    // Narrow the range to A1:A2 — A5 should lose its fill
    model
        .update_conditional_formatting(0, 0, "A1:A2", color_scale())
        .unwrap();
    assert!(model
        .get_extended_cell_style(0, 5, 1)
        .unwrap()
        .style
        .fill
        .bg_color
        .is_none());
}

// ---------------------------------------------------------------------------
// Undo / redo — add
// ---------------------------------------------------------------------------

#[test]
fn test_undo_add_removes_rule() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();
    model.undo().unwrap();
    assert!(model
        .get_conditional_formatting_list(0)
        .unwrap()
        .is_empty());
}

#[test]
fn test_redo_add_restores_rule() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();
    model.undo().unwrap();
    model.redo().unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].range, "A1:A5");
    assert!(matches!(list[0].cf_rule, CfRule::ColorScale { .. }));
}

#[test]
fn test_undo_add_removes_applied_style() {
    let mut model = new_empty_user_model();
    for i in 1i32..=5 {
        model.set_user_input(0, i, 1, &i.to_string()).unwrap();
    }
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();
    assert!(model
        .get_extended_cell_style(0, 1, 1)
        .unwrap()
        .style
        .fill
        .bg_color
        .is_some());

    model.undo().unwrap();
    assert!(model
        .get_extended_cell_style(0, 1, 1)
        .unwrap()
        .style
        .fill
        .bg_color
        .is_none());
}

// ---------------------------------------------------------------------------
// Undo / redo — delete
// ---------------------------------------------------------------------------

#[test]
fn test_undo_delete_restores_rule() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();
    model.delete_conditional_formatting(0, 0).unwrap();
    model.undo().unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].range, "A1:A5");
    assert!(matches!(list[0].cf_rule, CfRule::ColorScale { .. }));
}

#[test]
fn test_redo_delete_removes_rule_again() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();
    model.delete_conditional_formatting(0, 0).unwrap();
    model.undo().unwrap();
    model.redo().unwrap();
    assert!(model
        .get_conditional_formatting_list(0)
        .unwrap()
        .is_empty());
}

#[test]
fn test_undo_delete_restores_applied_style() {
    let mut model = new_empty_user_model();
    for i in 1i32..=5 {
        model.set_user_input(0, i, 1, &i.to_string()).unwrap();
    }
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();
    model.delete_conditional_formatting(0, 0).unwrap();
    assert!(model
        .get_extended_cell_style(0, 1, 1)
        .unwrap()
        .style
        .fill
        .bg_color
        .is_none());

    model.undo().unwrap();
    assert!(model
        .get_extended_cell_style(0, 1, 1)
        .unwrap()
        .style
        .fill
        .bg_color
        .is_some());
}

// ---------------------------------------------------------------------------
// Undo / redo — update
// ---------------------------------------------------------------------------

#[test]
fn test_undo_update_restores_old_range_and_rule() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();
    model
        .update_conditional_formatting(0, 0, "C1:C10", data_bar())
        .unwrap();
    model.undo().unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list[0].range, "A1:A5");
    assert!(matches!(list[0].cf_rule, CfRule::ColorScale { .. }));
}

#[test]
fn test_redo_update_reapplies_new_range_and_rule() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();
    model
        .update_conditional_formatting(0, 0, "C1:C10", data_bar())
        .unwrap();
    model.undo().unwrap();
    model.redo().unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list[0].range, "C1:C10");
    assert!(matches!(list[0].cf_rule, CfRule::DataBar { .. }));
}

#[test]
fn test_undo_update_preserves_priority() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();
    let priority_before = model.get_conditional_formatting_list(0).unwrap()[0].priority;
    model
        .update_conditional_formatting(0, 0, "C1:C10", data_bar())
        .unwrap();
    model.undo().unwrap();
    let priority_after = model.get_conditional_formatting_list(0).unwrap()[0].priority;
    assert_eq!(priority_before, priority_after);
}

// ---------------------------------------------------------------------------
// Multi-operation undo sequence
// ---------------------------------------------------------------------------

#[test]
fn test_undo_sequence_add_add_delete() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();
    model
        .add_conditional_formatting(0, "B1:B5", data_bar())
        .unwrap();
    model.delete_conditional_formatting(0, 0).unwrap();

    // Only B1:B5 remains
    assert_eq!(model.get_conditional_formatting_list(0).unwrap().len(), 1);

    // Undo delete → both rules back
    model.undo().unwrap();
    assert_eq!(model.get_conditional_formatting_list(0).unwrap().len(), 2);

    // Undo second add → only A1:A5
    model.undo().unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].range, "A1:A5");

    // Undo first add → empty
    model.undo().unwrap();
    assert!(model
        .get_conditional_formatting_list(0)
        .unwrap()
        .is_empty());
}

#[test]
fn test_cell_is_rule_undo_redo() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(
            0,
            "A1:A5",
            CfRule::CellIs {
                operator: ValueOperator::GreaterThan,
                formula: "3".to_string(),
                formula2: None,
                dxf_id: 0,
            },
        )
        .unwrap();
    assert!(matches!(
        model.get_conditional_formatting_list(0).unwrap()[0].cf_rule,
        CfRule::CellIs {
            operator: ValueOperator::GreaterThan,
            ..
        }
    ));

    model.undo().unwrap();
    assert!(model
        .get_conditional_formatting_list(0)
        .unwrap()
        .is_empty());

    model.redo().unwrap();
    assert_eq!(model.get_conditional_formatting_list(0).unwrap().len(), 1);
}
