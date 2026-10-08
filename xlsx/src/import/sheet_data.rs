//! Streams the `sheetData` element of a worksheet with a pull parser.
//!
//! `sheetData` is nearly all of a worksheet's XML, and reading it as a tree
//! costs many times its size in memory. The reader below walks the worksheet
//! part once with `quick-xml`: cells inside `sheetData` become `Cell`s row by
//! row, and every other element goes into a small tree (the worksheet without
//! its cells) that the rest of the importer reads.

use std::borrow::Cow;
use std::collections::HashMap;
use std::io::BufRead;

use ironcalc_base::expressions::parser::Parser;
use ironcalc_base::types::{Cell, Row, SheetData};
use quick_xml::events::{BytesEnd, BytesStart, Event};

use crate::error::XlsxError;

use super::shared_strings::SharedStringTable;
use super::util::parse_bool_with_default;
use super::worksheets::{
    find_or_add_formula, from_a1_to_rc, get_cell_from_excel, parse_cell_reference, parse_range,
    CellArrayKind,
};
use super::xml::{attribute_value_of, reference_of, XmlNode, XmlTreeBuilder};
use quick_xml::XmlVersion;

// sheetData
// <row r="1" spans="1:15" x14ac:dyDescent="0.35">
//     <c r="A1" t="s">
//         <v>0</v>
//     </c>
//     <c r="D1">
//         <f>C1+1</f>
//     </c>
// </row>

const DEFAULT_ROW_HEIGHT: f64 = 14.5;

/// What `read_sheet_data` reads from a worksheet part.
pub(super) struct SheetDataXml {
    /// The worksheet with the `sheetData` element removed.
    pub(super) worksheet: XmlNode,
    pub(super) sheet_data: SheetData,
    /// The rows that carry a height, a style or a flag.
    pub(super) rows: Vec<Row>,
    pub(super) shared_formulas: Vec<String>,
}

/// The elements inside `sheetData` the reader cares about.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tag {
    Row,
    Cell,
    Value,
    Formula,
    InlineString,
    Text,
    Other,
}

fn tag_of(element: &BytesStart) -> Tag {
    match element.local_name().as_ref() {
        "row" => Tag::Row,
        "c" => Tag::Cell,
        "v" => Tag::Value,
        "f" => Tag::Formula,
        "is" => Tag::InlineString,
        "t" => Tag::Text,
        _ => Tag::Other,
    }
}

fn is_sheet_data(element: &BytesStart) -> bool {
    element.local_name().as_ref() == "sheetData"
}

fn is_sheet_data_end(element: &BytesEnd) -> bool {
    element.local_name().as_ref() == "sheetData"
}

/// A `<row>` while its cells are being read.
struct RowXml {
    /// From the `r` attribute, or from the first cell if the row has none.
    index: Option<i32>,
    /// With their column, in the order the file gives them.
    cells: Vec<(i32, Cell)>,
}

/// An `<f>` element: the formula of a cell.
#[derive(Default)]
struct FormulaXml {
    /// The `t` attribute: normal (absent), shared, array or dataTable.
    kind: Option<String>,
    /// The `ref` attribute: the range of a shared or array formula.
    reference: Option<String>,
    /// The `si` attribute: the index of a shared formula.
    shared_index: Option<String>,
    /// The `ca` attribute is "1": calculate always.
    calculate_always: bool,
    /// None if the element has no text at all.
    text: Option<String>,
    has_elements: bool,
}

/// A `<c>` element while its children are being read.
#[derive(Default)]
struct CellXml {
    reference: String,
    /// The style index, the default style being 0.
    style: i32,
    /// The `t` attribute.
    cell_type: Option<Cow<'static, str>>,
    /// The `cm` attribute is "1".
    is_dynamic_array: bool,
    /// The `vm` attribute.
    value_metadata: Option<String>,
    /// The text of the value child `<v>`; None if there is no `<v>`.
    value: Option<String>,
    /// The joined text of the `<t>` elements under `<is>`; None if there is
    /// no `<is>`.
    /// <c r="A1" t="inlineStr">
    ///   <is>
    ///     <t>Hello, World!</t>
    ///   </is>
    /// </c>
    rich_text: Option<String>,
    formula: Option<FormulaXml>,
}

struct SheetDataReader<'a, 'p> {
    sheet_name: &'a str,
    parser: &'a mut Parser<'p>,
    shared_strings: &'a mut SharedStringTable,

    sheet_data: SheetData,
    rows: Vec<Row>,
    shared_formulas: Vec<String>,
    /// Where each formula is among `shared_formulas`, see `find_or_add_formula`.
    formula_lookup: HashMap<String, i32>,
    /// From the shared formula index in Excel (`si`) to the index in IronCalc.
    index_map: HashMap<i32, i32>,
    /// Cells covered by an array formula, with the anchor cell of the formula.
    array_cell: HashMap<(i32, i32), (i32, i32)>,

    /// The open elements inside `sheetData`.
    stack: Vec<Tag>,
    row: Option<RowXml>,
    cell: Option<CellXml>,

    /// What the last row and the last cell were read into, kept for the next
    /// ones: a sheet has millions of cells, and with these none of them
    /// asks for memory of its own while it is read.
    row_buffer: Vec<(i32, Cell)>,
    reference_buffer: String,
    value_buffer: String,
}

/// The `t` attribute of a cell. The types there are need no string of their own.
fn cell_type_of(value: &str) -> Cow<'static, str> {
    for known in ["b", "d", "e", "inlineStr", "n", "s", "str"] {
        if value == known {
            return Cow::Borrowed(known);
        }
    }
    Cow::Owned(value.to_string())
}

impl SheetDataReader<'_, '_> {
    fn process(&mut self, event: &Event) -> Result<(), XlsxError> {
        match event {
            Event::Start(element) => {
                let tag = tag_of(element);
                self.open(tag, element)?;
                self.stack.push(tag);
            }
            Event::Empty(element) => {
                let tag = tag_of(element);
                self.open(tag, element)?;
                self.close(tag)?;
            }
            Event::End(_) => {
                if let Some(tag) = self.stack.pop() {
                    self.close(tag)?;
                }
            }
            Event::Text(text) => {
                let text = text.xml_content(XmlVersion::Implicit1_0);
                self.text(&text);
            }
            Event::CData(data) => {
                let text = data.xml_content(XmlVersion::Implicit1_0);
                self.text(&text);
            }
            Event::GeneralRef(reference) => {
                let text = reference_of(reference)?;
                self.text(&text);
            }
            _ => {}
        }
        Ok(())
    }

    fn open(&mut self, tag: Tag, element: &BytesStart) -> Result<(), XlsxError> {
        if self.stack.last() == Some(&Tag::Formula) {
            if let Some(formula) = self.cell.as_mut().and_then(|c| c.formula.as_mut()) {
                formula.has_elements = true;
            }
        }
        match tag {
            Tag::Row => self.start_row(element)?,
            Tag::Cell => self.start_cell(element)?,
            Tag::Value => {
                if let Some(cell) = self.cell.as_mut() {
                    let mut value = std::mem::take(&mut self.value_buffer);
                    value.clear();
                    cell.value = Some(value);
                }
            }
            Tag::Formula => {
                if let Some(cell) = self.cell.as_mut() {
                    let mut formula = FormulaXml::default();
                    for attribute in element.attributes() {
                        let attribute = attribute?;
                        let value = attribute_value_of(&attribute)?;
                        match attribute.key.as_ref() {
                            "t" => formula.kind = Some(value.into_owned()),
                            "ref" => formula.reference = Some(value.into_owned()),
                            "si" => formula.shared_index = Some(value.into_owned()),
                            "ca" => formula.calculate_always = &*value == "1",
                            _ => {}
                        }
                    }
                    cell.formula = Some(formula);
                }
            }
            Tag::InlineString => {
                if let Some(cell) = self.cell.as_mut() {
                    cell.rich_text = Some(String::new());
                }
            }
            Tag::Text | Tag::Other => {}
        }
        Ok(())
    }

    fn close(&mut self, tag: Tag) -> Result<(), XlsxError> {
        match tag {
            Tag::Row => self.end_row(),
            Tag::Cell => self.end_cell(),
            _ => Ok(()),
        }
    }

    fn text(&mut self, text: &str) {
        let Some(cell) = self.cell.as_mut() else {
            return;
        };
        match self.stack.last() {
            Some(Tag::Value) => {
                if let Some(value) = cell.value.as_mut() {
                    value.push_str(text);
                }
            }
            Some(Tag::Formula) => {
                if let Some(formula) = cell.formula.as_mut() {
                    formula.text.get_or_insert_with(String::new).push_str(text);
                }
            }
            Some(Tag::Text) if self.stack.contains(&Tag::InlineString) => {
                if let Some(rich_text) = cell.rich_text.as_mut() {
                    rich_text.push_str(text);
                }
            }
            _ => {}
        }
    }

    fn start_row(&mut self, element: &BytesStart) -> Result<(), XlsxError> {
        // <row r="1" spans="1:15" ht="30" customHeight="1" s="3" customFormat="1" hidden="1">
        // `r` is the row number, 1-indexed; `ht` the height of the row.
        // `spans` is not used in IronCalc at the moment (it's an optimization).
        // Unused attributes:
        // * thickBot, thickTop, ph, collapsed, outlineLevel
        let mut index = None;
        let mut height = DEFAULT_ROW_HEIGHT;
        let mut has_height_attribute = false;
        let mut custom_height = false;
        let mut style = 0;
        let mut custom_format = false;
        let mut hidden = false;
        for attribute in element.attributes() {
            let attribute = attribute?;
            let value = attribute_value_of(&attribute)?;
            match attribute.key.as_ref() {
                "r" => index = Some(value.parse::<i32>()?),
                "ht" => {
                    has_height_attribute = true;
                    height = value.parse::<f64>().unwrap_or(DEFAULT_ROW_HEIGHT);
                }
                "customHeight" => custom_height = parse_bool_with_default(Some(&value), false),
                "s" => style = value.parse::<i32>().unwrap_or(0),
                "customFormat" => custom_format = parse_bool_with_default(Some(&value), false),
                "hidden" => hidden = parse_bool_with_default(Some(&value), false),
                _ => {}
            }
        }
        if let Some(index) = index {
            // The height of the row is always the visible height of the row.
            // If custom_height is false the height was computed automatically,
            // for example because a cell has many lines or a larger font.
            if custom_height || custom_format || style != 0 || has_height_attribute || hidden {
                self.rows.push(Row {
                    r: index,
                    height,
                    s: style,
                    custom_height,
                    custom_format,
                    hidden,
                });
            }
        }
        self.row = Some(RowXml {
            index,
            cells: std::mem::take(&mut self.row_buffer),
        });
        Ok(())
    }

    fn end_row(&mut self) -> Result<(), XlsxError> {
        let Some(mut row) = self.row.take() else {
            return Ok(());
        };
        match row.index {
            Some(index) => {
                self.sheet_data.set_row(index, row.cells.drain(..));
                self.row_buffer = row.cells;
                Ok(())
            }
            None => Err(XlsxError::Xml(
                "Row without a row index (r attribute)".to_string(),
            )),
        }
    }

    fn start_cell(&mut self, element: &BytesStart) -> Result<(), XlsxError> {
        // 18.3.1.4 c (Cell)
        // Child Elements:
        // * v: Cell value
        // * is: Rich Text Inline
        // * f: Formula
        // Attributes:
        // r: reference. A1 style
        // s: style index
        // t: cell type
        // cm: cell metadata (used for dynamic arrays)
        // vm: value metadata (used for #SPILL! and #CALC! errors)
        // ph: Show Phonetic, unused
        let mut cell = CellXml {
            reference: std::mem::take(&mut self.reference_buffer),
            ..Default::default()
        };
        cell.reference.clear();
        let mut has_reference = false;
        for attribute in element.attributes() {
            let attribute = attribute?;
            let value = attribute_value_of(&attribute)?;
            match attribute.key.as_ref() {
                "r" => {
                    has_reference = true;
                    cell.reference.clear();
                    cell.reference.push_str(&value);
                }
                "s" => cell.style = value.parse::<i32>().unwrap_or(0),
                "t" => cell.cell_type = Some(cell_type_of(&value)),
                "cm" => cell.is_dynamic_array = &*value == "1",
                "vm" => cell.value_metadata = Some(value.into_owned()),
                _ => {}
            }
        }
        if !has_reference {
            return Err(XlsxError::Xml("Missing \"r\" XML attribute".to_string()));
        }
        self.cell = Some(cell);
        Ok(())
    }

    fn end_cell(&mut self) -> Result<(), XlsxError> {
        let Some(cell) = self.cell.take() else {
            return Ok(());
        };
        let Some(row) = self.row.as_mut() else {
            return Err(XlsxError::Xml("Cell outside of a row".to_string()));
        };
        let (r_index, column_index) =
            parse_cell_reference(&cell.reference).map_err(XlsxError::Xml)?;
        // A row without an `r` attribute takes its index from its first cell
        if row.index.is_none() {
            row.index = Some(r_index);
        }

        // type, the default type being "n" for number
        // If the cell does not have a value is an empty cell
        let cell_type = match cell.cell_type.as_deref() {
            Some(t) => t,
            None => {
                if cell.value.is_none() {
                    "empty"
                } else {
                    "n"
                }
            }
        };

        // In Excel some formulas are shared and some are not, but in IronCalc all formulas are shared
        // A cell with a "non-shared" formula is like:
        // <c r="E3">
        //   <f>C2+1</f>
        //   <v>3</v>
        // </c>
        // A cell with a shared formula will be either an "anchor" cell:
        // <c r="D2">
        //   <f t="shared" ref="D2:D3" si="0">C2+1</f>
        //   <v>3</v>
        // </c>
        // Or a child cell:
        // <c r="D3">
        //   <f t="shared" si="0"/>
        //   <v>4</v>
        // </c>
        // In IronCalc two cells have the same formula iff the R1C1 representation is the same
        // TODO: This algorithm could end up with "repeated" shared formulas
        //       We could solve that with a second transversal.

        // In Excel a volatile spill formula might have an f element in the spilled cells.
        // But it is not a shared formula. For example:
        // <c r="A19" s="3" cm="1">
        //   <f t="array" aca="1" ref="A19:C21" ca="1">_xlfn.RANDARRAY(3,3, 0, 100,TRUE)</f>
        //   <v>52</v>
        // </c>
        // <c r="B19" s="3">
        //   <f ca="1"/>
        //   <v>20</v>
        // </c>
        // <c r="C19" s="3">
        //   <f ca="1"/>
        //   <v>41</v>
        // </c>
        // aca: Always Calculate Array
        // ca: Calculate Always
        // Those are hints Excel uses to always calculate volatiles
        // We do not use those in IronCalc
        let mut formula_index = -1;
        let mut array_kind = CellArrayKind::None;
        if let Some(formula) = cell.formula {
            // formula types:
            // 18.18.6 ST_CellFormulaType (Formula Type)
            // array (Array Formula) Formula is an array formula.
            // dataTable (Table Formula) Formula is a data table formula.
            // normal (Normal) Formula is a regular cell formula. (Default)
            // shared (Shared Formula) Formula is part of a shared formula.
            let mut formula_type = formula.kind.as_deref().unwrap_or("normal");
            if formula_type == "normal"
                && formula.calculate_always
                && formula.text.is_none()
                && !formula.has_elements
            {
                // A daughter cell of a shared formula (<f t="shared" ca="1" si="1"/>) is
                // also empty and may carry ca="1"; only an untyped <f ca="1"/> is a
                // volatile spill placeholder.
                // This is a volatile formula that needs to be recalculated at each calculation.
                // <f ca="1"/>
                formula_type = "hint-volatile";
            }
            let formula_text = formula.text.unwrap_or_default();
            let context = format!("{}!{}", self.sheet_name, cell.reference);
            match formula_type {
                "shared" => {
                    let si = formula
                        .shared_index
                        .ok_or_else(|| XlsxError::Xml("Missing \"si\" XML attribute".to_string()))?
                        .parse::<i32>()?;
                    match formula.reference {
                        Some(_) => {
                            // It's the anchor cell. We do not use the ref attribute in IronCalc
                            let formula = from_a1_to_rc(formula_text, self.parser, context, false)?;
                            match self.index_map.get(&si) {
                                Some(index) => {
                                    // The index for that formula already exists meaning we bumped into a daughter cell first:
                                    // it holds a placeholder, which the formula replaces. (Inserting
                                    // it there instead would move every formula after it, and the
                                    // cells that already point at them would point at the wrong one.)
                                    formula_index = *index;
                                    self.formula_lookup
                                        .entry(formula.clone())
                                        .or_insert(formula_index);
                                    if let Some(slot) =
                                        self.shared_formulas.get_mut(formula_index as usize)
                                    {
                                        *slot = formula;
                                    }
                                }
                                None => {
                                    // We haven't met any of the daughter cells
                                    // If the formula is already present that index is used
                                    formula_index = find_or_add_formula(
                                        formula,
                                        &mut self.shared_formulas,
                                        &mut self.formula_lookup,
                                    );
                                    self.index_map.insert(si, formula_index);
                                }
                            }
                        }
                        None => {
                            // It's a daughter cell
                            match self.index_map.get(&si) {
                                Some(index) => {
                                    formula_index = *index;
                                }
                                None => {
                                    // Haven't bumped into the anchor cell yet. We insert a placeholder.
                                    // Note that it is perfectly possible that the formula of the anchor cell
                                    // is already in the set of array formulas. This will lead to the above mention duplicity.
                                    // This is not a problem
                                    self.shared_formulas.push(String::new());
                                    formula_index = self.shared_formulas.len() as i32 - 1;
                                    self.index_map.insert(si, formula_index);
                                }
                            }
                        }
                    }
                }
                "dataTable" => {
                    return Err(XlsxError::NotImplemented("data table formulas".to_string()));
                }
                "array" => {
                    let range = match formula.reference {
                        Some(r) => r,
                        None => {
                            return Err(XlsxError::Xml(
                                "Array formulas must have a ref attribute".to_string(),
                            ))
                        }
                    };
                    // The reference is the set of cell it spills into.
                    let (row1, column1, row2, column2) = parse_range(&range)
                        .map_err(|_| XlsxError::Xml(format!("Invalid range: {}", range)))?;
                    // (row1, colum1) has to be this cell. We need to mark all the other ones as part of the array formula
                    if row1 != r_index || column1 != column_index {
                        return Err(XlsxError::Xml(
                            "The first cell of the range of an array formula must be the anchor cell".to_string(),
                        ));
                    }
                    for r in row1..=row2 {
                        for c in column1..=column2 {
                            if r == row1 && c == column1 {
                                // skip the anchor cell
                                continue;
                            }
                            self.array_cell.insert((r, c), (r_index, column_index));
                        }
                    }
                    if cell.is_dynamic_array {
                        array_kind =
                            CellArrayKind::DynamicArray(column2 - column1 + 1, row2 - row1 + 1);
                    } else {
                        array_kind =
                            CellArrayKind::ArrayFormula(column2 - column1 + 1, row2 - row1 + 1);
                    }
                    let formula = from_a1_to_rc(formula_text, self.parser, context, true)?;
                    formula_index = find_or_add_formula(
                        formula,
                        &mut self.shared_formulas,
                        &mut self.formula_lookup,
                    );
                }
                "normal" => {
                    // Its a cell with a simple formula
                    let formula = from_a1_to_rc(formula_text, self.parser, context, false)?;
                    formula_index = find_or_add_formula(
                        formula,
                        &mut self.shared_formulas,
                        &mut self.formula_lookup,
                    );
                }
                "hint-volatile" => {}
                _ => {
                    return Err(XlsxError::Xml(format!(
                        "Invalid formula type {formula_type:?}.",
                    )));
                }
            }
        }
        let anchor_cell = self.array_cell.get(&(r_index, column_index)).cloned();
        let cell_read = get_cell_from_excel(
            cell.value.as_deref(),
            cell.value_metadata.as_deref(),
            cell_type,
            cell.style,
            formula_index,
            self.sheet_name,
            &cell.reference,
            self.shared_strings,
            cell.rich_text,
            anchor_cell,
            array_kind,
        );
        row.cells.push((column_index, cell_read));
        self.reference_buffer = cell.reference;
        if let Some(value) = cell.value {
            self.value_buffer = value;
        }
        Ok(())
    }
}

/// Reads a worksheet part: its cells into a `SheetData`, and everything else
/// into a copy of the XML without them.
pub(super) fn read_sheet_data<R: BufRead>(
    reader: R,
    sheet_name: &str,
    parser: &mut Parser,
    shared_strings: &mut SharedStringTable,
) -> Result<SheetDataXml, XlsxError> {
    let mut reader = quick_xml::Reader::from_reader(reader);
    let mut tree = XmlTreeBuilder::new();
    let mut buffer = Vec::new();
    let mut in_sheet_data = false;
    let mut state = SheetDataReader {
        sheet_name,
        parser,
        shared_strings,
        sheet_data: SheetData::new(),
        rows: Vec::new(),
        shared_formulas: Vec::new(),
        formula_lookup: HashMap::new(),
        index_map: HashMap::new(),
        array_cell: HashMap::new(),
        stack: Vec::new(),
        row: None,
        cell: None,
        row_buffer: Vec::new(),
        reference_buffer: String::new(),
        value_buffer: String::new(),
    };
    loop {
        let event = reader.read_event_into(&mut buffer)?;
        match &event {
            Event::Eof => break,
            Event::Start(element) if !in_sheet_data && is_sheet_data(element) => {
                in_sheet_data = true;
            }
            Event::End(element)
                if in_sheet_data && state.stack.is_empty() && is_sheet_data_end(element) =>
            {
                in_sheet_data = false;
            }
            Event::Empty(element) if !in_sheet_data && is_sheet_data(element) => {}
            _ if in_sheet_data => state.process(&event)?,
            _ => tree.push(&event)?,
        }
        buffer.clear();
    }
    Ok(SheetDataXml {
        worksheet: tree.finish()?,
        sheet_data: state.sheet_data,
        rows: state.rows,
        shared_formulas: state.shared_formulas,
    })
}
