use crate::error::XlsxError;
use crate::import::colors::{self, get_indexed_color};
use quick_xml::events::{BytesStart, BytesText, Event};
use std::borrow::Cow;
use std::{
    collections::HashMap,
    io::{BufReader, Read},
};

use ironcalc_base::{
    expressions::utils::column_to_number,
    types::{Cell, Col, Row, SheetData, Table, Worksheet, WorksheetView},
};

use super::{
    from_a1_to_rc, get_cell_from_excel, get_column_from_ref, get_formula_index,
    parse_cell_reference, parse_range, SheetSettings, SheetView,
};

pub(super) fn load_sheet<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
    path: &str,
    settings: SheetSettings,
    worksheets: &[String],
    tables: &HashMap<String, Table>,
    shared_strings: &mut Vec<String>,
) -> Result<(Worksheet, bool), XlsxError> {
    let mut sheet_parser = SheetParser::new(settings, shared_strings, worksheets, tables);

    let zipfile = archive.by_name(path)?;
    let xmlfile = BufReader::new(zipfile);
    let mut xmlfile = quick_xml::Reader::from_reader(xmlfile);
    xmlfile.config_mut().expand_empty_elements = true;

    const BUF_SIZE: usize = 700;
    let mut buf = Vec::with_capacity(BUF_SIZE);
    loop {
        match xmlfile
            .read_event_into(&mut buf)
            .map_err(|e| XlsxError::Xml(e.to_string()))?
        {
            Event::Eof => break,
            event => sheet_parser.process(event)?,
        };
        buf.clear();
    }

    sheet_parser.worksheet()
}

#[derive(Debug)]
struct CellData {
    cell_value: Option<String>,
    value_metadata: Option<String>,
    cell_type: Option<String>,
    cell_style: i32,
    formula_index: i32,
    cell_ref: String,
    column: i32,

    formula_data: FormulaData,
}

impl CellData {
    // Performance optimization: cheaper than deallocating / re-allocating the entire struct.
    fn set_to_default_values(&mut self) {
        self.cell_value = Default::default();
        self.value_metadata = Default::default();
        self.cell_type = Default::default();
        self.cell_style = Default::default();
        self.formula_index = -1;
        self.cell_ref = Default::default();
        self.column = Default::default();
        self.formula_data = Default::default();
    }
}

impl Default for CellData {
    fn default() -> Self {
        // Custom default impl since formula_index needs to be -1 by default.
        Self {
            cell_value: Default::default(),
            value_metadata: Default::default(),
            cell_type: Default::default(),
            cell_style: Default::default(),
            formula_index: -1,
            cell_ref: Default::default(),
            column: Default::default(),
            formula_data: Default::default(),
        }
    }
}

#[derive(Debug, Default)]
struct FormulaData {
    formula_type: String,
    formula_si: Option<String>,
    formula_has_ref: bool,
    formula_text: String,
}

#[derive(Debug, Clone, PartialEq)]
enum ParseState {
    Start,
    Worksheet,
    SheetViews,
    SheetView,
    SheetPr,
    SheetData,
    Column,
    Row,
    Cell,
    Value,
    Formula,
    MergeCell,
    End,
}

struct SheetParser<'a> {
    settings: SheetSettings,
    worksheets: &'a [String],
    tables: &'a HashMap<String, Table>,
    shared_strings: &'a mut Vec<String>,

    state: ParseState,
    dimensions: Vec<String>,
    sheet_views: Vec<SheetView>,
    current_sheet_view: SheetView,
    colors: Vec<Option<String>>,
    sheet_data: SheetData,
    current_data_row: HashMap<i32, Cell>,
    current_cell_data: CellData,
    current_row_index: i32,
    rows: Vec<Row>,
    cols: Vec<Col>,
    shared_formulas: Vec<String>,
    merge_cells: Vec<String>,

    // holds a map from the formula index in Excel to the index in IronCalc
    index_map: HashMap<i32, i32>,
}

impl<'a> SheetParser<'a> {
    fn new(
        settings: SheetSettings,
        shared_strings: &'a mut Vec<String>,
        worksheets: &'a [String],
        tables: &'a HashMap<String, Table>,
    ) -> Self {
        Self {
            settings,
            worksheets,
            tables,
            state: ParseState::Start,
            dimensions: vec![],
            sheet_views: vec![],
            current_sheet_view: SheetView::default(),
            colors: vec![],
            sheet_data: SheetData::default(),
            current_data_row: HashMap::default(),
            current_cell_data: CellData::default(),
            shared_strings,
            current_row_index: 0,
            rows: vec![],
            cols: vec![],
            shared_formulas: vec![],
            merge_cells: vec![],
            index_map: HashMap::default(),
        }
    }

    fn load_dimension(&mut self, tag: BytesStart) -> Result<(), XlsxError> {
        // <dimension ref="A1:O18"/>
        if let Some(dimension) = get_optional_attribute_streaming(&tag, "ref")? {
            self.dimensions.push(dimension.to_string());
        }

        Ok(())
    }

    fn load_current_sheet_view_attributes(&mut self, tag: BytesStart) -> Result<(), XlsxError> {
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

        self.current_sheet_view.is_selected =
            get_optional_attribute_streaming(&tag, "tabSelected")?.unwrap_or("0".into()) == "1";

        self.current_sheet_view.show_grid_lines =
            get_optional_attribute_streaming(&tag, "showGridLines")?.unwrap_or("1".into()) == "1";

        Ok(())
    }

    fn load_current_sheet_view_pane(&mut self, tag: BytesStart) -> Result<(), XlsxError> {
        // 18.18.53 ST_PaneState (Pane State)
        // frozen, frozenSplit, split
        if let Some(state) = get_optional_attribute_streaming(&tag, "state")? {
            if state == "frozen" {
                // TODO: Should we assert that topLeft is consistent?
                // let top_left_cell = pane[0].attribute("topLeftCell").unwrap_or("A1").to_string();

                self.current_sheet_view.frozen_columns = get_number_streaming(&tag, "xSplit");
                self.current_sheet_view.frozen_rows = get_number_streaming(&tag, "ySplit");
            }
        }

        Ok(())
    }

    fn load_current_sheet_view_selection(&mut self, tag: BytesStart) -> Result<(), XlsxError> {
        let active_cell = match get_optional_attribute_streaming(&tag, "activeCell")?
            .map(|a| parse_cell_reference(&a))
        {
            Some(Ok(s)) => Some(s),
            _ => None,
        };

        let sqref = match get_optional_attribute_streaming(&tag, "sqref")?.map(|s| parse_range(&s))
        {
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

        self.current_sheet_view.selected_row = selected_row;
        self.current_sheet_view.selected_column = selected_column;
        self.current_sheet_view.range = [row1, column1, row2, column2];

        Ok(())
    }

    fn load_column(&mut self, tag: BytesStart) -> Result<(), XlsxError> {
        // cols
        // <cols>
        //     <col min="5" max="5" width="38.26953125" customWidth="1"/>
        //     <col min="6" max="6" width="9.1796875" style="1"/>
        //     <col min="8" max="8" width="4" customWidth="1"/>
        // </cols>

        let min = get_required_attribute_streaming(&tag, "min")?;
        let min = min.parse::<i32>()?;

        let max = get_required_attribute_streaming(&tag, "max")?;
        let max = max.parse::<i32>()?;

        let width = get_required_attribute_streaming(&tag, "width")?;
        let width = width.parse::<f64>()?;

        let custom_width = match get_optional_attribute_streaming(&tag, "customWidth")? {
            Some(w) => w == "1",
            None => false,
        };

        let style =
            get_optional_attribute_streaming(&tag, "style")?.map(|s| s.parse::<i32>().unwrap_or(0));
        self.cols.push(Col {
            min,
            max,
            width,
            custom_width,
            style,
        });

        Ok(())
    }

    fn load_sheet_color(&mut self, tag: BytesStart) -> Result<(), XlsxError> {
        // <sheetPr>
        //     <tabColor theme="5" tint="-0.249977111117893"/>
        // </sheetPr>
        let color = get_color_streaming(tag)?;
        self.colors.push(color);

        Ok(())
    }

    fn load_merge_cell(&mut self, tag: BytesStart) -> Result<(), XlsxError> {
        // 18.3.1.55 Merge Cells
        // <mergeCells count="1">
        //    <mergeCell ref="K7:L10"/>
        // </mergeCells>
        let reference = get_required_attribute_streaming(&tag, "ref")?.to_string();
        self.merge_cells.push(reference);

        Ok(())
    }

    fn load_row(&mut self, tag: BytesStart) -> Result<(), XlsxError> {
        // sheetData
        // <row r="1" spans="1:15" x14ac:dyDescent="0.35">
        //     <c r="A1" t="s">
        //         <v>0</v>
        //     </c>
        //     <c r="D1">
        //         <f>C1+1</f>
        //     </c>
        // </row>

        let default_row_height = 14.5;

        let mut height_attribute: Option<f64> = None;

        // The height of the row is always the visible height of the row
        // If custom_height is false that means the height was calculated automatically:
        // for example because a cell has many lines or a larger font
        let mut custom_height = false;

        let mut row_style: i32 = 0;
        let mut custom_format = false;
        let mut hidden = false;

        // Performance optimization: more efficient than multiple calls to get_optional_attribute_streaming and
        // get_required_attribute_streaming since each call has to re-parse the attributes.
        // Don't check for duplicate attributes for performance reasons.
        for attribute in tag.attributes().with_checks(false) {
            let attribute = attribute.map_err(|e| {
                XlsxError::Xml(format!("Unable to parse attribute: {:?}", e.to_string()))
            })?;

            match attribute.key.as_ref() {
                attr_name @ b"r" => {
                    // Performance optimization: converting directly from_utf8 here is faster than calling
                    // attribute.unescape_value(), which does this then unescapes. There is nothing to unescape
                    // in the attribute values for XLSX, so skip that.
                    let value = std::str::from_utf8(&attribute.value).map_err(|e| {
                        XlsxError::Xml(format!(
                            "Unable to decode attribute: \"{:?}\": {:?}",
                            attr_name,
                            e.to_string()
                        ))
                    })?;
                    // This is the row number 1-indexed
                    self.current_row_index = value.parse()?;
                }
                attr_name @ b"ht" => {
                    // Performance optimization: converting directly from_utf8 here is faster than calling
                    // attribute.unescape_value(), which does this then unescapes. There is nothing to unescape
                    // in the attribute values for XLSX, so skip that.
                    let value = std::str::from_utf8(&attribute.value).map_err(|e| {
                        XlsxError::Xml(format!(
                            "Unable to decode attribute: \"{:?}\": {:?}",
                            attr_name,
                            e.to_string()
                        ))
                    })?;

                    height_attribute = Some(value.parse().unwrap_or(default_row_height));
                }
                attr_name @ b"customHeight" => {
                    // Performance optimization: converting directly from_utf8 here is faster than calling
                    // attribute.unescape_value(), which does this then unescapes. There is nothing to unescape
                    // in the attribute values for XLSX, so skip that.
                    let value = std::str::from_utf8(&attribute.value).map_err(|e| {
                        XlsxError::Xml(format!(
                            "Unable to decode attribute: \"{:?}\": {:?}",
                            attr_name,
                            e.to_string()
                        ))
                    })?;

                    custom_height = value == "1";
                }
                attr_name @ b"s" => {
                    // Performance optimization: converting directly from_utf8 here is faster than calling
                    // attribute.unescape_value(), which does this then unescapes. There is nothing to unescape
                    // in the attribute values for XLSX, so skip that.
                    let value = std::str::from_utf8(&attribute.value).map_err(|e| {
                        XlsxError::Xml(format!(
                            "Unable to decode attribute: \"{:?}\": {:?}",
                            attr_name,
                            e.to_string()
                        ))
                    })?;

                    row_style = value.parse().unwrap_or(0);
                }
                attr_name @ b"customFormat" => {
                    // Performance optimization: converting directly from_utf8 here is faster than calling
                    // attribute.unescape_value(), which does this then unescapes. There is nothing to unescape
                    // in the attribute values for XLSX, so skip that.
                    let value = std::str::from_utf8(&attribute.value).map_err(|e| {
                        XlsxError::Xml(format!(
                            "Unable to decode attribute: \"{:?}\": {:?}",
                            attr_name,
                            e.to_string()
                        ))
                    })?;

                    custom_format = value == "1";
                }
                attr_name @ b"hidden" => {
                    // Performance optimization: converting directly from_utf8 here is faster than calling
                    // attribute.unescape_value(), which does this then unescapes. There is nothing to unescape
                    // in the attribute values for XLSX, so skip that.
                    let value = std::str::from_utf8(&attribute.value).map_err(|e| {
                        XlsxError::Xml(format!(
                            "Unable to decode attribute: \"{:?}\": {:?}",
                            attr_name,
                            e.to_string()
                        ))
                    })?;

                    hidden = value == "1";
                }
                _ => {}
            }
        }

        // `spans` is not used in IronCalc at the moment (it's an optimization)
        // let spans = row.attribute("spans");
        // This is the height of the row
        let has_height_attribute = height_attribute.is_some();
        let height = height_attribute.unwrap_or(default_row_height);

        if custom_height || custom_format || row_style != 0 || has_height_attribute || hidden {
            self.rows.push(Row {
                r: self.current_row_index,
                height,
                s: row_style,
                custom_height,
                custom_format,
                hidden,
            });
        }

        // Unused attributes:
        // * thickBot, thickTop, ph, collapsed, outlineLevel

        Ok(())
    }

    fn cleanup_current_data_row(&mut self) -> Result<(), XlsxError> {
        // Memory optimization: more efficent than cloning for insertion into current_row_index, then clearing.
        let data_row = std::mem::take(&mut self.current_data_row);
        self.sheet_data.insert(self.current_row_index, data_row);

        self.current_row_index = 0;

        Ok(())
    }

    fn cleanup_current_cell_data(&mut self) -> Result<(), XlsxError> {
        // type, the default type being "n" for number
        // If the cell does not have a value is an empty cell
        let cell_type = match self.current_cell_data.cell_type.as_ref() {
            Some(t) => t,
            None => {
                if self.current_cell_data.cell_value.is_none() {
                    "empty"
                } else {
                    "n"
                }
            }
        };

        let cell = get_cell_from_excel(
            self.current_cell_data.cell_value.as_deref(),
            self.current_cell_data.value_metadata.as_deref(),
            cell_type,
            self.current_cell_data.cell_style,
            self.current_cell_data.formula_index,
            &self.settings.name,
            &self.current_cell_data.cell_ref,
            self.shared_strings,
        );
        self.current_data_row
            .insert(self.current_cell_data.column, cell);
        self.current_cell_data.set_to_default_values();

        Ok(())
    }

    fn load_cell(&mut self, tag: BytesStart) -> Result<(), XlsxError> {
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

        let mut found_cell_ref = false;

        // Performance optimization: more efficient than multiple calls to get_optional_attribute_streaming and
        // get_required_attribute_streaming since each call has to re-parse the attributes.
        // Don't check for duplicate attributes for performance reasons.
        for attribute in tag.attributes().with_checks(false) {
            let attribute = attribute.map_err(|e| {
                XlsxError::Xml(format!("Unable to parse attribute: {:?}", e.to_string()))
            })?;

            match attribute.key.as_ref() {
                attr_name @ b"r" => {
                    // Performance optimization: converting directly from_utf8 here is faster than calling
                    // attribute.unescape_value(), which does this then unescapes. There is nothing to unescape
                    // in the attribute values for XLSX, so skip that.
                    let value = std::str::from_utf8(&attribute.value).map_err(|e| {
                        XlsxError::Xml(format!(
                            "Unable to decode attribute: \"{:?}\": {:?}",
                            attr_name,
                            e.to_string()
                        ))
                    })?;
                    self.current_cell_data.cell_ref = value.to_string();
                    found_cell_ref = true;
                }
                attr_name @ b"vm" => {
                    let value = std::str::from_utf8(&attribute.value).map_err(|e| {
                        XlsxError::Xml(format!(
                            "Unable to decode attribute: \"{:?}\": {:?}",
                            attr_name,
                            e.to_string()
                        ))
                    })?;
                    self.current_cell_data.value_metadata = Some(value.to_string());
                }
                attr_name @ b"t" => {
                    let value = std::str::from_utf8(&attribute.value).map_err(|e| {
                        XlsxError::Xml(format!(
                            "Unable to decode attribute: \"{:?}\": {:?}",
                            attr_name,
                            e.to_string()
                        ))
                    })?;
                    self.current_cell_data.cell_type = Some(value.to_string());
                }
                // style index, the default style is 0
                attr_name @ b"s" => {
                    let value = std::str::from_utf8(&attribute.value).map_err(|e| {
                        XlsxError::Xml(format!(
                            "Unable to decode attribute: \"{:?}\": {:?}",
                            attr_name,
                            e.to_string()
                        ))
                    })?;
                    self.current_cell_data.cell_style = value.parse::<i32>().unwrap_or(0);
                }
                _ => {}
            }
        }

        if !found_cell_ref {
            return Err(XlsxError::Xml(
                "Missing required \"r\" XML attribute".to_string(),
            ));
        }

        let column_letter = get_column_from_ref(&self.current_cell_data.cell_ref);
        self.current_cell_data.column =
            column_to_number(column_letter.as_str()).map_err(XlsxError::Xml)?;

        Ok(())
    }

    fn load_formula_attributes(&mut self, tag: BytesStart) -> Result<(), XlsxError> {
        self.current_cell_data.formula_data.formula_type =
            get_optional_attribute_streaming(&tag, "t")?
                .unwrap_or("normal".into())
                .to_string();
        self.current_cell_data.formula_data.formula_si =
            get_optional_attribute_streaming(&tag, "si")?.map(|s| s.to_string());
        self.current_cell_data.formula_data.formula_has_ref =
            get_optional_attribute_streaming(&tag, "ref")?.is_some();

        Ok(())
    }

    fn load_formula_value(&mut self, tag: BytesText) -> Result<(), XlsxError> {
        self.current_cell_data.formula_data.formula_text = tag
            .unescape()
            .ok()
            .map(|t| t.to_string())
            .unwrap_or_default();

        Ok(())
    }

    fn cleanup_current_formula_data(&mut self) -> Result<(), XlsxError> {
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

        // formula types:
        // 18.18.6 ST_CellFormulaType (Formula Type)
        // array (Array Formula) Formula is an array formula.
        // dataTable (Table Formula) Formula is a data table formula.
        // normal (Normal) Formula is a regular cell formula. (Default)
        // shared (Shared Formula) Formula is part of a shared formula.
        self.current_cell_data.formula_index = -1;
        let formula_type = self.current_cell_data.formula_data.formula_type.as_str();
        match formula_type {
            "shared" => {
                // We have a shared formula
                let si = self
                    .current_cell_data
                    .formula_data
                    .formula_si
                    .as_ref()
                    .ok_or_else(|| {
                        XlsxError::Xml("Shared formulas must have an si attribute".to_string())
                    })?;
                let si = si.parse::<i32>()?;
                if self.current_cell_data.formula_data.formula_has_ref {
                    // It's the mother cell. We do not use the ref attribute in IronCalc
                    let formula = self.current_cell_data.formula_data.formula_text.clone();
                    let context =
                        format!("{}!{}", self.settings.name, self.current_cell_data.cell_ref);
                    let formula =
                        from_a1_to_rc(formula, self.worksheets, context, self.tables.clone())?;
                    match self.index_map.get(&si) {
                        Some(index) => {
                            // The index for that formula already exists meaning we bumped into a daughter cell first
                            // TODO: Worth assert the content is a placeholder?
                            self.current_cell_data.formula_index = *index;
                            self.shared_formulas
                                .insert(self.current_cell_data.formula_index as usize, formula);
                        }
                        None => {
                            // We haven't met any of the daughter cells
                            match get_formula_index(&formula, &self.shared_formulas) {
                                // The formula is already present, use that index
                                Some(index) => {
                                    self.current_cell_data.formula_index = index;
                                }
                                None => {
                                    self.shared_formulas.push(formula);
                                    self.current_cell_data.formula_index =
                                        self.shared_formulas.len() as i32 - 1;
                                }
                            };
                            self.index_map
                                .insert(si, self.current_cell_data.formula_index);
                        }
                    }
                } else {
                    // It's a daughter cell
                    match self.index_map.get(&si) {
                        Some(index) => {
                            self.current_cell_data.formula_index = *index;
                        }
                        None => {
                            // Haven't bumped into the mother cell yet. We insert a placeholder.
                            // Note that it is perfectly possible that the formula of the mother cell
                            // is already in the set of array formulas. This will lead to the above mention duplicity.
                            // This is not a problem
                            let placeholder = "".to_string();
                            self.shared_formulas.push(placeholder);
                            self.current_cell_data.formula_index =
                                self.shared_formulas.len() as i32 - 1;
                            self.index_map
                                .insert(si, self.current_cell_data.formula_index);
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
                let formula = self.current_cell_data.formula_data.formula_text.clone();
                let context = format!("{}!{}", self.settings.name, self.current_cell_data.cell_ref);
                let formula =
                    from_a1_to_rc(formula, self.worksheets, context, self.tables.clone())?;

                match get_formula_index(&formula, &self.shared_formulas) {
                    Some(index) => self.current_cell_data.formula_index = index,
                    None => {
                        self.shared_formulas.push(formula);
                        self.current_cell_data.formula_index =
                            self.shared_formulas.len() as i32 - 1;
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

        self.current_cell_data.formula_data = FormulaData::default();

        Ok(())
    }

    fn load_value(&mut self, tag: BytesText) -> Result<(), XlsxError> {
        // We check the value "v" child.
        self.current_cell_data.cell_value = Some(
            tag.unescape()
                .ok()
                .map(|t| t.to_string())
                .unwrap_or_default(),
        );

        Ok(())
    }

    fn process(&mut self, ev: Event) -> Result<(), XlsxError> {
        self.state = match self.state {
            ParseState::Start => match ev {
                Event::Start(e) if e.local_name().into_inner() == b"worksheet" => {
                    ParseState::Worksheet
                }
                _ => ParseState::Start,
            },
            ParseState::Worksheet => match ev {
                Event::Start(e) if e.local_name().into_inner() == b"dimension" => {
                    self.load_dimension(e)?;

                    ParseState::Worksheet
                }
                Event::Start(e) if e.local_name().into_inner() == b"sheetViews" => {
                    ParseState::SheetViews
                }
                Event::Start(e) if e.local_name().into_inner() == b"sheetData" => {
                    ParseState::SheetData
                }
                Event::Start(e) if e.local_name().into_inner() == b"sheetPr" => ParseState::SheetPr,
                Event::Start(e) if e.local_name().into_inner() == b"cols" => ParseState::Column,
                Event::Start(e) if e.local_name().into_inner() == b"mergeCells" => {
                    ParseState::MergeCell
                }
                Event::End(e) if e.local_name().into_inner() == b"worksheet" => ParseState::End,
                _ => ParseState::Worksheet,
            },
            ParseState::MergeCell => match ev {
                Event::Start(e) if e.local_name().into_inner() == b"mergeCell" => {
                    self.load_merge_cell(e)?;
                    ParseState::MergeCell
                }
                Event::End(e) if e.local_name().into_inner() == b"mergeCells" => {
                    ParseState::Worksheet
                }
                _ => ParseState::MergeCell,
            },
            ParseState::Column => match ev {
                Event::Start(e) if e.local_name().into_inner() == b"col" => {
                    self.load_column(e)?;
                    ParseState::Column
                }
                Event::End(e) if e.local_name().into_inner() == b"cols" => ParseState::Worksheet,
                _ => ParseState::Column,
            },
            ParseState::SheetPr => match ev {
                Event::Start(e) if e.local_name().into_inner() == b"tabColor" => {
                    self.load_sheet_color(e)?;
                    ParseState::SheetPr
                }
                Event::End(e) if e.local_name().into_inner() == b"sheetPr" => ParseState::Worksheet,
                _ => ParseState::SheetPr,
            },
            ParseState::SheetViews => match ev {
                Event::Start(e) if e.local_name().into_inner() == b"sheetView" => {
                    self.load_current_sheet_view_attributes(e)?;
                    ParseState::SheetView
                }
                Event::End(e) if e.local_name().into_inner() == b"sheetViews" => {
                    ParseState::Worksheet
                }
                _ => ParseState::SheetViews,
            },
            ParseState::SheetView => match ev {
                Event::Start(e) if e.local_name().into_inner() == b"pane" => {
                    self.load_current_sheet_view_pane(e)?;
                    ParseState::SheetView
                }
                Event::Start(e) if e.local_name().into_inner() == b"selection" => {
                    self.load_current_sheet_view_selection(e)?;
                    ParseState::SheetView
                }
                Event::End(e) if e.local_name().into_inner() == b"sheetView" => {
                    self.sheet_views.push(self.current_sheet_view.clone());
                    self.current_sheet_view = Default::default();
                    ParseState::SheetViews
                }
                _ => ParseState::SheetView,
            },
            ParseState::SheetData => match ev {
                Event::Start(e) if e.local_name().into_inner() == b"row" => {
                    self.load_row(e)?;
                    ParseState::Row
                }
                Event::End(e) if e.local_name().into_inner() == b"sheetData" => {
                    ParseState::Worksheet
                }
                _ => ParseState::SheetData,
            },
            ParseState::Row => match ev {
                Event::Start(e) if e.local_name().into_inner() == b"c" => {
                    self.load_cell(e)?;
                    ParseState::Cell
                }
                Event::End(e) if e.local_name().into_inner() == b"row" => {
                    self.cleanup_current_data_row()?;
                    ParseState::SheetData
                }
                _ => ParseState::Row,
            },
            ParseState::Cell => match ev {
                Event::Start(e) if e.local_name().into_inner() == b"v" => ParseState::Value,
                Event::Start(e) if e.local_name().into_inner() == b"f" => {
                    self.load_formula_attributes(e)?;
                    ParseState::Formula
                }
                Event::End(e) if e.local_name().into_inner() == b"c" => {
                    self.cleanup_current_cell_data()?;
                    ParseState::Row
                }
                _ => ParseState::Cell,
            },
            ParseState::Value => match ev {
                Event::Text(t) => {
                    self.load_value(t)?;
                    ParseState::Value
                }
                Event::End(e) if e.local_name().into_inner() == b"v" => ParseState::Cell,
                _ => ParseState::Value,
            },
            ParseState::Formula => match ev {
                Event::Text(t) => {
                    self.load_formula_value(t)?;
                    ParseState::Formula
                }
                Event::End(e) if e.local_name().into_inner() == b"f" => {
                    self.cleanup_current_formula_data()?;
                    ParseState::Cell
                }
                _ => ParseState::Formula,
            },
            ParseState::End => ParseState::End,
        };

        Ok(())
    }

    fn worksheet(mut self) -> Result<(Worksheet, bool), XlsxError> {
        if self.state != ParseState::End {
            return Err(XlsxError::Xml("Corrupt XML structure".to_string()));
        }

        let dimension = if self.dimensions.len() == 1 {
            self.dimensions.remove(0)
        } else {
            "A1".to_string()
        };

        let sheet_view = if self.sheet_views.len() == 1 {
            self.sheet_views.remove(0)
        } else {
            SheetView::default()
        };

        let color = if self.colors.len() == 1 {
            self.colors.remove(0)
        } else {
            None
        };

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
                cols: self.cols,
                rows: self.rows,
                name: self.settings.name,
                sheet_data: self.sheet_data,
                shared_formulas: self.shared_formulas,
                sheet_id: self.settings.id,
                state: self.settings.state,
                color,
                merge_cells: self.merge_cells,
                comments: self.settings.comments,
                frozen_rows: sheet_view.frozen_rows,
                frozen_columns: sheet_view.frozen_columns,
                show_grid_lines: sheet_view.show_grid_lines,
                views,
            },
            sheet_view.is_selected,
        ))
    }
}

fn get_required_attribute_streaming<'a>(
    tag: &'a BytesStart,
    attr_name: &str,
) -> Result<Cow<'a, str>, XlsxError> {
    tag.try_get_attribute(attr_name)
        .map_err(|e| {
            XlsxError::Xml(format!(
                "Unable to parse attribute: \"{:?}\": {:?}",
                attr_name,
                e.to_string()
            ))
        })?
        .ok_or_else(|| {
            XlsxError::Xml(format!(
                "Missing required \"{:?}\" XML attribute",
                attr_name
            ))
        })?
        .unescape_value()
        .map_err(|e| {
            XlsxError::Xml(format!(
                "Unable to decode and unescape attribute: \"{:?}\": {:?}",
                attr_name,
                e.to_string()
            ))
        })
}

fn get_optional_attribute_streaming<'a>(
    tag: &'a BytesStart,
    attr_name: &str,
) -> Result<Option<Cow<'a, str>>, XlsxError> {
    tag.try_get_attribute(attr_name)
        .map_err(|e| {
            XlsxError::Xml(format!(
                "Unable to parse attribute: \"{:?}\": {:?}",
                attr_name,
                e.to_string()
            ))
        })?
        .map(|a| a.unescape_value())
        .transpose()
        .map_err(|e| {
            XlsxError::Xml(format!(
                "Unable to decode and unescape attribute: \"{:?}\": {:?}",
                attr_name,
                e.to_string()
            ))
        })
}

fn get_number_streaming(tag: &BytesStart, attr_name: &str) -> i32 {
    get_optional_attribute_streaming(tag, attr_name)
        .ok()
        .unwrap_or(Some("0".into()))
        .unwrap_or("0".into())
        .parse::<i32>()
        .unwrap_or(0)
}

pub(super) fn get_color_streaming(tag: BytesStart) -> Result<Option<String>, XlsxError> {
    // 18.3.1.15 color (Data Bar Color)
    if let Some(mut val) = get_optional_attribute_streaming(&tag, "rbg")? {
        // FIXME the two first values is normally the alpha.
        if val.len() == 8 {
            val = format!("#{}", &val[2..8]).into();
        }
        Ok(Some(val.to_string()))
    } else if let Some(index) = get_optional_attribute_streaming(&tag, "indexed")? {
        let index = index.parse::<i32>()?;
        let rgb = get_indexed_color(index);
        Ok(Some(rgb))
        // Color::Indexed(val)
    } else if let Some(theme) = get_optional_attribute_streaming(&tag, "theme")? {
        let theme = theme.parse::<i32>()?;
        let tint = get_optional_attribute_streaming(&tag, "tint")?
            .map(|t| t.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);
        let rgb = colors::get_themed_color(theme, tint);
        Ok(Some(rgb))
    // Color::Theme { theme, tint }
    } else if get_optional_attribute_streaming(&tag, "auto")?.is_some() {
        // TODO: Is this correct?
        // A boolean value indicating the color is automatic and system color dependent.
        Ok(None)
    } else {
        println!("Unexpected color node {:?}", &tag);
        Ok(None)
    }
}
