#![allow(clippy::unwrap_used)]

use crate::{
    cf_types::{CfRuleInput, Cfvo, ColorScaleThreshold, ValueOperator},
    test::util::new_empty_model,
    types::Dxf,
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

    // color scale
    model
        .add_conditional_formatting(0, "A1:A5", color_scale_rule())
        .unwrap();

    let rule = CfRuleInput::CellIs {
        operator: ValueOperator::GreaterThan,
        formula: "3".to_string(),
        formula2: None,
        format: Dxf::default(),
    };
    model.add_conditional_formatting(0, "A2:C2", rule).unwrap();
    //
}
