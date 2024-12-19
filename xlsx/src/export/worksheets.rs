#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

//! # A note on shared formulas
//! Although both Excel and IronCalc uses shared formulas they are used in a slightly different way that cannot be mapped 1-1
//! In IronCalc _all_ formulas are shared and there is a list of shared formulas much like there is a list of shared strings.
//! In Excel the situation in more nuanced. A shared formula is shared amongst a rage of cells.
//! The top left cell would be the "mother" cell that would contain the shared formula:
//! <c r="F4" t="str">
//!    <f t="shared" ref="F4:F8" si="42">A4+C4</f>
//!    <v>123</v>
//! </c>
//! Cells in the range F4:F8 will then link to that formula like so:
//! <c r="F6">
//!   <f t="shared" si="42"/>
//!   <v>1</v>
//! </c>
//! Formula in F6 would then be 'A6+C6'
use std::collections::HashMap;

use itertools::Itertools;

use ironcalc_base::{
    expressions::{
        parser::{stringify::to_excel_string, Node},
        types::CellReferenceRC,
        utils::number_to_column,
    },
    types::{Cell, Worksheet},
};

use super::{escape::escape_xml, xml_constants::XML_DECLARATION};

fn get_cell_style_attribute(s: i32) -> String {
    if s == 0 {
        "".to_string()
    } else {
        format!(" s=\"{}\"", s)
    }
}

fn get_formula_attribute(
    sheet_name: String,
    row: i32,
    column: i32,
    parsed_formula: &Node,
) -> String {
    let cell_ref = CellReferenceRC {
        sheet: sheet_name,
        row,
        column,
    };
    let formula = &to_excel_string(parsed_formula, &cell_ref);
    escape_xml(formula).to_string()
}

pub(crate) fn get_worksheet_xml(
    worksheet: &Worksheet,
    parsed_formulas: &[Node],
    dimension: &str,
    is_sheet_selected: bool,
) -> String {
    let mut sheet_data_str: Vec<String> = vec![];
    let mut cols_str: Vec<String> = vec![];
    let mut merged_cells_str: Vec<String> = vec![];

    for col in &worksheet.cols {
        // <col min="4" max="4" width="12" customWidth="1"/>
        let min = col.min;
        let max = col.max;
        let width = col.width;
        let custom_width = i32::from(col.custom_width);
        let column_style = match col.style {
            Some(s) => format!(" style=\"{s}\""),
            None => "".to_string(),
        };
        cols_str.push(format!(
            "<col min=\"{min}\" max=\"{max}\" width=\"{width}\" customWidth=\"{custom_width}\"{column_style}/>"
        ));
    }

    // this is a bit of an overkill. A dictionary of the row styles by row_index
    let mut row_style_dict = HashMap::new();
    for row in &worksheet.rows {
        // {
        //     "height": 13,
        //     "r": 7,
        //     "custom_format": false,
        //     "custom_height": true,
        //     "s": 0
        //     "hidden": false,
        //   },
        row_style_dict.insert(row.r, row.clone());
    }

    for (row_index, row_data) in worksheet.sheet_data.iter().sorted_by_key(|x| x.0) {
        let mut row_data_str: Vec<String> = vec![];
        for (column_index, cell) in row_data.iter().sorted_by_key(|x| x.0) {
            let column_name = number_to_column(*column_index).unwrap();
            let cell_name = format!("{column_name}{row_index}");
            match cell {
                Cell::EmptyCell { s } => {
                    // they only hold the style
                    let style = get_cell_style_attribute(*s);
                    row_data_str.push(format!("<c r=\"{cell_name}\"{style}/>"));
                }
                Cell::BooleanCell { v, s } => {
                    // <c r="A8" t="b" s="1">
                    //     <v>1</v>
                    // </c>
                    let b = i32::from(*v);
                    let style = get_cell_style_attribute(*s);
                    row_data_str.push(format!(
                        "<c r=\"{cell_name}\" t=\"b\"{style}><v>{b}</v></c>"
                    ));
                }
                Cell::NumberCell { v, s } => {
                    // Normally the type number is left out. Example:
                    // <c r="C6" s="1">
                    //     <v>3</v>
                    // </c>
                    let style = get_cell_style_attribute(*s);
                    row_data_str.push(format!("<c r=\"{cell_name}\"{style}><v>{v}</v></c>"));
                }
                Cell::ErrorCell { ei, s } => {
                    let style = get_cell_style_attribute(*s);
                    row_data_str.push(format!(
                        "<c r=\"{cell_name}\" t=\"e\"{style}><v>{ei}</v></c>"
                    ));
                }
                Cell::SharedString { si, s } => {
                    // Example:
                    // <c r="A1" s="1" t="s">
                    //    <v>5</v>
                    // </c>
                    // Cell on A1 contains a string (t="s") of style="1". The string is the 6th in the list of shared strings
                    let style = get_cell_style_attribute(*s);
                    row_data_str.push(format!(
                        "<c r=\"{cell_name}\" t=\"s\"{style}><v>{si}</v></c>"
                    ));
                }
                Cell::CellFormula { f: _, s: _ } => {
                    panic!("Model needs to be evaluated before saving!");
                }
                Cell::CellFormulaBoolean { f, v, s } => {
                    // <c r="A4" t="b" s="3">
                    //   <f>ISTEXT(A5)</f>
                    //   <v>1</v>
                    // </c>
                    let style = get_cell_style_attribute(*s);

                    let formula = get_formula_attribute(
                        worksheet.get_name(),
                        *row_index,
                        *column_index,
                        &parsed_formulas[*f as usize],
                    );

                    let b = i32::from(*v);
                    row_data_str.push(format!(
                        "<c r=\"{cell_name}\" t=\"b\"{style}><f>{formula}</f><v>{b}</v></c>"
                    ));
                }
                Cell::CellFormulaNumber { f, v, s } => {
                    // Note again type is skipped
                    // <c r="C4" s="3">
                    //   <f>A5+C3</f>
                    //   <v>123</v>
                    // </c>

                    let formula = get_formula_attribute(
                        worksheet.get_name(),
                        *row_index,
                        *column_index,
                        &parsed_formulas[*f as usize],
                    );
                    let style = get_cell_style_attribute(*s);

                    row_data_str.push(format!(
                        "<c r=\"{cell_name}\"{style}><f>{formula}</f><v>{v}</v></c>"
                    ));
                }
                Cell::CellFormulaString { f, v, s } => {
                    // <c r="C6" t="str" s="5">
                    //   <f>CONCATENATE(A1, A2)</f>
                    //   <v>Hello world!</v>
                    // </c>
                    let formula = get_formula_attribute(
                        worksheet.get_name(),
                        *row_index,
                        *column_index,
                        &parsed_formulas[*f as usize],
                    );
                    let style = get_cell_style_attribute(*s);
                    let escaped_v = escape_xml(v);

                    row_data_str.push(format!(
                        "<c r=\"{cell_name}\" t=\"str\"{style}><f>{formula}</f><v>{escaped_v}</v></c>"
                    ));
                }
                Cell::CellFormulaError {
                    f,
                    ei,
                    s,
                    o: _,
                    m: _,
                } => {
                    // <c r="C6" t="e" s="4">
                    //   <f>A1/A3<f/>
                    //   <v>#DIV/0!</v>
                    // </c>
                    let formula = get_formula_attribute(
                        worksheet.get_name(),
                        *row_index,
                        *column_index,
                        &parsed_formulas[*f as usize],
                    );
                    let style = get_cell_style_attribute(*s);
                    row_data_str.push(format!(
                        "<c r=\"{cell_name}\" t=\"e\"{style}><f>{formula}</f><v>{ei}</v></c>"
                    ));
                }
            }
        }
        let row_style_str = match row_style_dict.get(row_index) {
            Some(row_style) => {
                let hidden_str = if row_style.hidden {
                    r#" hidden="1""#
                } else {
                    ""
                };
                format!(
                    r#" s="{}" ht="{}" customHeight="{}" customFormat="{}"{}"#,
                    row_style.s,
                    row_style.height,
                    i32::from(row_style.custom_height),
                    i32::from(row_style.custom_format),
                    hidden_str,
                )
            }
            None => "".to_string(),
        };
        sheet_data_str.push(format!(
            "<row r=\"{row_index}\"{row_style_str}>{}</row>",
            row_data_str.join("")
        ))
    }
    let sheet_data = sheet_data_str.join("");

    for merge_cell_ref in &worksheet.merge_cells {
        merged_cells_str.push(format!("<mergeCell ref=\"{merge_cell_ref}\"/>"))
    }
    let merged_cells_count = merged_cells_str.len();

    let cols = cols_str.join("");
    let cols = if cols.is_empty() {
        "".to_string()
    } else {
        format!("<cols>{cols}</cols>")
    };

    let tab_selected = if is_sheet_selected {
        " tabSelected=\"1\""
    } else {
        ""
    };

    let show_grid_lines = if !worksheet.show_grid_lines {
        " showGridLines=\"0\""
    } else {
        ""
    };

    let mut active_cell = "A1".to_string();
    let mut sqref = "A1".to_string();

    let views = &worksheet.views;
    if let Some(view) = views.get(&0) {
        let range = view.range;
        let row = view.row;
        let column = view.column;
        let column_name = number_to_column(column).unwrap_or("A".to_string());
        active_cell = format!("{column_name}{row}");

        let column_start = number_to_column(range[1]).unwrap_or("A".to_string());
        let column_end = number_to_column(range[3]).unwrap_or("A".to_string());
        if range[0] == range[2] && range[1] == range[3] {
            sqref = format!("{column_start}{}", range[0]);
        } else {
            sqref = format!("{}{}:{}{}", column_start, range[0], column_end, range[2]);
        }
    }

    let merge_cells_section = if merged_cells_count > 0 {
        format!(
            "<mergeCells count=\"{}\">{}</mergeCells>",
            merged_cells_count,
            merged_cells_str.join("")
        )
    } else {
        "".to_string()
    };

    format!(
        "{XML_DECLARATION}
<worksheet \
xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" \
xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">\
  <dimension ref=\"{dimension}\"/>\
  <sheetViews>\
    <sheetView workbookViewId=\"0\"{show_grid_lines}{tab_selected}>\
        <selection activeCell=\"{active_cell}\" sqref=\"{sqref}\"/>\
    </sheetView>\
  </sheetViews>\
  {cols}\
  <sheetData>\
  {sheet_data}\
  </sheetData>\
  {merge_cells_section}\
</worksheet>"
    )
}
