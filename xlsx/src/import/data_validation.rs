#![allow(clippy::unwrap_used)]

use ironcalc_base::types::DataValidation;

use super::xml::XmlNode;

// 18.3.1.33 dataValidations (Data Validations)
// <dataValidations count="1">
//   <dataValidation type="list" allowBlank="1" showInputMessage="1" showErrorMessage="1" sqref="B2:B10">
//     <formula1>"Red,Green,Blue"</formula1>
//   </dataValidation>
// </dataValidations>
//
// Excel 2010+ writes rules whose list source lives on another sheet in the
// worksheet's extLst instead:
// <extLst>
//   <ext uri="{CCE6A557-97BC-4b89-ADB6-D9C93CAAB3DF}" xmlns:x14="...">
//     <x14:dataValidations count="1" xmlns:xm="...">
//       <x14:dataValidation type="list" allowBlank="1">
//         <x14:formula1><xm:f>Lists!$A$1:$A$3</xm:f></x14:formula1>
//         <xm:sqref>B2:B10</xm:sqref>
//       </x14:dataValidation>
//     </x14:dataValidations>
//   </ext>
// </extLst>
// Both are read into the same list. They are written back in the main
// element, which accepts cross-sheet references since Excel 2010.

const VALIDATION_TYPES: &[&str] = &[
    "none",
    "whole",
    "decimal",
    "list",
    "date",
    "time",
    "textLength",
    "custom",
];

fn attr_bool(node: &XmlNode, name: &str) -> bool {
    matches!(node.attribute(name), Some("1") | Some("true"))
}

fn attr_string(node: &XmlNode, name: &str) -> Option<String> {
    node.attribute(name).map(|s| s.to_string())
}

/// The text of `<formulaN>` (main element) or `<x14:formulaN><xm:f>` (extension).
fn formula_text(rule: &XmlNode, name: &str) -> Option<String> {
    let node = rule.children().find(|n| n.has_tag_name(name))?;
    let text = match node.children().find(|n| n.has_tag_name("f")) {
        Some(f) => f.text(),
        None => node.text(),
    }?;
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

fn load_rule(rule: &XmlNode, sqref: String) -> Option<DataValidation> {
    let validation_type = rule.attribute("type").unwrap_or("none");
    if !VALIDATION_TYPES.contains(&validation_type) {
        return None;
    }
    Some(DataValidation {
        sqref,
        validation_type: validation_type.to_string(),
        operator: attr_string(rule, "operator"),
        formula1: formula_text(rule, "formula1"),
        formula2: formula_text(rule, "formula2"),
        allow_blank: attr_bool(rule, "allowBlank"),
        show_drop_down: attr_bool(rule, "showDropDown"),
        show_input_message: attr_bool(rule, "showInputMessage"),
        show_error_message: attr_bool(rule, "showErrorMessage"),
        error_style: attr_string(rule, "errorStyle"),
        error_title: attr_string(rule, "errorTitle"),
        error: attr_string(rule, "error"),
        prompt_title: attr_string(rule, "promptTitle"),
        prompt: attr_string(rule, "prompt"),
    })
}

pub(crate) fn load_data_validations(ws: &XmlNode) -> Vec<DataValidation> {
    let mut rules = Vec::new();
    for block in ws.children().filter(|n| n.has_tag_name("dataValidations")) {
        for rule in block
            .children()
            .filter(|n| n.has_tag_name("dataValidation"))
        {
            let sqref = rule.attribute("sqref").unwrap_or("").to_string();
            if sqref.is_empty() {
                continue;
            }
            if let Some(dv) = load_rule(rule, sqref) {
                rules.push(dv);
            }
        }
    }
    for ext_lst in ws.children().filter(|n| n.has_tag_name("extLst")) {
        for block in ext_lst
            .descendants()
            .filter(|n| n.has_tag_name("dataValidations"))
        {
            for rule in block
                .children()
                .filter(|n| n.has_tag_name("dataValidation"))
            {
                let sqref = rule
                    .children()
                    .find(|n| n.has_tag_name("sqref"))
                    .and_then(|n| n.text())
                    .unwrap_or("")
                    .to_string();
                if sqref.is_empty() {
                    continue;
                }
                if let Some(dv) = load_rule(rule, sqref) {
                    rules.push(dv);
                }
            }
        }
    }
    rules
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(xml: &str) -> Vec<DataValidation> {
        let ws = XmlNode::parse_str(xml).unwrap();
        load_data_validations(&ws)
    }

    #[test]
    fn reads_main_and_extension_rules() {
        let xml = r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:x14="http://schemas.microsoft.com/office/spreadsheetml/2009/9/main" xmlns:xm="http://schemas.microsoft.com/office/excel/2006/main">
  <sheetData/>
  <dataValidations count="2">
    <dataValidation type="list" allowBlank="1" showInputMessage="1" showErrorMessage="1" sqref="B2:B10"><formula1>"Red,Green,Blue"</formula1></dataValidation>
    <dataValidation type="whole" operator="between" errorStyle="warning" error="1 to 10" sqref="C2 D4:D5"><formula1>1</formula1><formula2>10</formula2></dataValidation>
  </dataValidations>
  <extLst><ext uri="{CCE6A557-97BC-4b89-ADB6-D9C93CAAB3DF}"><x14:dataValidations count="1"><x14:dataValidation type="list" allowBlank="1"><x14:formula1><xm:f>Lists!$A$1:$A$3</xm:f></x14:formula1><xm:sqref>E2:E10</xm:sqref></x14:dataValidation></x14:dataValidations></ext></extLst>
</worksheet>"#;
        let rules = parse(xml);
        assert_eq!(rules.len(), 3);
        assert_eq!(rules[0].validation_type, "list");
        assert_eq!(rules[0].formula1.as_deref(), Some("\"Red,Green,Blue\""));
        assert!(rules[0].allow_blank && rules[0].show_error_message);
        assert_eq!(rules[1].sqref, "C2 D4:D5");
        assert_eq!(rules[1].operator.as_deref(), Some("between"));
        assert_eq!(rules[1].formula2.as_deref(), Some("10"));
        assert_eq!(rules[1].error_style.as_deref(), Some("warning"));
        assert_eq!(rules[1].error.as_deref(), Some("1 to 10"));
        assert_eq!(rules[2].sqref, "E2:E10");
        assert_eq!(rules[2].formula1.as_deref(), Some("Lists!$A$1:$A$3"));
    }

    #[test]
    fn skips_unknown_types_and_rules_without_a_range() {
        let xml = r#"<worksheet><dataValidations><dataValidation type="bogus" sqref="A1"/><dataValidation type="list"><formula1>"a"</formula1></dataValidation></dataValidations></worksheet>"#;
        assert!(parse(xml).is_empty());
    }
}
