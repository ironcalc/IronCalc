use ironcalc_base::types::Workbook;

use super::{escape::escape_xml, xml_constants::XML_DECLARATION};

pub(crate) fn get_shared_strings_xml(model: &Workbook) -> String {
    let mut shared_strings: Vec<String> = vec![];
    let count = &model.shared_strings.len();
    let unique_count = &model.shared_strings.len();
    for (index, shared_string) in model.shared_strings.iter().enumerate() {
        // Some engines drop the leading and trailing whitespace of a `<t>` element
        // unless the element asks for it to be kept.
        let space = if needs_space_preserve(shared_string) {
            r#" xml:space="preserve""#
        } else {
            ""
        };
        // The furigana go after the text (18.4.6 rPh)
        let mut phonetic = String::new();
        if let Some(runs) = model.phonetic_runs.get(&(index as i32)) {
            for run in runs {
                phonetic.push_str(&format!(
                    "<rPh sb=\"{}\" eb=\"{}\"><t>{}</t></rPh>",
                    run.start,
                    run.end,
                    escape_xml(&run.text)
                ));
            }
        }
        shared_strings.push(format!(
            "<si><t{space}>{}</t>{phonetic}</si>",
            escape_xml(shared_string)
        ));
    }
    format!("{}\n\
      <sst xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" count=\"{count}\" uniqueCount=\"{unique_count}\">\
        {}\
      </sst>", XML_DECLARATION, shared_strings.join(""))
}

/// True if the string starts or ends with whitespace (including newlines),
/// which XML consumers are allowed to normalize away.
fn needs_space_preserve(s: &str) -> bool {
    s.starts_with(char::is_whitespace) || s.ends_with(char::is_whitespace)
}

#[cfg(test)]
mod tests {
    use super::needs_space_preserve;

    #[test]
    fn preserve_detection() {
        assert!(needs_space_preserve(" girl"));
        assert!(needs_space_preserve("girl "));
        assert!(needs_space_preserve("\n"));
        assert!(!needs_space_preserve("girl"));
        assert!(!needs_space_preserve(""));
        assert!(!needs_space_preserve("a b"));
    }
}
