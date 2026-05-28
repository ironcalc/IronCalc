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
                stop_if_true: false,
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
    // 5-star rating. Thresholds stored lowest-first (as imported from XLSX):
    //   [0, 20, 40, 60, 80]. count = number of thresholds the value exceeds.
    // v=50 exceeds [0, 20, 40] → count = 3.
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
                    (Cfvo::Number(0.0), true),  // >= 0  → count ≥ 1
                    (Cfvo::Number(20.0), true), // >= 20 → count ≥ 2
                    (Cfvo::Number(40.0), true), // >= 40 → count ≥ 3
                    (Cfvo::Number(60.0), true), // >= 60 → count ≥ 4
                    (Cfvo::Number(80.0), true), // >= 80 → count = 5
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
    // A1=1 … A5=5. Thresholds lowest-first: [1, 2, 3, 4, 5].
    // count = number of thresholds the value meets (>=), so Ai → count = i.
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
                    (Cfvo::Number(1.0), true),
                    (Cfvo::Number(2.0), true),
                    (Cfvo::Number(3.0), true),
                    (Cfvo::Number(4.0), true),
                    (Cfvo::Number(5.0), true),
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
    // Thresholds lowest-first (is_strict=true, >=): [1, 2, 3, 4, 5] → max = 5.
    // A1=7   → exceeds all 5 thresholds → count = 5 (= max, naturally clamped)
    // A2=-1  → exceeds none             → count = 0
    // A3=2.7 → 2.7>=1 and 2.7>=2, but 2.7<3 → count = 2
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
                    (Cfvo::Number(1.0), true),
                    (Cfvo::Number(2.0), true),
                    (Cfvo::Number(3.0), true),
                    (Cfvo::Number(4.0), true),
                    (Cfvo::Number(5.0), true),
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
    assert_eq!(r1.count, 5, "7 exceeds all 5 thresholds → max (5) stars");
    assert_eq!(r1.max, 5);

    let r2 = model
        .get_extended_cell_style(0, 2, 1)
        .unwrap()
        .rating
        .unwrap();
    assert_eq!(r2.count, 0, "-1 is below all thresholds → 0 filled icons");

    let r3 = model
        .get_extended_cell_style(0, 3, 1)
        .unwrap()
        .rating
        .unwrap();
    assert_eq!(r3.count, 2, "2.7 >= 1 and >= 2 but < 3 → 2 filled icons");
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
                stop_if_true: false,
            },
        )
        .unwrap();
}

// ---------------------------------------------------------------------------
// Cut-and-paste: CF range and formula reference updates
// ---------------------------------------------------------------------------

#[test]
fn cut_paste_updates_cf_range_fully_inside_moved_area() {
    let mut model = new_empty_user_model();
    // CF rule covers exactly the cells being cut: A1:A5
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();

    // Cut A1:A5 and paste at F11
    model.set_selected_range(1, 1, 5, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(11, 6).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 5, 1), &cp.data, true)
        .unwrap();

    // CF range should have moved to F11:F15
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].range, "F11:F15");
}

#[test]
fn cut_paste_cf_range_outside_moved_area_unchanged() {
    let mut model = new_empty_user_model();
    // CF rule covers C1:C5, which is NOT in the cut area
    model
        .add_conditional_formatting(0, "C1:C5", color_scale())
        .unwrap();

    // Cut A1:A3 and paste at F11
    model.set_selected_range(1, 1, 3, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(11, 6).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 3, 1), &cp.data, true)
        .unwrap();

    // CF range should be unchanged
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].range, "C1:C5");
}

#[test]
fn cut_paste_cf_range_partial_overlap_unchanged() {
    let mut model = new_empty_user_model();
    // CF rule covers A1:A5, but we only cut A1:A3 (partial overlap)
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();

    // Cut A1:A3 and paste at F11
    model.set_selected_range(1, 1, 3, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(11, 6).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 3, 1), &cp.data, true)
        .unwrap();

    // CF range should be unchanged (partial overlap → no update)
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].range, "A1:A5");
}

#[test]
fn cut_paste_updates_absolute_ref_in_cf_formula() {
    let mut model = new_empty_user_model();
    // CF rule on B1:B5 with formula that has an absolute ref to A1
    model
        .add_conditional_formatting(
            0,
            "B1:B5",
            CfRuleInput::Formula {
                formula: "=$A$1>5".to_string(),
                format: Dxf::default(),
                stop_if_true: false,
            },
        )
        .unwrap();

    // Cut A1 and paste at F11
    model.set_selected_range(1, 1, 1, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(11, 6).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 1, 1), &cp.data, true)
        .unwrap();

    // CF formula should reference F11 now
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 1);
    if let CfRule::Formula { formula, .. } = &list[0].cf_rule {
        assert_eq!(formula, "=$F$11>5");
    } else {
        panic!("Expected Formula CF rule");
    }
}

#[test]
fn cut_paste_undo_restores_cf_range() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();

    // Cut A1:A5 and paste at F11
    model.set_selected_range(1, 1, 5, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(11, 6).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 5, 1), &cp.data, true)
        .unwrap();

    assert_eq!(
        model.get_conditional_formatting_list(0).unwrap()[0].range,
        "F11:F15"
    );

    // Undo should restore original CF range
    model.undo().unwrap();
    assert_eq!(
        model.get_conditional_formatting_list(0).unwrap()[0].range,
        "A1:A5"
    );
}

// ---------------------------------------------------------------------------
// Copy-and-paste: CF rule duplication
// ---------------------------------------------------------------------------

#[test]
fn copy_paste_duplicates_cf_rule_for_pasted_range() {
    let mut model = new_empty_user_model();
    // CF rule exactly covering the cells to copy
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();

    // Copy A1:A5, paste at F11
    model.set_selected_range(1, 1, 5, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(11, 6).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 5, 1), &cp.data, false)
        .unwrap();

    // Should now have two CF rules: original + new for pasted area
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 2);
    let ranges: Vec<&str> = list.iter().map(|cf| cf.range.as_str()).collect();
    assert!(ranges.contains(&"A1:A5"), "original CF range missing");
    assert!(ranges.contains(&"F11:F15"), "pasted CF range missing");
}

#[test]
fn copy_paste_original_cf_rule_unchanged() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();

    model.set_selected_range(1, 1, 5, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(11, 6).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 5, 1), &cp.data, false)
        .unwrap();

    // Original CF rule should be unchanged
    let list = model.get_conditional_formatting_list(0).unwrap();
    let original = list.iter().find(|cf| cf.range == "A1:A5");
    assert!(original.is_some(), "original CF rule should still exist");
}

#[test]
fn copy_paste_no_cf_when_source_has_none() {
    let mut model = new_empty_user_model();
    // No CF rules at all

    model.set_selected_range(1, 1, 5, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(11, 6).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 5, 1), &cp.data, false)
        .unwrap();

    assert!(model.get_conditional_formatting_list(0).unwrap().is_empty());
}

#[test]
fn copy_paste_cf_intersection_with_copy_range() {
    let mut model = new_empty_user_model();
    // CF rule covers more than the copied range
    model
        .add_conditional_formatting(0, "A1:A10", color_scale())
        .unwrap();

    // Copy only A1:A5, paste at F11
    model.set_selected_range(1, 1, 5, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(11, 6).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 5, 1), &cp.data, false)
        .unwrap();

    // New rule should only cover F11:F15 (intersection A1:A5 mapped to target)
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 2);
    let new_range = list
        .iter()
        .find(|cf| cf.range != "A1:A10")
        .map(|cf| cf.range.as_str());
    assert_eq!(new_range, Some("F11:F15"));
}

#[test]
fn copy_paste_no_cf_when_rule_outside_copy_range() {
    let mut model = new_empty_user_model();
    // CF rule covers C1:C5, copied range is A1:A3 — no overlap
    model
        .add_conditional_formatting(0, "C1:C5", color_scale())
        .unwrap();

    model.set_selected_range(1, 1, 3, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(11, 6).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 3, 1), &cp.data, false)
        .unwrap();

    // Only the original CF rule, nothing added
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].range, "C1:C5");
}

#[test]
fn copy_paste_undo_removes_duplicated_cf_rule() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();

    model.set_selected_range(1, 1, 5, 1).unwrap();
    let cp = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(11, 6).unwrap();
    model
        .paste_from_clipboard(0, (1, 1, 5, 1), &cp.data, false)
        .unwrap();

    assert_eq!(model.get_conditional_formatting_list(0).unwrap().len(), 2);

    model.undo().unwrap();

    // Undo should remove the duplicated rule
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].range, "A1:A5");
}

// ---------------------------------------------------------------------------
// Row/column insert / delete / move: CF range updates
// ---------------------------------------------------------------------------

#[test]
fn insert_row_above_cf_range_shifts_down() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A5:A10", color_scale())
        .unwrap();
    model.insert_rows(0, 2, 1).unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list[0].range, "A6:A11");
}

#[test]
fn insert_row_inside_cf_range_expands() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();
    model.insert_rows(0, 3, 1).unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list[0].range, "A1:A6");
}

#[test]
fn insert_row_below_cf_range_unchanged() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();
    model.insert_rows(0, 10, 1).unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list[0].range, "A1:A5");
}

#[test]
fn delete_row_above_cf_range_shifts_up() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A5:A10", color_scale())
        .unwrap();
    model.delete_rows(0, 2, 1).unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list[0].range, "A4:A9");
}

#[test]
fn delete_row_inside_cf_range_contracts() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:A5", color_scale())
        .unwrap();
    model.delete_rows(0, 3, 1).unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list[0].range, "A1:A4");
}

#[test]
fn insert_column_left_of_cf_range_shifts_right() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "C1:C5", color_scale())
        .unwrap();
    model.insert_columns(0, 1, 1).unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list[0].range, "D1:D5");
}

#[test]
fn insert_column_inside_cf_range_expands() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:E5", color_scale())
        .unwrap();
    model.insert_columns(0, 3, 1).unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list[0].range, "A1:F5");
}

#[test]
fn delete_column_inside_cf_range_contracts() {
    let mut model = new_empty_user_model();
    model
        .add_conditional_formatting(0, "A1:E5", color_scale())
        .unwrap();
    model.delete_columns(0, 3, 1).unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list[0].range, "A1:D5");
}

#[test]
fn move_row_up_shifts_intermediate_cf_range() {
    let mut model = new_empty_user_model();
    // CF on A1:A3; move row 5 up to row 1 (delta=-4).
    // Rows 1–4 are intermediate and each shift down by 1: A1:A3 → A2:A4.
    model
        .add_conditional_formatting(0, "A1:A3", color_scale())
        .unwrap();
    model.move_rows_action(0, 5, 1, -4).unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list[0].range, "A2:A4");
}

#[test]
fn move_column_right_shifts_intermediate_cf_range() {
    let mut model = new_empty_user_model();
    // CF on C1:C5 (col 3); move col B (col 2) right to col D (delta=2).
    // Col C is intermediate and shifts left to col B: "C1:C5" → "B1:B5".
    model
        .add_conditional_formatting(0, "C1:C5", color_scale())
        .unwrap();
    model.move_columns_action(0, 2, 1, 2).unwrap();
    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list[0].range, "B1:B5");
}
