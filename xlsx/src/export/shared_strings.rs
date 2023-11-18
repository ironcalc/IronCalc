use ironcalc_base::types::Workbook;

use super::{escape::escape_xml, xml_constants::XML_DECLARATION};

pub(crate) fn get_shared_strings_xml(model: &Workbook) -> String {
    let mut shared_strings: Vec<String> = vec![];
    let count = &model.shared_strings.len();
    let unique_count = &model.shared_strings.len();
    for shared_string in &model.shared_strings {
        shared_strings.push(format!("<si><t>{}</t></si>", escape_xml(shared_string)));
    }
    format!("{}\n\
      <sst xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" count=\"{count}\" uniqueCount=\"{unique_count}\">\
        {}\
      </sst>", XML_DECLARATION, shared_strings.join(""))
}
