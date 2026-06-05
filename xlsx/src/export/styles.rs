use ironcalc_base::types::{Color, Styles, Workbook};

use crate::export::{
    dxfs_styles::get_dxfs_xml,
    styles_util::{get_alignment, get_border_xml, get_fill_xml},
};

use super::{escape::escape_xml, xml_constants::XML_DECLARATION};

fn get_fonts_xml(styles: &Styles) -> String {
    let fonts = &styles.fonts;
    let mut fonts_str: Vec<String> = vec![];
    for font in fonts {
        let size = format!("<sz val=\"{}\"/>", font.sz);
        let color = match &font.color {
            Color::Rgb(s) => format!("<color rgb=\"FF{}\"/>", s.trim_start_matches('#')),
            Color::Theme(idx, tint) => {
                if *tint == 0.0 {
                    format!("<color theme=\"{idx}\"/>")
                } else {
                    format!("<color theme=\"{idx}\" tint=\"{:.16}\"/>", tint)
                }
            }
            Color::None => "".to_string(),
        };
        let name = format!("<name val=\"{}\"/>", escape_xml(&font.name));
        let bold = if font.b { "<b/>" } else { "" };
        let italic = if font.i { "<i/>" } else { "" };
        let underline = if font.u { "<u/>" } else { "" };
        let strike = if font.strike { "<strike/>" } else { "" };
        let family = format!("<family val=\"{}\"/>", font.family);
        let scheme = format!("<scheme val=\"{}\"/>", font.scheme);
        fonts_str.push(format!(
            "<font>\
                {size}\
                {color}\
                {name}\
                {bold}\
                {italic}\
                {underline}\
                {strike}\
                {family}\
                {scheme}\
             </font>"
        ));
    }
    let font_count = fonts.len();
    format!(
        "<fonts count=\"{font_count}\">{}</fonts>",
        fonts_str.join("")
    )
}

fn get_fills_xml(styles: &Styles) -> String {
    let fills = &styles.fills;
    // The first two fills must be the default fills
    let mut fills_str = vec![
        "<fill><patternFill patternType=\"none\"/></fill>".to_string(),
        "<fill><patternFill patternType=\"gray125\"/></fill>".to_string(),
    ];
    let mut fill_count = 0;
    for fill in fills {
        if fill_count < 2 {
            fill_count += 1;
            continue;
        } else {
            fills_str.push(get_fill_xml(fill));
        }
        fill_count += 1;
    }
    let fill_count = fills.len();
    format!(
        "<fills count=\"{fill_count}\">{}</fills>",
        fills_str.join("")
    )
}

fn get_borders_xml(styles: &Styles) -> String {
    let borders = &styles.borders;
    let mut borders_str: Vec<String> = vec![];
    let border_count = borders.len();
    for border in borders {
        borders_str.push(get_border_xml(border));
    }
    format!(
        "<borders count=\"{border_count}\">{}</borders>",
        borders_str.join("")
    )
}

// <numFmts count="1">
//   <numFmt numFmtId="164" formatCode="##,#00;[Blue]\-\-#,##0"/>
// </numFmts>
fn get_cell_number_formats_xml(styles: &Styles) -> String {
    let num_fmts = &styles.num_fmts;
    let mut num_fmts_str: Vec<String> = vec![];
    let num_fmt_count = num_fmts.len();
    for num_fmt in num_fmts {
        let num_fmt_id = num_fmt.num_fmt_id;
        let format_code = &num_fmt.format_code;
        let format_code = escape_xml(format_code);
        num_fmts_str.push(format!(
            "<numFmt numFmtId=\"{num_fmt_id}\" formatCode=\"{format_code}\"/>"
        ));
    }
    if num_fmt_count == 0 {
        return "".to_string();
    }
    format!(
        "<numFmts count=\"{num_fmt_count}\">{}</numFmts>",
        num_fmts_str.join("")
    )
}

fn get_cell_style_xfs_xml(styles: &Styles) -> String {
    let cell_style_xfs = &styles.cell_style_xfs;
    let mut cell_style_str: Vec<String> = vec![];
    for cell_style_xf in cell_style_xfs {
        let border_id = cell_style_xf.border_id;
        let fill_id = cell_style_xf.fill_id;
        let font_id = cell_style_xf.font_id;
        let num_fmt_id = cell_style_xf.num_fmt_id;
        let apply_alignment_str = if !cell_style_xf.apply_alignment {
            r#" applyAlignment="0""#
        } else {
            ""
        };
        let apply_font_str = if !cell_style_xf.apply_font {
            r#" applyFont="0""#
        } else {
            ""
        };
        let apply_fill_str = if !cell_style_xf.apply_fill {
            r#" applyFill="0""#
        } else {
            ""
        };
        let apply_number_format_str = if !cell_style_xf.apply_number_format {
            r#" applyNumberFormat="0""#
        } else {
            ""
        };
        let apply_border_str = if !cell_style_xf.apply_border {
            r#" applyBorder="0""#
        } else {
            ""
        };
        cell_style_str.push(format!(
            "<xf \
              borderId=\"{border_id}\" \
              fillId=\"{fill_id}\" \
              fontId=\"{font_id}\" \
              numFmtId=\"{num_fmt_id}\"\
              {apply_alignment_str}\
              {apply_font_str}\
              {apply_fill_str}\
              {apply_number_format_str}\
              {apply_border_str}/>"
        ));
    }
    let style_count = cell_style_xfs.len();
    format!(
        "<cellStyleXfs count=\"{style_count}\">{}</cellStyleXfs>",
        cell_style_str.join("")
    )
}

fn get_cell_xfs_xml(styles: &Styles) -> String {
    let cell_xfs = &styles.cell_xfs;
    let mut cell_xfs_str: Vec<String> = vec![];
    for cell_xf in cell_xfs {
        let xf_id = cell_xf.xf_id;
        let border_id = cell_xf.border_id;
        let fill_id = cell_xf.fill_id;
        let font_id = cell_xf.font_id;
        let num_fmt_id = cell_xf.num_fmt_id;
        let quote_prefix_str = if cell_xf.quote_prefix {
            r#" quotePrefix="1""#
        } else {
            ""
        };
        let apply_alignment_str = if cell_xf.apply_alignment {
            r#" applyAlignment="1""#
        } else {
            ""
        };
        let apply_font_str = if cell_xf.apply_font {
            r#" applyFont="1""#
        } else {
            ""
        };
        let apply_fill_str = if cell_xf.apply_fill {
            r#" applyFill="1""#
        } else {
            ""
        };
        let apply_number_format_str = if cell_xf.apply_number_format {
            r#" applyNumberFormat="1""#
        } else {
            ""
        };
        let apply_border_str = if cell_xf.apply_border {
            r#" applyBorder="1""#
        } else {
            ""
        };
        let properties = format!(
            "xfId=\"{xf_id}\" \
                borderId=\"{border_id}\" \
                fillId=\"{fill_id}\" \
                fontId=\"{font_id}\" \
                numFmtId=\"{num_fmt_id}\"\
                {quote_prefix_str}\
                {apply_alignment_str}\
                {apply_font_str}\
                {apply_fill_str}\
                {apply_number_format_str}\
                {apply_border_str}"
        );
        if let Some(alignment) = &cell_xf.alignment {
            let alignment = get_alignment(alignment);
            cell_xfs_str.push(format!("<xf {properties}>{alignment}</xf>"));
        } else {
            cell_xfs_str.push(format!("<xf {properties}/>"));
        }
    }
    let style_count = cell_xfs.len();
    format!(
        "<cellXfs count=\"{style_count}\">{}</cellXfs>",
        cell_xfs_str.join("")
    )
}

// <cellStyle xfId="0" name="Normal" builtinId="0"/>
fn get_cell_styles_xml(styles: &Styles) -> String {
    let cell_styles = &styles.cell_styles;
    let mut cell_styles_str: Vec<String> = vec![];
    for cell_style in cell_styles {
        let xf_id = cell_style.xf_id;
        let name = &cell_style.name;
        let name = escape_xml(name);
        let builtin_id = cell_style.builtin_id;
        cell_styles_str.push(format!(
            "<cellStyle xfId=\"{xf_id}\" name=\"{name}\" builtinId=\"{builtin_id}\"/>"
        ));
    }
    let style_count = cell_styles.len();
    format!(
        "<cellStyles count=\"{style_count}\">{}</cellStyles>",
        cell_styles_str.join("")
    )
}

pub(crate) fn get_styles_xml(model: &Workbook) -> String {
    let styles = &model.styles;
    let fonts = get_fonts_xml(styles);
    let fills = get_fills_xml(styles);
    let borders = get_borders_xml(styles);
    let number_formats = get_cell_number_formats_xml(styles);
    let cell_style_xfs = get_cell_style_xfs_xml(styles);
    let cell_xfs = get_cell_xfs_xml(styles);
    let cell_styles = get_cell_styles_xml(styles);
    let dxfs = get_dxfs_xml(styles);

    format!(
        "{XML_DECLARATION}
<styleSheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">\
{number_formats}\
{fonts}\
{fills}\
{borders}\
{cell_style_xfs}\
{cell_xfs}\
{cell_styles}\
{dxfs}\
</styleSheet>"
    )
}
