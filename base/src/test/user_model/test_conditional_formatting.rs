#![allow(clippy::unwrap_used)]

use crate::{
    cf_types::{CfRule, CfRuleInput, Cfvo, ColorScaleThreshold, Icon, ValueOperator},
    test::user_model::util::new_empty_user_model,
    types::Dxf,
};

fn color_scale() -> CfRuleInput {
    CfRuleInput::ColorScale {
        thresholds: vec![
            ColorScaleThreshold {
                cfvo: Cfvo::Min,
                color: "#FF0000".to_string(),
            },
            ColorScaleThreshold {
                cfvo: Cfvo::Max,
                color: "#00FF00".to_string(),
            },
        ],
    }
}

fn data_bar() -> CfRuleInput {
    CfRuleInput::DataBar {
        min: Some(Cfvo::Min),
        max: Some(Cfvo::Max),
        positive_color: "#0000FF".to_string(),
        negative_color: "#FF0000".to_string(),
        is_gradient: true,
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
    assert!(model.get_conditional_formatting_list(0).unwrap().is_empty());
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
    assert!(model.get_conditional_formatting_list(0).unwrap().is_empty());
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
    assert!(model.get_conditional_formatting_list(0).unwrap().is_empty());
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
    assert!(model.get_conditional_formatting_list(0).unwrap().is_empty());
}

#[test]
fn test_cell_is_rule_undo_redo() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(
            0,
            "A1:A5",
            CfRuleInput::CellIs {
                operator: ValueOperator::GreaterThan,
                formula: "3".to_string(),
                formula2: None,
                format: Dxf::default(),
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
    assert!(model.get_conditional_formatting_list(0).unwrap().is_empty());

    model.redo().unwrap();
    assert_eq!(model.get_conditional_formatting_list(0).unwrap().len(), 1);
}

// ---------------------------------------------------------------------------
// Icon rating: count is determined by thresholds, not raw cell value
// ---------------------------------------------------------------------------

#[test]
fn test_icon_rating_uses_thresholds() {
    // 5-star rating with boundaries at [80, 60, 40, 20] (is_strict=true → >=).
    // A cell value of 50 falls in [40, 60): exceeds thresholds at 20 and 40 → count = 3.
    // The old (buggy) implementation did round(50).clamp(0,5) = 5.
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "50").unwrap();
    model
        .add_conditional_formatting(
            0,
            "A1",
            CfRuleInput::IconRating {
                icon: Icon::Star,
                color: "#FFD700".to_string(),
                show_value: true,
                thresholds: vec![
                    (Cfvo::Number(80.0), true), // >= 80 → count = 5
                    (Cfvo::Number(60.0), true), // >= 60 → count = 4
                    (Cfvo::Number(40.0), true), // >= 40 → count = 3
                    (Cfvo::Number(20.0), true), // >= 20 → count = 2
                ],
            },
        )
        .unwrap();

    let rating = model
        .get_extended_cell_style(0, 1, 1)
        .unwrap()
        .rating
        .unwrap();
    assert_eq!(rating.count, 3, "50 is in [40,60) → 3 filled icons");
    assert_eq!(rating.max, 5);
}

#[test]
fn test_icon_rating_count_from_cell_value() {
    let mut model = new_empty_user_model();
    // A1=1, A2=2, A3=3, A4=4, A5=5
    // Thresholds with is_strict=true (>=): value>=2→2, >=3→3, >=4→4, >=5→5, else 1.
    for i in 1i32..=5 {
        model.set_user_input(0, i, 1, &i.to_string()).unwrap();
    }
    model
        .add_conditional_formatting(
            0,
            "A1:A5",
            CfRuleInput::IconRating {
                icon: Icon::Star,
                color: "#FFD700".to_string(),
                show_value: true,
                thresholds: vec![
                    (Cfvo::Number(5.0), true),
                    (Cfvo::Number(4.0), true),
                    (Cfvo::Number(3.0), true),
                    (Cfvo::Number(2.0), true),
                ],
            },
        )
        .unwrap();

    for i in 1i32..=5 {
        let style = model.get_extended_cell_style(0, i, 1).unwrap();
        let rating = style.rating.unwrap();
        assert_eq!(
            rating.count, i as u32,
            "row {} count should equal its value",
            i
        );
        assert_eq!(rating.max, 5, "max should always be 5");
    }
}

#[test]
fn test_icon_rating_clamps_to_max() {
    let mut model = new_empty_user_model();
    // Thresholds (is_strict=true, >=): [5, 4, 3, 2] → 5 buckets.
    // A1=7  → exceeds all 4 thresholds → count=5 (clamped to max)
    // A2=-1 → exceeds none → count=1 (minimum)
    // A3=2.7 → 2.7>=2 but 2.7<3 → count=2
    model.set_user_input(0, 1, 1, "7").unwrap();
    model.set_user_input(0, 2, 1, "-1").unwrap();
    model.set_user_input(0, 3, 1, "2.7").unwrap();
    model
        .add_conditional_formatting(
            0,
            "A1:A3",
            CfRuleInput::IconRating {
                icon: Icon::Star,
                color: "#FFD700".to_string(),
                thresholds: vec![
                    (Cfvo::Number(5.0), true),
                    (Cfvo::Number(4.0), true),
                    (Cfvo::Number(3.0), true),
                    (Cfvo::Number(2.0), true),
                ],
                show_value: true,
            },
        )
        .unwrap();

    let r1 = model
        .get_extended_cell_style(0, 1, 1)
        .unwrap()
        .rating
        .unwrap();
    assert_eq!(r1.count, 5, "7 exceeds all thresholds, clamped to max 5");

    let r2 = model
        .get_extended_cell_style(0, 2, 1)
        .unwrap()
        .rating
        .unwrap();
    assert_eq!(
        r2.count, 1,
        "-1 is below all thresholds, minimum count is 1"
    );

    let r3 = model
        .get_extended_cell_style(0, 3, 1)
        .unwrap()
        .rating
        .unwrap();
    assert_eq!(r3.count, 2, "2.7 >= 2 but < 3 → falls in the second bucket");
}

#[test]
fn test_priority_of_overlapping_rules() {
    let mut model = new_empty_user_model();
    for i in 1i32..=5 {
        model.set_user_input(0, i, 1, &i.to_string()).unwrap();
    }
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();
    model
        .add_conditional_formatting(
            0,
            "A2:C2",
            CfRuleInput::CellIs {
                operator: ValueOperator::GreaterThan,
                formula: "3".to_string(),
                formula2: None,
                format: Dxf::default(),
            },
        )
        .unwrap();
}
