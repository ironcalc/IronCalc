use crate::export::styles_util::{get_alignment, get_border_xml, get_color_xml, get_fill_xml};
use ironcalc_base::types::{Dxf, DxfFont, Fill, NumFmt, Styles};

use super::escape::escape_xml;

fn get_dxf_font_xml(font: &DxfFont) -> String {
    let mut parts: Vec<String> = Vec::new();
    if font.b == Some(true) {
        parts.push("<b/>".to_string());
    }
    if font.i == Some(true) {
        parts.push("<i/>".to_string());
    }
    if font.u == Some(true) {
        parts.push("<u/>".to_string());
    }
    if font.strike == Some(true) {
        parts.push("<strike/>".to_string());
    }
    if let Some(sz) = font.sz {
        parts.push(format!("<sz val=\"{sz}\"/>"));
    }
    if let Some(color) = &font.color {
        parts.push(format!(
            "<color rgb=\"FF{}\"/>",
            color.trim_start_matches('#')
        ));
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
    let mut parts: Vec<String> = Vec::new();
    if let Some(font) = &dxf.font {
        let inner = get_dxf_font_xml(font);
        if !inner.is_empty() {
            parts.push(format!("<font>{inner}</font>"));
        }
    }
    if let Some(fill) = &dxf.fill {
        parts.push(get_fill_xml(fill));
    }
    if let Some(border) = &dxf.border {
        parts.push(get_border_xml(border));
    }
    if let Some(num_fmt) = &dxf.num_fmt {
        parts.push(get_dxf_num_fmt_xml(num_fmt));
    }
    if let Some(alignment) = &dxf.alignment {
        parts.push(get_alignment(alignment));
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
