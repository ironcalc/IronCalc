use ironcalc_base::types::DataValidation;

use super::escape::escape_xml;

fn bool_attr(name: &str, value: bool) -> String {
    if value {
        format!(r#" {name}="1""#)
    } else {
        String::new()
    }
}

fn string_attr(name: &str, value: &Option<String>) -> String {
    match value {
        Some(v) => format!(r#" {name}="{}""#, escape_xml(v)),
        None => String::new(),
    }
}

/// The `<dataValidations>` element for a worksheet, or "" when it has no rules.
/// Every rule goes in the main element (cross-sheet list sources included,
/// which Excel accepts there since Excel 2010).
pub(crate) fn get_data_validations_xml(rules: &[DataValidation]) -> String {
    if rules.is_empty() {
        return String::new();
    }
    let mut out = Vec::with_capacity(rules.len());
    for dv in rules {
        let ty = if dv.validation_type == "none" {
            String::new()
        } else {
            format!(r#" type="{}""#, escape_xml(&dv.validation_type))
        };
        let mut formulas = String::new();
        if let Some(f) = &dv.formula1 {
            formulas.push_str(&format!("<formula1>{}</formula1>", escape_xml(f)));
        }
        if let Some(f) = &dv.formula2 {
            formulas.push_str(&format!("<formula2>{}</formula2>", escape_xml(f)));
        }
        out.push(format!(
            r#"<dataValidation{ty}{}{}{}{}{}{}{}{}{}{} sqref="{}">{formulas}</dataValidation>"#,
            string_attr("errorStyle", &dv.error_style),
            string_attr("operator", &dv.operator),
            bool_attr("allowBlank", dv.allow_blank),
            bool_attr("showDropDown", dv.show_drop_down),
            bool_attr("showInputMessage", dv.show_input_message),
            bool_attr("showErrorMessage", dv.show_error_message),
            string_attr("errorTitle", &dv.error_title),
            string_attr("error", &dv.error),
            string_attr("promptTitle", &dv.prompt_title),
            string_attr("prompt", &dv.prompt),
            escape_xml(&dv.sqref),
        ));
    }
    format!(
        r#"<dataValidations count="{}">{}</dataValidations>"#,
        rules.len(),
        out.join("")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_rules_in_schema_attribute_order() {
        let rules = vec![
            DataValidation {
                sqref: "B2:B10".to_string(),
                validation_type: "list".to_string(),
                formula1: Some("\"Red,Green,Blue\"".to_string()),
                allow_blank: true,
                show_error_message: true,
                ..Default::default()
            },
            DataValidation {
                sqref: "C2".to_string(),
                validation_type: "custom".to_string(),
                formula1: Some("AND(C2>0,C2<B2)".to_string()),
                error_style: Some("warning".to_string()),
                error: Some("Must be < B".to_string()),
                ..Default::default()
            },
        ];
        assert_eq!(
            get_data_validations_xml(&rules),
            concat!(
                r#"<dataValidations count="2">"#,
                r#"<dataValidation type="list" allowBlank="1" showErrorMessage="1" sqref="B2:B10"><formula1>&quot;Red,Green,Blue&quot;</formula1></dataValidation>"#,
                r#"<dataValidation type="custom" errorStyle="warning" error="Must be &lt; B" sqref="C2"><formula1>AND(C2&gt;0,C2&lt;B2)</formula1></dataValidation>"#,
                r#"</dataValidations>"#
            )
        );
        assert_eq!(get_data_validations_xml(&[]), "");
    }
}
