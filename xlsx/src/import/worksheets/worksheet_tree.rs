use core::str;
use std::{collections::HashMap, io::Read};

use ironcalc_base::{
    expressions::utils::column_to_number,
    types::{Col, Row, SheetData, Table, Worksheet, WorksheetView},
};
use roxmltree::Node;

use crate::{error::XlsxError, import::util::get_number};

use super::{
    from_a1_to_rc, get_attribute, get_cell_from_excel, get_color, get_column_from_ref,
    get_formula_index, parse_cell_reference, parse_range, SheetSettings, SheetView,
};

fn load_dimension(ws: Node) -> String {
    // <dimension ref="A1:O18"/>
    let application_nodes = ws
        .children()
        .filter(|n| n.has_tag_name("dimension"))
        .collect::<Vec<Node>>();
    if application_nodes.len() == 1 {
        application_nodes[0]
            .attribute("ref")
            .unwrap_or("A1")
            .to_string()
    } else {
        "A1".to_string()
    }
}

fn load_columns(ws: Node) -> Result<Vec<Col>, XlsxError> {
    // cols
    // <cols>
    //     <col min="5" max="5" width="38.26953125" customWidth="1"/>
    //     <col min="6" max="6" width="9.1796875" style="1"/>
    //     <col min="8" max="8" width="4" customWidth="1"/>
    // </cols>
    let mut cols = Vec::new();
    let columns = ws
        .children()
        .filter(|n| n.has_tag_name("cols"))
        .collect::<Vec<Node>>();
    if columns.len() == 1 {
        for col in columns[0].children() {
            let min = get_attribute(&col, "min")?;
            let min = min.parse::<i32>()?;
            let max = get_attribute(&col, "max")?;
            let max = max.parse::<i32>()?;
            let width = get_attribute(&col, "width")?;
            let width = width.parse::<f64>()?;
            let custom_width = matches!(col.attribute("customWidth"), Some("1"));
            let style = col
                .attribute("style")
                .map(|s| s.parse::<i32>().unwrap_or(0));
            cols.push(Col {
                min,
                max,
                width,
                custom_width,
                style,
            })
        }
    }
    Ok(cols)
}

fn load_merge_cells(ws: Node) -> Result<Vec<String>, XlsxError> {
    // 18.3.1.55 Merge Cells
    // <mergeCells count="1">
    //    <mergeCell ref="K7:L10"/>
    // </mergeCells>
    let mut merge_cells = Vec::new();
    let merge_cells_nodes = ws
        .children()
        .filter(|n| n.has_tag_name("mergeCells"))
        .collect::<Vec<Node>>();
    if merge_cells_nodes.len() == 1 {
        for merge_cell in merge_cells_nodes[0].children() {
            let reference = get_attribute(&merge_cell, "ref")?.to_string();
            merge_cells.push(reference);
        }
    }
    Ok(merge_cells)
}

fn load_sheet_color(ws: Node) -> Result<Option<String>, XlsxError> {
    // <sheetPr>
    //     <tabColor theme="5" tint="-0.249977111117893"/>
    // </sheetPr>
    let mut color = None;
    let sheet_pr = ws
        .children()
        .filter(|n| n.has_tag_name("sheetPr"))
        .collect::<Vec<Node>>();
    if sheet_pr.len() == 1 {
        let tabs = sheet_pr[0]
            .children()
            .filter(|n| n.has_tag_name("tabColor"))
            .collect::<Vec<Node>>();
        if tabs.len() == 1 {
            color = get_color(tabs[0])?;
        }
    }
    Ok(color)
}

fn get_sheet_view(ws: Node) -> SheetView {
    // <sheetViews>
    //   <sheetView workbookViewId="0">
    //     <selection activeCell="E10" sqref="E10"/>
    //   </sheetView>
    // </sheetViews>
    // <sheetFormatPr defaultRowHeight="14.5" x14ac:dyDescent="0.35"/>

    // If we have frozen rows and columns:

    // <sheetView tabSelected="1" workbookViewId="0">
    //   <pane xSplit="3" ySplit="2" topLeftCell="D3" activePane="bottomRight" state="frozen"/>
    //   <selection pane="topRight" activeCell="D1" sqref="D1"/>
    //   <selection pane="bottomLeft" activeCell="A3" sqref="A3"/>
    //   <selection pane="bottomRight" activeCell="K16" sqref="K16"/>
    // </sheetView>

    // 18.18.52 ST_Pane (Pane Types)
    // bottomLeft, bottomRight, topLeft, topRight

    // NB: bottomLeft is used when only rows are frozen, etc
    // IronCalc ignores all those.

    let mut frozen_rows = 0;
    let mut frozen_columns = 0;

    // In IronCalc there can only be one sheetView
    let sheet_views = ws
        .children()
        .filter(|n| n.has_tag_name("sheetViews"))
        .collect::<Vec<Node>>();

    // We are only expecting one `sheetViews` element. Otherwise return a default
    if sheet_views.len() != 1 {
        return SheetView::default();
    }

    let sheet_view = sheet_views[0]
        .children()
        .filter(|n| n.has_tag_name("sheetView"))
        .collect::<Vec<Node>>();

    // We are only expecting one `sheetView` element. Otherwise return a default
    if sheet_view.len() != 1 {
        return SheetView::default();
    }

    let sheet_view = sheet_view[0];
    let is_selected = sheet_view.attribute("tabSelected").unwrap_or("0") == "1";
    let show_grid_lines = sheet_view.attribute("showGridLines").unwrap_or("1") == "1";

    let pane = sheet_view
        .children()
        .filter(|n| n.has_tag_name("pane"))
        .collect::<Vec<Node>>();

    // 18.18.53 ST_PaneState (Pane State)
    // frozen, frozenSplit, split
    if pane.len() == 1 {
        if let Some("frozen") = pane[0].attribute("state") {
            // TODO: Should we assert that topLeft is consistent?
            // let top_left_cell = pane[0].attribute("topLeftCell").unwrap_or("A1").to_string();

            frozen_columns = get_number(pane[0], "xSplit");
            frozen_rows = get_number(pane[0], "ySplit");
        }
    }
    let selections = sheet_view
        .children()
        .filter(|n| n.has_tag_name("selection"))
        .collect::<Vec<Node>>();

    if let Some(selection) = selections.last() {
        let active_cell = match selection.attribute("activeCell").map(parse_cell_reference) {
            Some(Ok(s)) => Some(s),
            _ => None,
        };
        let sqref = match selection.attribute("sqref").map(parse_range) {
            Some(Ok(s)) => Some(s),
            _ => None,
        };

        let (selected_row, selected_column, row1, column1, row2, column2) =
            match (active_cell, sqref) {
                (Some(cell), Some(range)) => (cell.0, cell.1, range.0, range.1, range.2, range.3),
                (Some(cell), None) => (cell.0, cell.1, cell.0, cell.1, cell.0, cell.1),
                (None, Some(range)) => (range.0, range.1, range.0, range.1, range.2, range.3),
                _ => (1, 1, 1, 1, 1, 1),
            };

        SheetView {
            frozen_rows,
            frozen_columns,
            selected_row,
            selected_column,
            is_selected,
            show_grid_lines,
            range: [row1, column1, row2, column2],
        }
    } else {
        SheetView::default()
    }
}

pub(super) fn load_sheet<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
    path: &str,
    settings: SheetSettings,
    worksheets: &[String],
    tables: &HashMap<String, Table>,
    shared_strings: &mut Vec<String>,
) -> Result<(Worksheet, bool), XlsxError> {
    let sheet_name = &settings.name;
    let sheet_id = settings.id;
    let state = &settings.state;

    let mut file = archive.by_name(path)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    let doc = roxmltree::Document::parse(&text)?;
    let ws = doc
        .root()
        .first_child()
        .ok_or_else(|| XlsxError::Xml("Corrupt XML structure".to_string()))?;
    let mut shared_formulas = Vec::new();

    let dimension = load_dimension(ws);

    let sheet_view = get_sheet_view(ws);

    let cols = load_columns(ws)?;
    let color = load_sheet_color(ws)?;

    // sheetData
    // <row r="1" spans="1:15" x14ac:dyDescent="0.35">
    //     <c r="A1" t="s">
    //         <v>0</v>
    //     </c>
    //     <c r="D1">
    //         <f>C1+1</f>
    //     </c>
    // </row>

    // holds the row heights
    let mut rows = Vec::new();
    let mut sheet_data = SheetData::new();
    let sheet_data_nodes = ws
        .children()
        .filter(|n| n.has_tag_name("sheetData"))
        .collect::<Vec<Node>>()[0];

    let default_row_height = 14.5;

    // holds a map from the formula index in Excel to the index in IronCalc
    let mut index_map = HashMap::new();
    for row in sheet_data_nodes.children() {
        // This is the row number 1-indexed
        let row_index = get_attribute(&row, "r")?.parse::<i32>()?;
        // `spans` is not used in IronCalc at the moment (it's an optimization)
        // let spans = row.attribute("spans");
        // This is the height of the row
        let has_height_attribute;
        let height = match row.attribute("ht") {
            Some(s) => {
                has_height_attribute = true;
                s.parse::<f64>().unwrap_or(default_row_height)
            }
            None => {
                has_height_attribute = false;
                default_row_height
            }
        };
        let custom_height = matches!(row.attribute("customHeight"), Some("1"));
        // The height of the row is always the visible height of the row
        // If custom_height is false that means the height was calculated automatically:
        // for example because a cell has many lines or a larger font

        let row_style = match row.attribute("s") {
            Some(s) => s.parse::<i32>().unwrap_or(0),
            None => 0,
        };
        let custom_format = matches!(row.attribute("customFormat"), Some("1"));
        let hidden = matches!(row.attribute("hidden"), Some("1"));

        if custom_height || custom_format || row_style != 0 || has_height_attribute || hidden {
            rows.push(Row {
                r: row_index,
                height,
                s: row_style,
                custom_height,
                custom_format,
                hidden,
            });
        }

        // Unused attributes:
        // * thickBot, thickTop, ph, collapsed, outlineLevel

        let mut data_row = HashMap::new();

        // 18.3.1.4 c (Cell)
        // Child Elements:
        // * v: Cell value
        // * is: Rich Text Inline (not used in IronCalc)
        // * f: Formula
        // Attributes:
        // r: reference. A1 style
        // s: style index
        // t: cell type
        // Unused attributes
        // cm (cell metadata), ph (Show Phonetic), vm (value metadata)
        for cell in row.children() {
            let cell_ref = get_attribute(&cell, "r")?;
            let column_letter = get_column_from_ref(cell_ref);
            let column: i32 = column_to_number(column_letter.as_str()).map_err(XlsxError::Xml)?;

            let value_metadata = cell.attribute("vm");

            // We check the value "v" child.
            let vs: Vec<Node> = cell.children().filter(|n| n.has_tag_name("v")).collect();
            let cell_value = if vs.len() == 1 {
                Some(vs[0].text().unwrap_or(""))
            } else {
                None
            };

            // type, the default type being "n" for number
            // If the cell does not have a value is an empty cell
            let cell_type = match cell.attribute("t") {
                Some(t) => t,
                None => {
                    if cell_value.is_none() {
                        "empty"
                    } else {
                        "n"
                    }
                }
            };

            // style index, the default style is 0
            let cell_style = match cell.attribute("s") {
                Some(s) => s.parse::<i32>().unwrap_or(0),
                None => 0,
            };

            // Check for formula
            // In Excel some formulas are shared and some are not, but in IronCalc all formulas are shared
            // A cell with a "non-shared" formula is like:
            // <c r="E3">
            //   <f>C2+1</f>
            //   <v>3</v>
            // </c>
            // A cell with a shared formula will be either a "mother" cell:
            // <c r="D2">
            //   <f t="shared" ref="D2:D3" si="0">C2+1</f>
            //   <v>3</v>
            // </c>
            // Or a "daughter" cell:
            // <c r="D3">
            //   <f t="shared" si="0"/>
            //   <v>4</v>
            // </c>
            // In IronCalc two cells have the same formula iff the R1C1 representation is the same
            // TODO: This algorithm could end up with "repeated" shared formulas
            //       We could solve that with a second transversal.
            let fs: Vec<Node> = cell.children().filter(|n| n.has_tag_name("f")).collect();
            let mut formula_index = -1;
            if fs.len() == 1 {
                // formula types:
                // 18.18.6 ST_CellFormulaType (Formula Type)
                // array (Array Formula) Formula is an array formula.
                // dataTable (Table Formula) Formula is a data table formula.
                // normal (Normal) Formula is a regular cell formula. (Default)
                // shared (Shared Formula) Formula is part of a shared formula.
                let formula_type = fs[0].attribute("t").unwrap_or("normal");
                match formula_type {
                    "shared" => {
                        // We have a shared formula
                        let si = get_attribute(&fs[0], "si")?;
                        let si = si.parse::<i32>()?;
                        match fs[0].attribute("ref") {
                            Some(_) => {
                                // It's the mother cell. We do not use the ref attribute in IronCalc
                                let formula = fs[0].text().unwrap_or("").to_string();
                                let context = format!("{}!{}", sheet_name, cell_ref);
                                let formula =
                                    from_a1_to_rc(formula, worksheets, context, tables.clone())?;
                                match index_map.get(&si) {
                                    Some(index) => {
                                        // The index for that formula already exists meaning we bumped into a daughter cell first
                                        // TODO: Worth assert the content is a placeholder?
                                        formula_index = *index;
                                        shared_formulas.insert(formula_index as usize, formula);
                                    }
                                    None => {
                                        // We haven't met any of the daughter cells
                                        match get_formula_index(&formula, &shared_formulas) {
                                            // The formula is already present, use that index
                                            Some(index) => {
                                                formula_index = index;
                                            }
                                            None => {
                                                shared_formulas.push(formula);
                                                formula_index = shared_formulas.len() as i32 - 1;
                                            }
                                        };
                                        index_map.insert(si, formula_index);
                                    }
                                }
                            }
                            None => {
                                // It's a daughter cell
                                match index_map.get(&si) {
                                    Some(index) => {
                                        formula_index = *index;
                                    }
                                    None => {
                                        // Haven't bumped into the mother cell yet. We insert a placeholder.
                                        // Note that it is perfectly possible that the formula of the mother cell
                                        // is already in the set of array formulas. This will lead to the above mention duplicity.
                                        // This is not a problem
                                        let placeholder = "".to_string();
                                        shared_formulas.push(placeholder);
                                        formula_index = shared_formulas.len() as i32 - 1;
                                        index_map.insert(si, formula_index);
                                    }
                                }
                            }
                        }
                    }
                    "array" => {
                        return Err(XlsxError::NotImplemented("array formulas".to_string()));
                    }
                    "dataTable" => {
                        return Err(XlsxError::NotImplemented("data table formulas".to_string()));
                    }
                    "normal" => {
                        // Its a cell with a simple formula
                        let formula = fs[0].text().unwrap_or("").to_string();
                        let context = format!("{}!{}", sheet_name, cell_ref);
                        let formula = from_a1_to_rc(formula, worksheets, context, tables.clone())?;

                        match get_formula_index(&formula, &shared_formulas) {
                            Some(index) => formula_index = index,
                            None => {
                                shared_formulas.push(formula);
                                formula_index = shared_formulas.len() as i32 - 1;
                            }
                        }
                    }
                    _ => {
                        return Err(XlsxError::Xml(format!(
                            "Invalid formula type {:?}.",
                            formula_type,
                        )));
                    }
                }
            }
            let cell = get_cell_from_excel(
                cell_value,
                value_metadata,
                cell_type,
                cell_style,
                formula_index,
                sheet_name,
                cell_ref,
                shared_strings,
            );
            data_row.insert(column, cell);
        }
        sheet_data.insert(row_index, data_row);
    }

    let merge_cells = load_merge_cells(ws)?;

    // Conditional Formatting
    // <conditionalFormatting sqref="B1:B9">
    //     <cfRule type="colorScale" priority="1">
    //         <colorScale>
    //             <cfvo type="min"/>
    //             <cfvo type="max"/>
    //             <color rgb="FFF8696B"/>
    //             <color rgb="FFFCFCFF"/>
    //         </colorScale>
    //     </cfRule>
    // </conditionalFormatting>
    // pageSetup
    // <pageSetup orientation="portrait" r:id="rId1"/>

    let mut views = HashMap::new();
    views.insert(
        0,
        WorksheetView {
            row: sheet_view.selected_row,
            column: sheet_view.selected_column,
            range: sheet_view.range,
            top_row: 1,
            left_column: 1,
        },
    );

    Ok((
        Worksheet {
            dimension,
            cols,
            rows,
            shared_formulas,
            sheet_data,
            name: sheet_name.to_string(),
            sheet_id,
            state: state.to_owned(),
            color,
            merge_cells,
            comments: settings.comments,
            frozen_rows: sheet_view.frozen_rows,
            frozen_columns: sheet_view.frozen_columns,
            show_grid_lines: sheet_view.show_grid_lines,
            views,
        },
        sheet_view.is_selected,
    ))
}
