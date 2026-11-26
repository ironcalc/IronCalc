use crate::export::styles_util::{get_alignment, get_border_xml, get_dxf_fill_xml};
use ironcalc_base::types::{Color, Dxf, DxfFont, NumFmt, Styles};

use super::escape::escape_xml;

fn get_dxf_font_xml(font: &DxfFont) -> String {
    // The order of the children is fixed by the schema (CT_Font):
    // b, i, strike, u, sz, color
    let mut parts: Vec<String> = Vec::new();
    if font.b == Some(true) {
        parts.push("<b/>".to_string());
    }
    if font.i == Some(true) {
        parts.push("<i/>".to_string());
    }
    if font.strike == Some(true) {
        parts.push("<strike/>".to_string());
    }
    if font.u == Some(true) {
        parts.push("<u/>".to_string());
    }
    if let Some(sz) = font.sz {
        parts.push(format!("<sz val=\"{sz}\"/>"));
    }
    match &font.color {
        Color::Rgb(s) => {
            parts.push(format!("<color rgb=\"FF{}\"/>", s.trim_start_matches('#')));
        }
        Color::Theme(idx, tint) => {
            if *tint == 0.0 {
                parts.push(format!("<color theme=\"{idx}\"/>"));
            } else {
                parts.push(format!("<color theme=\"{idx}\" tint=\"{}\"/>", tint));
            }
        }
        Color::None => {}
    }
    parts.join("")
}

fn get_dxf_num_fmt_xml(num_fmt: &NumFmt) -> String {
    let code = escape_xml(&num_fmt.format_code);
    format!(
        "<numFmt numFmtId=\"{}\" formatCode=\"{code}\"/>",
        num_fmt.num_fmt_id
    )
}

fn get_dxf_xml(dxf: &Dxf) -> String {
    // The order of the children is fixed by the schema (CT_Dxf):
    // font, numFmt, fill, alignment, protection, border
    let mut parts: Vec<String> = Vec::new();
    if let Some(font) = &dxf.font {
        let inner = get_dxf_font_xml(font);
        if !inner.is_empty() {
            parts.push(format!("<font>{inner}</font>"));
        }
    }
    if let Some(num_fmt) = &dxf.num_fmt {
        parts.push(get_dxf_num_fmt_xml(num_fmt));
    }
    if let Some(fill) = &dxf.fill {
        parts.push(get_dxf_fill_xml(fill));
    }
    if let Some(alignment) = &dxf.alignment {
        parts.push(get_alignment(alignment));
    }
    if let Some(border) = &dxf.border {
        parts.push(get_border_xml(border));
    }
    format!("<dxf>{}</dxf>", parts.join(""))
}

pub(crate) fn get_dxfs_xml(styles: &Styles) -> String {
    let dxfs = &styles.dxfs;
    let count = dxfs.len();
    if count == 0 {
        return "<dxfs count=\"0\"/>".to_string();
    }
    let inner: String = dxfs.iter().map(get_dxf_xml).collect();
    format!("<dxfs count=\"{count}\">{inner}</dxfs>")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use ironcalc_base::types::{Alignment, Border, Fill, HorizontalAlignment};

    fn positions(xml: &str, tags: &[&str]) -> Vec<usize> {
        tags.iter().map(|tag| xml.find(tag).unwrap()).collect()
    }

    fn full_dxf() -> Dxf {
        Dxf {
            font: Some(DxfFont {
                b: Some(true),
                i: Some(true),
                strike: Some(true),
                u: Some(true),
                sz: Some(11),
                color: Color::Rgb("#FF0000".to_string()),
            }),
            fill: Some(Fill {
                color: Color::Rgb("#00FF00".to_string()),
            }),
            border: Some(Border::default()),
            num_fmt: Some(NumFmt {
                num_fmt_id: 164,
                format_code: "0.00".to_string(),
            }),
            alignment: Some(Alignment {
                horizontal: HorizontalAlignment::Center,
                ..Default::default()
            }),
        }
    }

    // The schema (CT_Font) is a sequence: the children must come in this order
    // or strict readers (the Open XML SDK validator) refuse the file.
    #[test]
    fn dxf_font_children_follow_the_schema_order() {
        let xml = get_dxf_xml(&full_dxf());
        let tags = ["<b/>", "<i/>", "<strike/>", "<u/>", "<sz ", "<color "];
        assert!(positions(&xml, &tags).is_sorted(), "{xml}");
    }

    // Same for CT_Dxf: font, numFmt, fill, alignment, protection, border
    #[test]
    fn dxf_children_follow_the_schema_order() {
        let xml = get_dxf_xml(&full_dxf());
        let tags = ["<font>", "<numFmt ", "<fill>", "<alignment", "<border"];
        assert!(positions(&xml, &tags).is_sorted(), "{xml}");
    }
}
