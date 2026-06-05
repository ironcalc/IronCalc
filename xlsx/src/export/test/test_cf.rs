use std::fs;

use ironcalc_base::{
    cf_types::{
        icon_set_icons, CfRule, CfRuleInput, Cfvo, ColorScaleThreshold, Icon, IconThreshold,
        PeriodType, TextOperator, ValueOperator,
    },
    types::{Dxf, DxfFont, Fill},
    Model,
};

use crate::export::save_to_icalc;
use crate::import::load_from_icalc;
use crate::{export::save_to_xlsx, import::load_from_xlsx};

pub fn new_empty_model<'a>() -> Model<'a> {
    Model::new_empty("model", "en", "UTC", "en").unwrap()
}

#[test]
fn test_values() {
    let mut model = new_empty_model();
    // numbers
    model
        .set_user_input(0, 1, 1, "123.456".to_string())
        .unwrap();
    // strings
    model
        .set_user_input(0, 2, 1, "Hello world!".to_string())
        .unwrap();
    model
        .set_user_input(0, 3, 1, "Hello world!".to_string())
        .unwrap();
    model
        .set_user_input(0, 4, 1, "你好世界！".to_string())
        .unwrap();
    // booleans
    model.set_user_input(0, 5, 1, "TRUE".to_string()).unwrap();
    model.set_user_input(0, 6, 1, "FALSE".to_string()).unwrap();
    // errors
    model
        .set_user_input(0, 7, 1, "#VALUE!".to_string())
        .unwrap();

    // noop
    model.evaluate();
    {
        let temp_file_name = "temp_file_test_values.xlsx";
        save_to_xlsx(&model, temp_file_name).unwrap();

        let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
        assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "123.456");
        assert_eq!(
            model.get_formatted_cell_value(0, 2, 1).unwrap(),
            "Hello world!"
        );
        assert_eq!(
            model.get_formatted_cell_value(0, 3, 1).unwrap(),
            "Hello world!"
        );
        assert_eq!(
            model.get_formatted_cell_value(0, 4, 1).unwrap(),
            "你好世界！"
        );
        assert_eq!(model.get_formatted_cell_value(0, 5, 1).unwrap(), "TRUE");
        assert_eq!(model.get_formatted_cell_value(0, 6, 1).unwrap(), "FALSE");
        assert_eq!(model.get_formatted_cell_value(0, 7, 1).unwrap(), "#VALUE!");

        fs::remove_file(temp_file_name).unwrap();
    }
    {
        let temp_file_name = "temp_file_test_values.ic";
        save_to_icalc(&model, temp_file_name).unwrap();

        let model = load_from_icalc(temp_file_name, "en").unwrap();
        assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "123.456");
        assert_eq!(
            model.get_formatted_cell_value(0, 2, 1).unwrap(),
            "Hello world!"
        );
        assert_eq!(
            model.get_formatted_cell_value(0, 3, 1).unwrap(),
            "Hello world!"
        );
        assert_eq!(
            model.get_formatted_cell_value(0, 4, 1).unwrap(),
            "你好世界！"
        );
        assert_eq!(model.get_formatted_cell_value(0, 5, 1).unwrap(), "TRUE");
        assert_eq!(model.get_formatted_cell_value(0, 6, 1).unwrap(), "FALSE");
        assert_eq!(model.get_formatted_cell_value(0, 7, 1).unwrap(), "#VALUE!");

        fs::remove_file(temp_file_name).unwrap();
    }
}

/// Helpers for building Dxf test values.
fn fill_dxf(color: &str) -> Dxf {
    Dxf {
        fill: Some(Fill {
            pattern_type: "solid".to_string(),
            color: Some(color.to_string()),
        }),
        ..Default::default()
    }
}

fn font_dxf(color: &str) -> Dxf {
    Dxf {
        font: Some(DxfFont {
            color: Some(color.to_string()),
            b: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Verify that a Dxf fill color round-trips correctly.
fn assert_dxf_fill(model: &Model, dxf_id: u32, expected_color: &str) {
    let dxf = model
        .workbook
        .styles
        .dxfs
        .get(dxf_id as usize)
        .expect("dxf not found");
    let color = dxf
        .fill
        .as_ref()
        .and_then(|f| f.color.as_deref())
        .unwrap_or("");
    assert_eq!(color, expected_color, "dxfId={dxf_id} fill mismatch");
}

#[test]
fn test_cf_round_trip() {
    let mut model = new_empty_model();

    // Populate some numeric data used by evaluated rules.
    for i in 1i32..=10 {
        model.set_user_input(0, i, 1, i.to_string()).unwrap();
    }
    model.evaluate();

    // 1. CellIs – greaterThan, with fill DXF
    model
        .add_conditional_formatting(
            0,
            "A1:A10",
            CfRuleInput::CellIs {
                operator: ValueOperator::GreaterThan,
                formula: "5".to_string(),
                formula2: None,
                format: fill_dxf("#FF0000"),
                stop_if_true: false,
            },
        )
        .unwrap();

    // 2. CellIs – between, with fill DXF
    model
        .add_conditional_formatting(
            0,
            "B1:B10",
            CfRuleInput::CellIs {
                operator: ValueOperator::Between,
                formula: "3".to_string(),
                formula2: Some("7".to_string()),
                format: fill_dxf("#00FF00"),
                stop_if_true: false,
            },
        )
        .unwrap();

    // 3. ColorScale – 2-color min/max
    model
        .add_conditional_formatting(
            0,
            "C1:C10",
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
            },
        )
        .unwrap();

    // 4. ColorScale – 3-color with percentile midpoint
    model
        .add_conditional_formatting(
            0,
            "D1:D10",
            CfRuleInput::ColorScale {
                thresholds: vec![
                    ColorScaleThreshold {
                        cfvo: Cfvo::Min,
                        color: "#FF0000".to_string(),
                    },
                    ColorScaleThreshold {
                        cfvo: Cfvo::Percentile(50.0),
                        color: "#FFFF00".to_string(),
                    },
                    ColorScaleThreshold {
                        cfvo: Cfvo::Max,
                        color: "#00FF00".to_string(),
                    },
                ],
            },
        )
        .unwrap();

    // 5. DataBar – gradient, auto min/max
    model
        .add_conditional_formatting(
            0,
            "E1:E10",
            CfRuleInput::DataBar {
                min: None,
                max: None,
                positive_color: "#638EC6".to_string(),
                negative_color: "#FF0000".to_string(),
                is_gradient: true,
                show_value: true,
            },
        )
        .unwrap();

    // 6. DataBar – solid, explicit min/max, hide value
    model
        .add_conditional_formatting(
            0,
            "F1:F10",
            CfRuleInput::DataBar {
                min: Some(Cfvo::Number(2.0)),
                max: Some(Cfvo::Number(8.0)),
                positive_color: "#FF555A".to_string(),
                negative_color: "#FF0000".to_string(),
                is_gradient: false,
                show_value: false,
            },
        )
        .unwrap();

    // 7. IconSet – 3TrafficLights1
    let tl_icons = icon_set_icons("3TrafficLights1").unwrap();
    model
        .add_conditional_formatting(
            0,
            "G1:G10",
            CfRuleInput::IconSet {
                thresholds: tl_icons
                    .into_iter()
                    .zip([
                        (Cfvo::Percent(0.0), true),
                        (Cfvo::Percent(33.0), true),
                        (Cfvo::Percent(67.0), true),
                    ])
                    .map(|((icon, color), (cfvo, is_strict))| IconThreshold {
                        icon,
                        cfvo,
                        color,
                        is_strict,
                    })
                    .collect(),
                show_value: true,
            },
        )
        .unwrap();

    // 8. IconSet – 5Arrows (more icons)
    let arrow_icons = icon_set_icons("5Arrows").unwrap();
    model
        .add_conditional_formatting(
            0,
            "H1:H10",
            CfRuleInput::IconSet {
                thresholds: arrow_icons
                    .into_iter()
                    .zip([
                        (Cfvo::Percent(0.0), true),
                        (Cfvo::Percent(20.0), true),
                        (Cfvo::Percent(40.0), true),
                        (Cfvo::Percent(60.0), true),
                        (Cfvo::Percent(80.0), true),
                    ])
                    .map(|((icon, color), (cfvo, is_strict))| IconThreshold {
                        icon,
                        cfvo,
                        color,
                        is_strict,
                    })
                    .collect(),
                show_value: false,
            },
        )
        .unwrap();

    // 9. IconRating – 3 stars
    model
        .add_conditional_formatting(
            0,
            "I1:I10",
            CfRuleInput::IconRating {
                icon: Icon::Star,
                color: "#FFD700".to_string(),
                thresholds: vec![
                    (Cfvo::Percent(0.0), true),
                    (Cfvo::Percent(33.0), true),
                    (Cfvo::Percent(67.0), true),
                ],
                show_value: true,
            },
        )
        .unwrap();

    // 10. IconRating – 5 quarters (circles)
    model
        .add_conditional_formatting(
            0,
            "J1:J10",
            CfRuleInput::IconRating {
                icon: Icon::Circle,
                color: "#FFD700".to_string(),
                thresholds: vec![
                    (Cfvo::Percent(0.0), true),
                    (Cfvo::Percent(20.0), true),
                    (Cfvo::Percent(40.0), true),
                    (Cfvo::Percent(60.0), true),
                    (Cfvo::Percent(80.0), true),
                ],
                show_value: false,
            },
        )
        .unwrap();

    // 11. DuplicateValues – with font DXF
    model
        .add_conditional_formatting(
            0,
            "K1:K10",
            CfRuleInput::DuplicateValues {
                format: font_dxf("#FF0000"),
                stop_if_true: false,
            },
        )
        .unwrap();

    // 12. UniqueValues
    model
        .add_conditional_formatting(
            0,
            "L1:L10",
            CfRuleInput::UniqueValues {
                format: fill_dxf("#C6EFCE"),
                stop_if_true: false,
            },
        )
        .unwrap();

    // 13. AboveAverage
    model
        .add_conditional_formatting(
            0,
            "M1:M10",
            CfRuleInput::AboveAverage {
                format: fill_dxf("#FFEB9C"),
                stop_if_true: false,
            },
        )
        .unwrap();

    // 14. BelowAverage
    model
        .add_conditional_formatting(
            0,
            "N1:N10",
            CfRuleInput::BelowAverage {
                format: fill_dxf("#FFC7CE"),
                stop_if_true: false,
            },
        )
        .unwrap();

    // 15. Top10 (count)
    model
        .add_conditional_formatting(
            0,
            "O1:O10",
            CfRuleInput::Top10 {
                rank: 3,
                percent: false,
                format: fill_dxf("#63BE7B"),
                stop_if_true: false,
            },
        )
        .unwrap();

    // 16. Bottom10 (percent)
    model
        .add_conditional_formatting(
            0,
            "P1:P10",
            CfRuleInput::Bottom10 {
                rank: 20,
                percent: true,
                format: fill_dxf("#F8696B"),
                stop_if_true: false,
            },
        )
        .unwrap();

    // 17. Text – contains
    model
        .add_conditional_formatting(
            0,
            "Q1:Q10",
            CfRuleInput::Text {
                operator: TextOperator::Contains,
                value: "hello".to_string(),
                format: fill_dxf("#FFFF00"),
                stop_if_true: false,
            },
        )
        .unwrap();

    // 18. Text – begins with
    model
        .add_conditional_formatting(
            0,
            "R1:R10",
            CfRuleInput::Text {
                operator: TextOperator::BeginsWith,
                value: "test".to_string(),
                format: fill_dxf("#0000FF"),
                stop_if_true: false,
            },
        )
        .unwrap();

    // 19. Text – ends with
    model
        .add_conditional_formatting(
            0,
            "S1:S10",
            CfRuleInput::Text {
                operator: TextOperator::EndsWith,
                value: ".txt".to_string(),
                format: Dxf::default(),
                stop_if_true: false,
            },
        )
        .unwrap();

    // 20. Text – does not contain
    model
        .add_conditional_formatting(
            0,
            "T1:T10",
            CfRuleInput::Text {
                operator: TextOperator::DoesNotContain,
                value: "error".to_string(),
                format: Dxf::default(),
                stop_if_true: false,
            },
        )
        .unwrap();

    // 21. Formula (expression)
    model
        .add_conditional_formatting(
            0,
            "U1:U10",
            CfRuleInput::Formula {
                formula: "=MOD(ROW(),2)=0".to_string(),
                format: fill_dxf("#E2EFDA"),
                stop_if_true: false,
            },
        )
        .unwrap();

    // 22. TimePeriod – today
    model
        .add_conditional_formatting(
            0,
            "V1:V10",
            CfRuleInput::TimePeriod {
                time_period: PeriodType::Today,
                date1: None,
                date2: None,
                format: fill_dxf("#FFFFCC"),
                stop_if_true: false,
            },
        )
        .unwrap();

    // 23. TimePeriod – last month
    model
        .add_conditional_formatting(
            0,
            "W1:W10",
            CfRuleInput::TimePeriod {
                time_period: PeriodType::LastMonth,
                date1: None,
                date2: None,
                format: fill_dxf("#CCE5FF"),
                stop_if_true: false,
            },
        )
        .unwrap();

    // 24. Blanks
    model
        .add_conditional_formatting(
            0,
            "X1:X10",
            CfRuleInput::Blanks {
                format: fill_dxf("#CCCCCC"),
                stop_if_true: false,
            },
        )
        .unwrap();

    // 25. NotBlanks
    model
        .add_conditional_formatting(
            0,
            "Y1:Y10",
            CfRuleInput::NotBlanks {
                format: fill_dxf("#FFFFFF"),
                stop_if_true: false,
            },
        )
        .unwrap();

    // 26. Errors
    model
        .add_conditional_formatting(
            0,
            "Z1:Z10",
            CfRuleInput::Errors {
                format: fill_dxf("#FF0000"),
                stop_if_true: false,
            },
        )
        .unwrap();

    // 27. NoErrors
    model
        .add_conditional_formatting(
            0,
            "AA1:AA10",
            CfRuleInput::NoErrors {
                format: fill_dxf("#00FF00"),
                stop_if_true: false,
            },
        )
        .unwrap();

    // stopIfTrue flag survives the round-trip
    model
        .add_conditional_formatting(
            0,
            "AB1:AB10",
            CfRuleInput::CellIs {
                operator: ValueOperator::Equal,
                formula: "0".to_string(),
                formula2: None,
                format: fill_dxf("#FF0000"),
                stop_if_true: true,
            },
        )
        .unwrap();

    model.evaluate();

    let temp_file = "temp_cf_round_trip.xlsx";
    save_to_xlsx(&model, temp_file).unwrap();
    let imported = load_from_xlsx(temp_file, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file).unwrap();

    let orig_cfs = &model.workbook.worksheets[0].conditional_formatting;
    let imp_cfs = &imported.workbook.worksheets[0].conditional_formatting;

    // Every original rule must survive.
    assert_eq!(
        orig_cfs.len(),
        imp_cfs.len(),
        "CF rule count mismatch: orig={} imported={}",
        orig_cfs.len(),
        imp_cfs.len()
    );

    // Build a lookup: range → imported rules (there may be multiple per range).
    use std::collections::HashMap;
    let mut by_range: HashMap<&str, Vec<&CfRule>> = HashMap::new();
    for cf in imp_cfs {
        by_range
            .entry(cf.range.as_str())
            .or_default()
            .push(&cf.cf_rule);
    }

    // Helper to find a rule with a matching discriminant for a given range.
    let rule_of_type = |range: &str, check: &dyn Fn(&CfRule) -> bool| {
        by_range
            .get(range)
            .and_then(|rules| rules.iter().find(|r| check(r)).copied())
    };

    // --- CellIs checks ---
    let r1 = rule_of_type("A1:A10", &|r| {
        matches!(
            r,
            CfRule::CellIs {
                operator: ValueOperator::GreaterThan,
                ..
            }
        )
    })
    .expect("CellIs/greaterThan not found");
    if let CfRule::CellIs {
        operator,
        formula,
        formula2,
        stop_if_true,
        ..
    } = r1
    {
        assert_eq!(*operator, ValueOperator::GreaterThan);
        assert_eq!(formula, "5");
        assert!(formula2.is_none());
        assert!(!stop_if_true);
    }

    let r2 = rule_of_type("B1:B10", &|r| {
        matches!(
            r,
            CfRule::CellIs {
                operator: ValueOperator::Between,
                ..
            }
        )
    })
    .expect("CellIs/between not found");
    if let CfRule::CellIs {
        operator,
        formula,
        formula2,
        ..
    } = r2
    {
        assert_eq!(*operator, ValueOperator::Between);
        assert_eq!(formula, "3");
        assert_eq!(formula2.as_deref(), Some("7"));
    }

    // --- ColorScale checks ---
    let r3 = rule_of_type("C1:C10", &|r| matches!(r, CfRule::ColorScale { .. }))
        .expect("ColorScale 2-color not found");
    if let CfRule::ColorScale { thresholds } = r3 {
        assert_eq!(thresholds.len(), 2);
        assert_eq!(thresholds[0].color, "#FF0000");
        assert!(matches!(thresholds[0].cfvo, Cfvo::Min));
        assert_eq!(thresholds[1].color, "#00FF00");
        assert!(matches!(thresholds[1].cfvo, Cfvo::Max));
    }

    let r4 = rule_of_type("D1:D10", &|r| matches!(r, CfRule::ColorScale { .. }))
        .expect("ColorScale 3-color not found");
    if let CfRule::ColorScale { thresholds } = r4 {
        assert_eq!(thresholds.len(), 3);
        assert_eq!(thresholds[1].color, "#FFFF00");
        assert!(matches!(thresholds[1].cfvo, Cfvo::Percentile(_)));
    }

    // --- DataBar checks ---
    let r5 = rule_of_type("E1:E10", &|r| matches!(r, CfRule::DataBar { .. }))
        .expect("DataBar gradient not found");
    if let CfRule::DataBar {
        min,
        max,
        positive_color,
        is_gradient,
        show_value,
        ..
    } = r5
    {
        assert!(min.is_none(), "auto-min should be None");
        assert!(max.is_none(), "auto-max should be None");
        assert_eq!(positive_color, "#638EC6");
        assert!(is_gradient, "should be gradient");
        assert!(show_value);
    }

    let r6 = rule_of_type("F1:F10", &|r| matches!(r, CfRule::DataBar { .. }))
        .expect("DataBar solid not found");
    if let CfRule::DataBar {
        min,
        max,
        positive_color,
        is_gradient,
        show_value,
        ..
    } = r6
    {
        assert!(matches!(min, Some(Cfvo::Number(n)) if (*n - 2.0).abs() < 1e-9));
        assert!(matches!(max, Some(Cfvo::Number(n)) if (*n - 8.0).abs() < 1e-9));
        assert_eq!(positive_color, "#FF555A");
        assert!(!is_gradient, "should be solid");
        assert!(!show_value);
    }

    // --- IconSet checks ---
    let r7 = rule_of_type("G1:G10", &|r| {
        matches!(r, CfRule::IconSet { thresholds, .. } if thresholds.len() == 3
            && thresholds[0].icon == Icon::Circle)
    })
    .expect("IconSet 3TrafficLights not found");
    if let CfRule::IconSet {
        thresholds,
        show_value,
    } = r7
    {
        assert_eq!(thresholds.len(), 3);
        assert_eq!(thresholds[0].icon, Icon::Circle);
        assert!(show_value);
    }

    let r8 = rule_of_type(
        "H1:H10",
        &|r| matches!(r, CfRule::IconSet { thresholds, .. } if thresholds.len() == 5),
    )
    .expect("IconSet 5Arrows not found");
    if let CfRule::IconSet {
        thresholds,
        show_value,
    } = r8
    {
        assert_eq!(thresholds.len(), 5);
        assert!(!show_value);
    }

    // --- IconRating checks ---
    let r9 = rule_of_type("I1:I10", &|r| {
        matches!(r, CfRule::IconRating { icon: Icon::Star, thresholds, .. } if thresholds.len() == 3)
    })
    .expect("IconRating 3Stars not found");
    if let CfRule::IconRating {
        icon,
        color,
        thresholds,
        show_value,
    } = r9
    {
        assert_eq!(*icon, Icon::Star);
        assert_eq!(color, "#FFD700");
        assert_eq!(thresholds.len(), 3);
        assert!(show_value);
    }

    let r10 = rule_of_type("J1:J10", &|r| {
        matches!(r, CfRule::IconRating { icon: Icon::Circle, thresholds, .. } if thresholds.len() == 5)
    })
    .expect("IconRating 5Quarters not found");
    if let CfRule::IconRating {
        icon, show_value, ..
    } = r10
    {
        assert_eq!(*icon, Icon::Circle);
        assert!(!show_value);
    }

    // --- DXF-based simple rules ---
    let r11 = rule_of_type("K1:K10", &|r| matches!(r, CfRule::DuplicateValues { .. }))
        .expect("DuplicateValues not found");
    if let CfRule::DuplicateValues { dxf_id, .. } = r11 {
        // DXF should have a bold red font
        let dxf = imported.workbook.styles.dxfs.get(*dxf_id as usize).unwrap();
        assert_eq!(
            dxf.font.as_ref().and_then(|f| f.color.as_deref()),
            Some("#FF0000")
        );
        assert_eq!(dxf.font.as_ref().and_then(|f| f.b), Some(true));
    }

    let r12 = rule_of_type("L1:L10", &|r| matches!(r, CfRule::UniqueValues { .. }))
        .expect("UniqueValues not found");
    if let CfRule::UniqueValues { dxf_id, .. } = r12 {
        assert_dxf_fill(&imported, *dxf_id, "#C6EFCE");
    }

    let r13 = rule_of_type("M1:M10", &|r| matches!(r, CfRule::AboveAverage { .. }))
        .expect("AboveAverage not found");
    if let CfRule::AboveAverage { dxf_id, .. } = r13 {
        assert_dxf_fill(&imported, *dxf_id, "#FFEB9C");
    }

    let r14 = rule_of_type("N1:N10", &|r| matches!(r, CfRule::BelowAverage { .. }))
        .expect("BelowAverage not found");
    if let CfRule::BelowAverage { dxf_id, .. } = r14 {
        assert_dxf_fill(&imported, *dxf_id, "#FFC7CE");
    }

    let r15 = rule_of_type("O1:O10", &|r| {
        matches!(
            r,
            CfRule::Top10 {
                percent: false,
                rank: 3,
                ..
            }
        )
    })
    .expect("Top10 count not found");
    if let CfRule::Top10 {
        rank,
        percent,
        dxf_id,
        ..
    } = r15
    {
        assert_eq!(*rank, 3);
        assert!(!percent);
        assert_dxf_fill(&imported, *dxf_id, "#63BE7B");
    }

    let r16 = rule_of_type("P1:P10", &|r| {
        matches!(
            r,
            CfRule::Bottom10 {
                percent: true,
                rank: 20,
                ..
            }
        )
    })
    .expect("Bottom10 percent not found");
    if let CfRule::Bottom10 {
        rank,
        percent,
        dxf_id,
        ..
    } = r16
    {
        assert_eq!(*rank, 20);
        assert!(percent);
        assert_dxf_fill(&imported, *dxf_id, "#F8696B");
    }

    // --- Text rules ---
    let r17 = rule_of_type("Q1:Q10", &|r| {
        matches!(r, CfRule::Text { operator: TextOperator::Contains, value, .. } if value == "hello")
    })
    .expect("Text/Contains not found");
    if let CfRule::Text {
        operator, value, ..
    } = r17
    {
        assert_eq!(*operator, TextOperator::Contains);
        assert_eq!(value, "hello");
    }

    let r18 = rule_of_type("R1:R10", &|r| {
        matches!(
            r,
            CfRule::Text {
                operator: TextOperator::BeginsWith,
                ..
            }
        )
    })
    .expect("Text/BeginsWith not found");
    if let CfRule::Text { value, .. } = r18 {
        assert_eq!(value, "test");
    }

    let r19 = rule_of_type("S1:S10", &|r| {
        matches!(
            r,
            CfRule::Text {
                operator: TextOperator::EndsWith,
                ..
            }
        )
    })
    .expect("Text/EndsWith not found");
    if let CfRule::Text { value, .. } = r19 {
        assert_eq!(value, ".txt");
    }

    let _r20 = rule_of_type("T1:T10", &|r| {
        matches!(
            r,
            CfRule::Text {
                operator: TextOperator::DoesNotContain,
                ..
            }
        )
    })
    .expect("Text/DoesNotContain not found");

    // --- Formula (expression) ---
    let r21 = rule_of_type("U1:U10", &|r| matches!(r, CfRule::Formula { .. }))
        .expect("Formula not found");
    if let CfRule::Formula { formula, .. } = r21 {
        // Exported without =, re-imported with = prefix
        assert!(
            formula.contains("MOD") && formula.contains("ROW"),
            "unexpected formula: {formula}"
        );
    }

    // --- TimePeriod ---
    let r22 = rule_of_type("V1:V10", &|r| {
        matches!(
            r,
            CfRule::TimePeriod {
                time_period: PeriodType::Today,
                ..
            }
        )
    })
    .expect("TimePeriod/Today not found");
    if let CfRule::TimePeriod { time_period, .. } = r22 {
        assert_eq!(*time_period, PeriodType::Today);
    }

    let r23 = rule_of_type("W1:W10", &|r| {
        matches!(
            r,
            CfRule::TimePeriod {
                time_period: PeriodType::LastMonth,
                ..
            }
        )
    })
    .expect("TimePeriod/LastMonth not found");
    if let CfRule::TimePeriod { time_period, .. } = r23 {
        assert_eq!(*time_period, PeriodType::LastMonth);
    }

    // --- Blanks / NotBlanks / Errors / NoErrors ---
    let _r24 =
        rule_of_type("X1:X10", &|r| matches!(r, CfRule::Blanks { .. })).expect("Blanks not found");
    let _r25 = rule_of_type("Y1:Y10", &|r| matches!(r, CfRule::NotBlanks { .. }))
        .expect("NotBlanks not found");
    let _r26 =
        rule_of_type("Z1:Z10", &|r| matches!(r, CfRule::Errors { .. })).expect("Errors not found");
    let _r27 = rule_of_type("AA1:AA10", &|r| matches!(r, CfRule::NoErrors { .. }))
        .expect("NoErrors not found");

    // --- stopIfTrue ---
    let r28 = rule_of_type("AB1:AB10", &|r| {
        matches!(
            r,
            CfRule::CellIs {
                operator: ValueOperator::Equal,
                ..
            }
        )
    })
    .expect("CellIs/equal with stopIfTrue not found");
    if let CfRule::CellIs { stop_if_true, .. } = r28 {
        assert!(stop_if_true, "stopIfTrue should be preserved");
    }
}

#[test]
fn test_cf_custom_icon_set_round_trip() {
    // Tests that icon sets with icons not matching any named Excel set are still
    // exported and imported back correctly.
    let mut model = new_empty_model();
    for i in 1i32..=10 {
        model.set_user_input(0, i, 1, i.to_string()).unwrap();
    }
    model.evaluate();

    // Mix icons from different sets (Star + ArrowUp + Cross) – no named set matches this.
    model
        .add_conditional_formatting(
            0,
            "A1:A10",
            CfRuleInput::IconSet {
                thresholds: vec![
                    IconThreshold {
                        icon: Icon::Star,
                        cfvo: Cfvo::Percent(0.0),
                        color: "#FFD700".to_string(),
                        is_strict: true,
                    },
                    IconThreshold {
                        icon: Icon::ArrowUp,
                        cfvo: Cfvo::Percent(50.0),
                        color: "#84cb1f".to_string(),
                        is_strict: true,
                    },
                    IconThreshold {
                        icon: Icon::Cross,
                        cfvo: Cfvo::Percent(80.0),
                        color: "#f8696b".to_string(),
                        is_strict: false,
                    },
                ],
                show_value: false,
            },
        )
        .unwrap();

    // IconRating with heart icon (no named set exists).
    model
        .add_conditional_formatting(
            0,
            "B1:B10",
            CfRuleInput::IconRating {
                icon: Icon::Heart,
                color: "#FF69B4".to_string(),
                thresholds: vec![
                    (Cfvo::Percent(0.0), true),
                    (Cfvo::Percent(33.0), true),
                    (Cfvo::Percent(67.0), true),
                ],
                show_value: true,
            },
        )
        .unwrap();

    // ThumbsUp icon. No exported named set Exists and we can't export rules with 2 icons
    model
        .add_conditional_formatting(
            0,
            "C1:C10",
            CfRuleInput::IconSet {
                thresholds: vec![
                    IconThreshold {
                        icon: Icon::ThumbsDown,
                        cfvo: Cfvo::Percent(0.0),
                        color: "#FF0000".to_string(),
                        is_strict: true,
                    },
                    IconThreshold {
                        icon: Icon::ThumbsUp,
                        cfvo: Cfvo::Percent(50.0),
                        color: "#00FF00".to_string(),
                        is_strict: true,
                    },
                ],
                show_value: true,
            },
        )
        .unwrap();

    model.evaluate();

    let temp_file = "temp_cf_custom_icons.xlsx";
    save_to_xlsx(&model, temp_file).unwrap();
    let imported = load_from_xlsx(temp_file, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file).unwrap();

    let imp_cfs = &imported.workbook.worksheets[0].conditional_formatting;

    // Only 2 rules will survive.
    assert_eq!(imp_cfs.len(), 2, "expected 2 rules, got {}", imp_cfs.len());

    // The mixed-icon set (A1:A10) should come back as an IconSet with 3 thresholds.
    let has_mixed = imp_cfs.iter().any(|cf| {
        cf.range == "A1:A10"
            && matches!(&cf.cf_rule, CfRule::IconSet { thresholds, show_value: false } if thresholds.len() == 3)
    });
    assert!(
        has_mixed,
        "custom mixed-icon set not found in imported model"
    );

    // The Heart rating (B1:B10) should come back as an IconSet (from x14 custom).
    let has_heart = imp_cfs
        .iter()
        .any(|cf| cf.range == "B1:B10" && matches!(&cf.cf_rule, CfRule::IconSet { .. }));
    assert!(has_heart, "heart rating icon not found in imported model");

    // The ThumbsUp/Down set (C1:C10) will be missed.
    let has_thumbs = imp_cfs.iter().any(|cf| {
        cf.range == "C1:C10"
            && matches!(&cf.cf_rule, CfRule::IconSet { thresholds, .. } if thresholds.len() == 2)
    });
    assert!(
        !has_thumbs,
        "thumbs up/down icon set found in imported model (and not expected)"
    );
}
