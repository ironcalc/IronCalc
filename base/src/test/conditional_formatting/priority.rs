#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::{
    cf_types::{CfRuleInput, Cfvo, ColorScaleThreshold, Icon, IconThreshold, ValueOperator},
    test::util::new_empty_model,
    types::{Color, Dxf, Fill},
};

fn color_scale_rule() -> CfRuleInput {
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

fn data_bar_rule() -> CfRuleInput {
    CfRuleInput::DataBar {
        min: Some(Cfvo::Min),
        max: Some(Cfvo::Max),
        positive_color: "#0000FF".to_string(),
        negative_color: "#FF0000".to_string(),
        is_gradient: true,
        show_value: true,
    }
}

fn icon_set_rule() -> CfRuleInput {
    CfRuleInput::IconSet {
        thresholds: vec![
            IconThreshold {
                icon: Icon::ArrowDown,
                cfvo: Cfvo::Min,
                color: "#e43400".to_string(),
                is_strict: false,
            },
            IconThreshold {
                icon: Icon::ArrowRight,
                cfvo: Cfvo::Percent(33.0),
                color: "#ffeb84".to_string(),
                is_strict: false,
            },
            IconThreshold {
                icon: Icon::ArrowUp,
                cfvo: Cfvo::Percent(67.0),
                color: "#84cb1f".to_string(),
                is_strict: false,
            },
        ],
        show_value: true,
    }
}

fn model_with_values() -> crate::Model<'static> {
    let mut model = new_empty_model();
    for i in 1i32..=5 {
        model.set_user_input(0, i, 1, i.to_string()).unwrap();
    }
    model.evaluate();
    model
}

#[test]
fn test_priority() {
    let mut model = model_with_values();

    // First rule added → lower priority number → higher priority
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();

    let rule = CfRuleInput::CellIs {
        operator: ValueOperator::GreaterThan,
        formula: "3".to_string(),
        formula2: None,
        format: Dxf::default(),
        stop_if_true: false,
    };
    model.add_conditional_formatting(0, "A2:C2", rule).unwrap();

    model.evaluate();

    let list = model.get_conditional_formatting_list(0).unwrap();
    assert_eq!(list.len(), 2);
    // First-added rule (color scale) must have the lower priority number.
    assert!(
        list[0].priority < list[1].priority,
        "color scale (added first) should have lower priority number than CellIs"
    );

    // Color scale applies: A1 is the min (red) and A5 is the max (green).
    let style_a1 = model.get_extended_style_for_cell(0, 1, 1).unwrap();
    assert_eq!(
        style_a1.style.fill.color,
        Color::Rgb("#FF0000".to_string()),
        "A1 (min value) should be red from the color scale"
    );
    let style_a5 = model.get_extended_style_for_cell(0, 5, 1).unwrap();
    assert_eq!(
        style_a5.style.fill.color,
        Color::Rgb("#00FF00".to_string()),
        "A5 (max value) should be green from the color scale"
    );

    // A2 is in both ranges but its value (2) does not satisfy > 3, so only the
    // color scale applies; the background must be set (non-None).
    let style_a2 = model.get_extended_style_for_cell(0, 2, 1).unwrap();
    assert!(
        style_a2.style.fill.color.is_some(),
        "A2 should have a color-scale fill even though it is also in the CellIs range"
    );

    // B2 is inside the CellIs range but not the color scale range, and its
    // value is empty (0), which does not satisfy > 3, so no fill at all.
    let style_b2 = model.get_extended_style_for_cell(0, 2, 2).unwrap();
    assert!(
        style_b2.style.fill.color.is_none(),
        "B2 (empty cell in CellIs range) should have no fill"
    );
}

// A single cell covered simultaneously by a ColorScale, a DataBar, and an
// IconSet rule should carry all three decorations after evaluation.
#[test]
fn test_all_three_in_same_cell() {
    let mut model = model_with_values();

    // Priority 1 (highest): color scale
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    // Priority 2: data bar
    model
        .add_conditional_formatting(0, "A1:A5", data_bar_rule())
        .unwrap();
    // Priority 3 (lowest): icon set
    model
        .add_conditional_formatting(0, "A1:A5", icon_set_rule())
        .unwrap();

    model.evaluate();

    // A1 = 1 (min): color-scale red, data-bar value 0, arrow-down icon.
    let s1 = model.get_extended_style_for_cell(0, 1, 1).unwrap();
    assert_eq!(
        s1.style.fill.color,
        Color::Rgb("#FF0000".to_string()),
        "A1 should have color-scale red fill"
    );
    let db1 = s1.data_bar.expect("A1 should have a data bar");
    assert_eq!(db1.value, 0.0, "A1 data bar should be at 0 (min)");
    let icon1 = s1.icon.expect("A1 should have an icon");
    assert_eq!(icon1.icon, Icon::ArrowDown, "A1 should show ArrowDown");

    // A3 = 3 (middle value): color-scale blends, data-bar at 50%, arrow-right.
    let s3 = model.get_extended_style_for_cell(0, 3, 1).unwrap();
    assert!(
        s3.style.fill.color.is_some(),
        "A3 should have a color-scale fill"
    );
    let db3 = s3.data_bar.expect("A3 should have a data bar");
    assert!(
        db3.value > 0.0 && db3.value < 1.0,
        "A3 data bar should be between 0 and 1"
    );
    let icon3 = s3.icon.expect("A3 should have an icon");
    assert_eq!(icon3.icon, Icon::ArrowRight, "A3 should show ArrowRight");

    // A5 = 5 (max): color-scale green, data-bar at 1, arrow-up icon.
    let s5 = model.get_extended_style_for_cell(0, 5, 1).unwrap();
    assert_eq!(
        s5.style.fill.color,
        Color::Rgb("#00FF00".to_string()),
        "A5 should have color-scale green fill"
    );
    let db5 = s5.data_bar.expect("A5 should have a data bar");
    assert_eq!(db5.value, 1.0, "A5 data bar should be at 1 (max)");
    let icon5 = s5.icon.expect("A5 should have an icon");
    assert_eq!(icon5.icon, Icon::ArrowUp, "A5 should show ArrowUp");
}

// The rule added LAST gets the highest priority number and must win over
// the rule added first when both cover the same cell and type.
#[test]
fn test_higher_priority_number_wins() {
    let mut model = model_with_values();

    // Rule 1: added first → priority=1 (lower, less important). Red scale.
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();
    // Rule 2: added second → priority=2 (higher, more important). Blue scale.
    model
        .add_conditional_formatting(
            0,
            "A1:A5",
            CfRuleInput::ColorScale {
                thresholds: vec![
                    ColorScaleThreshold {
                        cfvo: Cfvo::Min,
                        color: "#0000FF".to_string(),
                    },
                    ColorScaleThreshold {
                        cfvo: Cfvo::Max,
                        color: "#FFFF00".to_string(),
                    },
                ],
            },
        )
        .unwrap();
    model.evaluate();

    // Rule 2 has the higher priority number and must win: A1 (min) should be blue.
    let style = model.get_extended_style_for_cell(0, 1, 1).unwrap();
    assert_eq!(style.style.fill.color, Color::Rgb("#0000FF".to_string()));
}

// Same as above but for icon sets.
#[test]
fn test_higher_priority_icon_set_wins() {
    let mut model = model_with_values();

    // Rule 1 (priority=1, lower): arrows.
    model
        .add_conditional_formatting(0, "A1:A5", icon_set_rule())
        .unwrap();
    // Rule 2 (priority=2, higher): flags — should win.
    model
        .add_conditional_formatting(
            0,
            "A1:A5",
            CfRuleInput::IconSet {
                thresholds: vec![
                    IconThreshold {
                        icon: Icon::Flag,
                        cfvo: Cfvo::Min,
                        color: "#f8696b".to_string(),
                        is_strict: false,
                    },
                    IconThreshold {
                        icon: Icon::Flag,
                        cfvo: Cfvo::Percent(33.0),
                        color: "#ffeb84".to_string(),
                        is_strict: false,
                    },
                    IconThreshold {
                        icon: Icon::Flag,
                        cfvo: Cfvo::Percent(67.0),
                        color: "#63be7b".to_string(),
                        is_strict: false,
                    },
                ],
                show_value: true,
            },
        )
        .unwrap();
    model.evaluate();

    // Rule 2 must win: A1 (min value) should show a Flag, not an ArrowDown.
    let extended = model.get_extended_style_for_cell(0, 1, 1).unwrap();
    let icon = extended.icon.expect("A1 should have an icon");
    assert_eq!(icon.icon, Icon::Flag);
}

// ColorScale fill must respect priority ordering relative to Dxf fill.
// The rule processed last (highest priority number) wins for fill.
#[test]
fn test_color_scale_vs_dxf_fill_priority() {
    let blue_fill = Dxf {
        fill: Some(Fill {
            color: Color::Rgb("#0000FF".to_string()),
        }),
        ..Dxf::default()
    };

    // Case 1: ColorScale (priority=1, lower) vs Dxf fill (priority=2, higher).
    // The Dxf fill must win for cells that match both rules.
    {
        let mut model = model_with_values();
        model
            .add_conditional_formatting(0, "A1:A5", color_scale_rule())
            .unwrap();
        model
            .add_conditional_formatting(
                0,
                "A1:A5",
                CfRuleInput::CellIs {
                    operator: ValueOperator::GreaterThan,
                    formula: "0".to_string(),
                    formula2: None,
                    format: blue_fill.clone(),
                    stop_if_true: false,
                },
            )
            .unwrap();
        model.evaluate();

        // A1=1 matches both; Dxf (priority=2) must override the ColorScale fill.
        let style = model.get_extended_style_for_cell(0, 1, 1).unwrap();
        assert_eq!(
            style.style.fill.color,
            Color::Rgb("#0000FF".to_string()),
            "Dxf fill (higher priority) should override ColorScale"
        );
    }

    // Case 2: Dxf fill (priority=1, lower) vs ColorScale (priority=2, higher).
    // The ColorScale must win.
    {
        let mut model = model_with_values();
        model
            .add_conditional_formatting(
                0,
                "A1:A5",
                CfRuleInput::CellIs {
                    operator: ValueOperator::GreaterThan,
                    formula: "0".to_string(),
                    formula2: None,
                    format: blue_fill.clone(),
                    stop_if_true: false,
                },
            )
            .unwrap();
        model
            .add_conditional_formatting(0, "A1:A5", color_scale_rule())
            .unwrap();
        model.evaluate();

        // A1=1 (min): ColorScale (priority=2) sets red; must override Dxf blue.
        let style = model.get_extended_style_for_cell(0, 1, 1).unwrap();
        assert_eq!(
            style.style.fill.color,
            Color::Rgb("#FF0000".to_string()),
            "ColorScale (higher priority) should override Dxf fill"
        );
    }
}
