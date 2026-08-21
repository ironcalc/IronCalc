//! Local edits: a mutator call turned into patches.
//!
//! Every mutator validates first — ordinal semantics, so a bad row or a bad name errors before a
//! single patch exists — then resolves ordinals to keys, materializing missing rows and columns as
//! `InsertRows`/`InsertColumns` in the same commit as the write, and hands the lot to
//! [`CollabModel::commit_local`]. There is no second path into the document: for [`Stable`] every
//! mutation is a patch.

use crate::cf_types::{CfRuleInput, ConditionalFormattingView};
use crate::collab::fractional_index::{FractionalIndex, FractionalKey, KeyBuf};
use crate::collab::hlc::Hlc;
use crate::collab::log::Timestamp;
use crate::collab::model::{CollabModel, LocalCommit, Stable, StableCellAddress, StableRange};
use crate::collab::patch::{
    CellInput, CfProperty, ColPropKind, ColProperty, ColState, ColumnSnapshot,
    ConditionalFormatState, Patch, RowPropKind, RowProperty, RowSnapshot, RowState, SheetId,
    SheetPropKind, SheetProperty, WorkbookPropKind, WorkbookProperty,
};
use crate::constants::{
    COLUMN_WIDTH_FACTOR, DEFAULT_COLUMN_WIDTH, DEFAULT_ROW_HEIGHT, LAST_COLUMN, LAST_ROW,
    ROW_HEIGHT_FACTOR,
};
use crate::expressions::parser::stringify::{
    rename_defined_name_in_node, rename_sheet_in_node, to_english_string, to_rc_format,
    to_string_displaced, DisplaceData,
};
use crate::expressions::parser::Node;
use crate::expressions::token::get_error_by_name;
use crate::expressions::types::Area;
use crate::expressions::types::{CellReferenceIndex, CellReferenceRC};
use crate::expressions::utils::{is_valid_column_number, is_valid_identifier, is_valid_row};
use crate::formatter::format::parse_formatted_number;
use crate::formatter::lexer::is_likely_date_number_format;
use crate::locale::get_locale;
use crate::new_empty::is_valid_sheet_name;
use crate::types::{
    Cell, Color, Comment, Dxf, Position, RangeRef, SheetProperties, SheetState, Style, Theme,
};
use crate::tz::Tz;
use crate::utils as common;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

impl CollabModel<'_> {
    /// Applies `patches` as one commit and queues it for the log.
    ///
    /// One mutator call is one commit, minting one [`Hlc`]: every register it writes ends up with
    /// the same [`Timestamp`], which is what keeps the author's guard table equal to its peers'.
    pub(crate) fn commit_local(&mut self, patches: Vec<Patch>) {
        let hlc = Hlc::now();
        let ts = Timestamp::new(hlc, self.local.session);
        for patch in &patches {
            self.apply_patch(patch, &ts);
        }
        self.resync_parsed();
        self.local.pending.push(LocalCommit { hlc, patches });
    }

    /// Takes the commits produced since the last call, for the framework to transport.
    pub fn flush(&mut self) -> Vec<LocalCommit> {
        std::mem::take(&mut self.local.pending)
    }

    /// `(worksheet index, sheet id)`, erroring on an unknown sheet exactly as the ordinal model.
    fn sheet_of(&self, sheet: u32) -> Result<(usize, SheetId), String> {
        Ok((sheet as usize, self.workbook.worksheet(sheet)?.sheet_id))
    }

    /// A fresh sheet id, hashed from this replica's session and a slot past every id it has seen:
    /// session-dependent, so two peers creating a sheet concurrently do not mint the same id.
    /// Existence guards outlive their sheet, so a deleted id is never handed out twice.
    fn new_sheet_id(&self) -> SheetId {
        let seen = |id: &SheetId| {
            self.workbook.worksheets.iter().any(|ws| &ws.sheet_id == id)
                || self.workbook.meta.sheet_existence.contains_key(id)
        };
        let mut slot = u64::from(
            self.workbook
                .worksheets
                .iter()
                .map(|ws| ws.sheet_id)
                .chain(self.workbook.meta.sheet_existence.keys().copied())
                .max()
                .unwrap_or(0),
        ) + 1;
        loop {
            let mut hasher = DefaultHasher::new();
            (self.local.session, slot).hash(&mut hasher);
            let id = hasher.finish() as SheetId;
            // Never 0: the ordinal model starts its ids at 1 and readers may treat 0 as "no sheet".
            if id != 0 && !seen(&id) {
                return id;
            }
            slot += 1;
        }
    }

    /// A tab-order key past every sheet this replica has seen. The session suffix separates two
    /// replicas appending concurrently, so the order stays total.
    fn sheet_position(&self) -> FractionalKey {
        let slot = self.workbook.meta.sheet_existence.len() as u32 + 1;
        let mut buf = KeyBuf::from(&(2 * slot).to_be_bytes()[1..]);
        buf.extend_from_slice(&self.suffix());
        FractionalKey::try_from_bytes(&buf).expect("position and suffix are 7 bytes")
    }

    /// The key row `row` of worksheet `i` answers to, appending to `patches` whatever has to be
    /// materialized first for it to exist.
    fn row_key(&self, i: usize, id: SheetId, row: i32, patches: &mut Vec<Patch>) -> FractionalKey {
        let index = &self.workbook.worksheets[i].index;
        match Stable::row_at(index, row) {
            Some(key) => key,
            None => {
                let keys = index.rows.plan_virtual(row as usize);
                let key = keys.last().cloned().expect("row is past the index");
                patches.push(Patch::InsertRows { sheet: id, keys });
                key
            }
        }
    }

    /// [`Self::row_key`] for a column.
    fn col_key(
        &self,
        i: usize,
        id: SheetId,
        column: i32,
        patches: &mut Vec<Patch>,
    ) -> FractionalKey {
        let index = &self.workbook.worksheets[i].index;
        match Stable::col_at(index, column) {
            Some(key) => key,
            None => {
                let keys = index.cols.plan_virtual(column as usize);
                let key = keys.last().cloned().expect("column is past the index");
                patches.push(Patch::InsertColumns { sheet: id, keys });
                key
            }
        }
    }

    /// Keys for `(row, column)` on worksheet `i`. If row/column IDs have to be created (via virtual
    /// fractional key creation), they will be returned as vec of patches with the result.
    fn resolve_cell(
        &self,
        i: usize,
        id: SheetId,
        row: i32,
        column: i32,
    ) -> (StableCellAddress, Vec<Patch>) {
        let mut patches = Vec::new();
        let row_key = self.row_key(i, id, row, &mut patches);
        let col_key = self.col_key(i, id, column, &mut patches);
        ((row_key, col_key), patches)
    }

    /// The stable twin of an ordinal rectangle, materializing whatever it names. An unbounded axis
    /// stays unbounded: it tracks the sheet rather than a pair of corners.
    fn stable_range(
        &self,
        i: usize,
        id: SheetId,
        range: &RangeRef,
        patches: &mut Vec<Patch>,
    ) -> StableRange {
        let rows = range.rows.map(|(a, b)| {
            (
                self.row_key(i, id, a, patches),
                self.row_key(i, id, b, patches),
            )
        });
        let cols = range.cols.map(|(a, b)| {
            (
                self.col_key(i, id, a, patches),
                self.col_key(i, id, b, patches),
            )
        });
        StableRange { rows, cols }
    }

    /// The row register's current value for `kind`, as the property a write would replace. A row
    /// with no record holds the defaults, which is what undoing a first write has to put back.
    fn row_prev(&self, i: usize, key: &FractionalKey, kind: RowPropKind) -> Option<RowProperty> {
        let record = self.workbook.worksheets[i]
            .rows
            .iter()
            .find(|r| &r.r == key);
        Some(match kind {
            RowPropKind::Style => RowProperty::Style(
                record
                    .filter(|r| r.custom_format)
                    .and_then(|r| self.workbook.styles.get_style(r.s).ok())
                    .map(Box::new),
            ),
            RowPropKind::Height => RowProperty::Height(
                record.map_or(DEFAULT_ROW_HEIGHT / ROW_HEIGHT_FACTOR, |r| r.height),
            ),
            RowPropKind::Hidden => RowProperty::Hidden(record.is_some_and(|r| r.hidden)),
        })
    }

    /// [`Self::row_prev`] for the column span `(key, key)`. Wider spans covering the same column
    /// are not consulted: a point write replaces only the point register.
    fn col_prev(&self, i: usize, key: &FractionalKey, kind: ColPropKind) -> Option<ColProperty> {
        let record = self.workbook.worksheets[i]
            .cols
            .iter()
            .find(|c| &c.min == key && &c.max == key);
        Some(match kind {
            ColPropKind::Style => ColProperty::Style(
                record
                    .and_then(|c| c.style)
                    .and_then(|s| self.workbook.styles.get_style(s).ok())
                    .map(Box::new),
            ),
            ColPropKind::Width => ColProperty::Width(
                record.map_or(DEFAULT_COLUMN_WIDTH / COLUMN_WIDTH_FACTOR, |c| c.width),
            ),
            ColPropKind::Hidden => ColProperty::Hidden(record.is_some_and(|c| c.hidden)),
        })
    }

    /// The sheet property `kind` currently holds, as the property a write would replace.
    fn sheet_prev(&self, i: usize, kind: SheetPropKind) -> Option<SheetProperty> {
        let sheet = &self.workbook.worksheets[i];
        Some(match kind {
            SheetPropKind::Name => SheetProperty::Name(sheet.name.clone()),
            SheetPropKind::Color => SheetProperty::Color(sheet.color.clone()),
            SheetPropKind::State => SheetProperty::State(sheet.state.clone()),
            SheetPropKind::ShowGridLines => SheetProperty::ShowGridLines(sheet.show_grid_lines),
            SheetPropKind::FrozenRows => SheetProperty::FrozenRows(sheet.frozen_rows),
            SheetPropKind::FrozenColumns => SheetProperty::FrozenColumns(sheet.frozen_columns),
            SheetPropKind::Position => SheetProperty::Position(
                self.workbook
                    .meta
                    .sheet_positions
                    .get(&sheet.sheet_id)?
                    .0
                    .clone(),
            ),
        })
    }

    /// What the cell at `at` would be written back as: its authored contents, never its evaluated
    /// value. An array anchor and a spill cell have no input we can reproduce, so they read as
    /// nothing — see [`invert_patches`](crate::collab::patch::Patch).
    fn cell_input(&self, i: usize, at: &StableCellAddress) -> Option<CellInput> {
        let sheet = &self.workbook.worksheets[i];
        let cell = sheet.sheet_data.get(&at.0)?.get(&at.1)?;
        match cell {
            Cell::BooleanCell { v, .. } => Some(CellInput::Boolean(*v)),
            Cell::NumberCell { v, .. } => Some(CellInput::Number(*v)),
            Cell::ErrorCell { ei, .. } => Some(CellInput::Error(ei.clone())),
            Cell::SharedString { si, .. } => self
                .workbook
                .shared_strings
                .get(*si as usize)
                .cloned() //TODO: we can do better than String::clone
                .map(CellInput::Text),
            Cell::CellFormula { f, .. } => {
                let formula = sheet
                    .shared_formulas
                    .get(*f as usize)
                    .cloned()
                    .unwrap_or_default();
                Some(CellInput::Formula(formula))
            }
            Cell::EmptyCell { .. } | Cell::ArrayFormula { .. } | Cell::SpillCell { .. } => None,
        }
    }

    /// Parses `formula` as the user typed it, anchored at the cell it goes into, retrying with a
    /// closing parenthesis exactly as the ordinal path does.
    fn parse_at(&mut self, i: usize, row: i32, column: i32, formula: &str) -> Node {
        let context = CellReferenceRC {
            sheet: self.workbook.worksheets[i].get_name(),
            row,
            column,
        };
        let node = self.parser.parse(formula, &context);
        if let Node::ParseErrorKind { .. } = node {
            let retry = self.parser.parse(&format!("{formula})"), &context);
            if !matches!(retry, Node::ParseErrorKind { .. }) {
                return retry;
            }
        }
        node
    }

    /// The authored contents `value` denotes and the style the ordinal path would leave behind:
    /// the same classification order — quote prefix, formula, number, boolean, error, text.
    fn classify_input(
        &mut self,
        i: usize,
        sheet: u32,
        row: i32,
        column: i32,
        value: &str,
        mut style: Style,
    ) -> Result<(Option<CellInput>, Style), String> {
        if value.is_empty() {
            return Ok((None, style));
        }
        if let Some(text) = value.strip_prefix('\'') {
            style.quote_prefix = true;
            return Ok((Some(CellInput::Text(text.to_string())), style));
        }
        style.quote_prefix = false;
        if let Some(formula) = self.formula_without_prefix(value) {
            let formula = formula.to_string();
            let node = self.parse_at(i, row, column, &formula);
            let cell = CellReferenceIndex { sheet, row, column };
            if let Some(units) = self.compute_node_units(&node, &cell) {
                style.num_fmt = units.get_num_fmt();
            }
            return Ok((Some(CellInput::Formula(to_rc_format(&node))), style));
        }
        // The list of currencies is '$', '€' and the local currency
        let mut currencies = vec!["$", "€"];
        let currency = &self.locale.currency.symbol;
        if !currencies.iter().any(|e| e == currency) {
            currencies.push(currency);
        }
        if let Ok((v, number_format)) = parse_formatted_number(value, &currencies, self.locale) {
            if let Some(num_fmt) = number_format {
                // A date written into an already date-formatted cell keeps the cell's format.
                if !(is_likely_date_number_format(&style.num_fmt)
                    && is_likely_date_number_format(&num_fmt))
                {
                    style.num_fmt = num_fmt;
                }
            }
            return Ok((Some(CellInput::Number(v)), style));
        }
        if let Ok(v) = value.to_lowercase().parse::<bool>() {
            return Ok((Some(CellInput::Boolean(v)), style));
        }
        match get_error_by_name(&value.to_uppercase(), self.language) {
            Some(error) => Ok((Some(CellInput::Error(error)), style)),
            None => Ok((Some(CellInput::Text(value.to_string())), style)),
        }
    }

    /// The patches writing `input` into `(row, column)`, style included, and nothing else.
    ///
    /// A clear takes the cell with it, and the cell style is a register of its own, so the style is
    /// written back to keep the formatting a cleared cell keeps under ordinal addressing.
    fn write_patches(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        input: Option<CellInput>,
        style: Style,
    ) -> Result<Vec<Patch>, String> {
        let (i, id) = self.sheet_of(sheet)?;
        let (at, mut patches) = self.resolve_cell(i, id, row, column);
        let prev = self.cell_input(i, &at);
        let clearing = input.is_none();
        patches.push(Patch::SetCellValue {
            sheet: id,
            at: at.clone(),
            value: input,
            prev: Box::new(prev),
        });
        let stored = self.get_cell_style_or_none(sheet, row, column)?;
        let restore = if clearing { stored.clone() } else { None };
        if restore.is_some() || (!clearing && Some(&style) != stored.as_ref()) {
            let style = restore.unwrap_or(style);
            patches.push(Patch::SetCellStyle {
                sheet: id,
                at,
                style: Some(Box::new(style)),
                prev: Box::new(stored),
            });
        }
        Ok(patches)
    }

    /// Validates that `(sheet, row, column)` is addressable, as the ordinal writers do.
    fn check_cell(&self, sheet: u32, row: i32, column: i32) -> Result<usize, String> {
        let (i, _) = self.sheet_of(sheet)?;
        if !is_valid_row(row) || !is_valid_column_number(column) {
            return Err("Incorrect row or column".to_string());
        }
        Ok(i)
    }
}

/// Document mutation: the collaborative twin of the ordinal writers, patch by patch.
impl CollabModel<'_> {
    /// Adds a sheet with an automatically generated name.
    pub fn new_sheet(&mut self) -> (String, u32) {
        let base_name = self.get_sheet_name();
        let base_name_uppercase = base_name.to_uppercase();
        let mut index = 1;
        while self
            .workbook
            .get_worksheet_names()
            .iter()
            .map(|s| s.to_uppercase())
            .any(|x| x == format!("{base_name_uppercase}{index}"))
        {
            index += 1;
        }
        let name = format!("{base_name}{index}");
        let id = self.new_sheet_id();
        self.commit_local(vec![Patch::AddSheet {
            id,
            name: name.clone(),
            position: self.sheet_position(),
            content: None,
        }]);
        let at = self.get_sheet_index_by_sheet_id(id).unwrap_or_default();
        (name, at)
    }

    /// Sets a cell as if a user had typed `value` into it.
    pub fn set_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: String,
    ) -> Result<(), String> {
        let i = self.check_cell(sheet, row, column)?;
        let style = self.get_style_for_cell(sheet, row, column)?;
        let (input, style) = self.classify_input(i, sheet, row, column, &value, style)?;
        let patches = self.write_patches(sheet, row, column, input, style)?;
        self.commit_local(patches);
        Ok(())
    }

    /// Clears the contents of a cell, keeping its formatting.
    pub fn cell_clear_contents(&mut self, sheet: u32, row: i32, column: i32) -> Result<(), String> {
        self.check_cell(sheet, row, column)?;
        let style = self.get_style_for_cell(sheet, row, column)?;
        let patches = self.write_patches(sheet, row, column, None, style)?;
        self.commit_local(patches);
        Ok(())
    }

    /// Updates a cell with text, quoting it if it would otherwise read as something else.
    pub fn update_cell_with_text(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: &str,
    ) -> Result<(), String> {
        self.check_cell(sheet, row, column)?;
        let mut style = self.get_style_for_cell(sheet, row, column)?;
        style.quote_prefix = common::value_needs_quoting(value, self.language);
        let input = Some(CellInput::Text(value.to_string()));
        let patches = self.write_patches(sheet, row, column, input, style)?;
        self.commit_local(patches);
        Ok(())
    }

    /// Updates a cell with a boolean. The style is unchanged bar the quote prefix.
    pub fn update_cell_with_bool(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: bool,
    ) -> Result<(), String> {
        self.check_cell(sheet, row, column)?;
        let mut style = self.get_style_for_cell(sheet, row, column)?;
        style.quote_prefix = false;
        let patches =
            self.write_patches(sheet, row, column, Some(CellInput::Boolean(value)), style)?;
        self.commit_local(patches);
        Ok(())
    }

    /// Updates a cell with a number. The style is unchanged bar the quote prefix.
    pub fn update_cell_with_number(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: f64,
    ) -> Result<(), String> {
        self.check_cell(sheet, row, column)?;
        let mut style = self.get_style_for_cell(sheet, row, column)?;
        style.quote_prefix = false;
        let patches =
            self.write_patches(sheet, row, column, Some(CellInput::Number(value)), style)?;
        self.commit_local(patches);
        Ok(())
    }

    /// Updates a cell with a formula, which must start with `=`.
    pub fn update_cell_with_formula(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        formula: String,
    ) -> Result<(), String> {
        let i = self.check_cell(sheet, row, column)?;
        let mut style = self.get_style_for_cell(sheet, row, column)?;
        style.quote_prefix = false;
        let Some(body) = self.formula_without_prefix(&formula) else {
            return Err(format!("\"{formula}\" is not a valid formula"));
        };
        let body = body.to_string();
        let node = self.parse_at(i, row, column, &body);
        let input = Some(CellInput::Formula(to_rc_format(&node)));
        let patches = self.write_patches(sheet, row, column, input, style)?;
        self.commit_local(patches);
        Ok(())
    }

    /// Clears a cell's contents *and* its formatting.
    pub fn cell_clear_all(&mut self, sheet: u32, row: i32, column: i32) -> Result<(), String> {
        let i = self.check_cell(sheet, row, column)?;
        let (_, id) = self.sheet_of(sheet)?;
        let (at, mut patches) = self.resolve_cell(i, id, row, column);
        let prev = self.cell_input(i, &at);
        let style = self.get_cell_style_or_none(sheet, row, column)?;
        patches.push(Patch::SetCellValue {
            sheet: id,
            at: at.clone(),
            value: None,
            prev: Box::new(prev),
        });
        patches.push(Patch::SetCellStyle {
            sheet: id,
            at,
            style: None,
            prev: Box::new(style),
        });
        self.commit_local(patches);
        Ok(())
    }

    /// Sets the style of a cell.
    pub fn set_cell_style(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        style: &Style,
    ) -> Result<(), String> {
        let i = self.check_cell(sheet, row, column)?;
        let (_, id) = self.sheet_of(sheet)?;
        let (at, mut patches) = self.resolve_cell(i, id, row, column);
        let prev = self.get_cell_style_or_none(sheet, row, column)?;
        patches.push(Patch::SetCellStyle {
            sheet: id,
            at,
            style: Some(Box::new(style.clone())),
            prev: Box::new(prev),
        });
        self.commit_local(patches);
        Ok(())
    }

    /// Sets the named style `style_name` on a cell. Named styles are a local table, so the patch
    /// carries the style it resolves to.
    pub fn set_cell_style_by_name(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        style_name: &str,
    ) -> Result<(), String> {
        let index = self.workbook.styles.get_style_index_by_name(style_name)?;
        let style = self.workbook.styles.get_style(index)?;
        self.set_cell_style(sheet, row, column, &style)
    }

    /// Changes the height of a row.
    pub fn set_row_height(&mut self, sheet: u32, row: i32, height: f64) -> Result<(), String> {
        let (i, id) = self.sheet_of(sheet)?;
        if !is_valid_row(row) {
            return Err(format!("Row number '{row}' is not valid."));
        }
        if height < 0.0 {
            return Err(format!("Can not set a negative height: {height}"));
        }
        let mut patches = Vec::new();
        let key = self.row_key(i, id, row, &mut patches);
        let prev = self.row_prev(i, &key, RowPropKind::Height);
        patches.push(Patch::SetRowProperty {
            sheet: id,
            row: key,
            property: RowProperty::Height(height / ROW_HEIGHT_FACTOR),
            prev,
        });
        self.commit_local(patches);
        Ok(())
    }

    /// Changes the hidden status of a row.
    pub fn set_row_hidden(&mut self, sheet: u32, row: i32, hidden: bool) -> Result<(), String> {
        let (i, id) = self.sheet_of(sheet)?;
        if !is_valid_row(row) {
            return Err(format!("Row number '{row}' is not valid."));
        }
        let mut patches = Vec::new();
        let key = self.row_key(i, id, row, &mut patches);
        let prev = self.row_prev(i, &key, RowPropKind::Hidden);
        patches.push(Patch::SetRowProperty {
            sheet: id,
            row: key,
            property: RowProperty::Hidden(hidden),
            prev,
        });
        self.commit_local(patches);
        Ok(())
    }

    /// Sets the style of a whole row.
    pub fn set_row_style(&mut self, sheet: u32, row: i32, style: &Style) -> Result<(), String> {
        let (i, id) = self.sheet_of(sheet)?;
        if !is_valid_row(row) {
            return Err(format!("Row number '{row}' is not valid."));
        }
        let mut patches = Vec::new();
        let key = self.row_key(i, id, row, &mut patches);
        let prev = self.row_prev(i, &key, RowPropKind::Style);
        patches.push(Patch::SetRowProperty {
            sheet: id,
            row: key,
            property: RowProperty::Style(Some(Box::new(style.clone()))),
            prev,
        });
        self.commit_local(patches);
        Ok(())
    }

    /// The patch a single-column property write emits. v1 writes points, never spans: see the
    /// shattering paragraph in [`patch`](crate::collab::patch).
    fn column_patches(
        &self,
        sheet: u32,
        column: i32,
        property: ColProperty,
    ) -> Result<Vec<Patch>, String> {
        let (i, id) = self.sheet_of(sheet)?;
        if !is_valid_column_number(column) {
            return Err(format!("Column number '{column}' is not valid."));
        }
        let mut patches = Vec::new();
        let key = self.col_key(i, id, column, &mut patches);
        let prev = self.col_prev(i, &key, property.kind());
        patches.push(Patch::SetColumnSpan {
            sheet: id,
            span: (key.clone(), key),
            property,
            prev,
        });
        Ok(patches)
    }

    /// Changes the width of a column.
    pub fn set_column_width(&mut self, sheet: u32, column: i32, width: f64) -> Result<(), String> {
        if width < 0.0 {
            return Err(format!("Can not set a negative width: {width}"));
        }
        let property = ColProperty::Width(width / COLUMN_WIDTH_FACTOR);
        let patches = self.column_patches(sheet, column, property)?;
        self.commit_local(patches);
        Ok(())
    }

    /// Changes the hidden status of a column.
    pub fn set_column_hidden(
        &mut self,
        sheet: u32,
        column: i32,
        hidden: bool,
    ) -> Result<(), String> {
        let patches = self.column_patches(sheet, column, ColProperty::Hidden(hidden))?;
        self.commit_local(patches);
        Ok(())
    }

    /// Sets the style of a whole column.
    pub fn set_column_style(
        &mut self,
        sheet: u32,
        column: i32,
        style: &Style,
    ) -> Result<(), String> {
        let property = ColProperty::Style(Some(Box::new(style.clone())));
        let patches = self.column_patches(sheet, column, property)?;
        self.commit_local(patches);
        Ok(())
    }

    /// Writes a sheet-scoped property, validating nothing beyond the sheet existing.
    fn commit_sheet_property(&mut self, sheet: u32, property: SheetProperty) -> Result<(), String> {
        let (i, id) = self.sheet_of(sheet)?;
        let prev = self.sheet_prev(i, property.kind());
        self.commit_local(vec![Patch::SetSheetProperty {
            sheet: id,
            property,
            prev,
        }]);
        Ok(())
    }

    /// Sets the color of the sheet tab.
    pub fn set_sheet_color(&mut self, sheet: u32, color: &Color) -> Result<(), String> {
        self.commit_sheet_property(sheet, SheetProperty::Color(color.clone()))
    }

    /// Changes the visibility of a sheet.
    pub fn set_sheet_state(&mut self, sheet: u32, state: SheetState) -> Result<(), String> {
        self.commit_sheet_property(sheet, SheetProperty::State(state))
    }

    /// Makes the grid lines visible (`true`) or hidden (`false`).
    pub fn set_show_grid_lines(&mut self, sheet: u32, show: bool) -> Result<(), String> {
        self.commit_sheet_property(sheet, SheetProperty::ShowGridLines(show))
    }

    /// Freezes the first `frozen_rows` rows of a sheet.
    pub fn set_frozen_rows(&mut self, sheet: u32, frozen_rows: i32) -> Result<(), String> {
        if frozen_rows < 0 {
            return Err("Frozen rows cannot be negative".to_string());
        }
        if frozen_rows >= LAST_ROW {
            return Err("Too many rows".to_string());
        }
        self.commit_sheet_property(sheet, SheetProperty::FrozenRows(frozen_rows))
    }

    /// Freezes the first `frozen_columns` columns of a sheet.
    pub fn set_frozen_columns(&mut self, sheet: u32, frozen_columns: i32) -> Result<(), String> {
        if frozen_columns < 0 {
            return Err("Frozen columns cannot be negative".to_string());
        }
        if frozen_columns >= LAST_COLUMN {
            return Err("Too many columns".to_string());
        }
        self.commit_sheet_property(sheet, SheetProperty::FrozenColumns(frozen_columns))
    }

    /// Merges or unmerges the cells an ordinal rectangle covers.
    pub fn set_merged_range(
        &mut self,
        sheet: u32,
        range: &RangeRef,
        merged: bool,
    ) -> Result<(), String> {
        let (i, id) = self.sheet_of(sheet)?;
        let mut patches = Vec::new();
        let range = self.stable_range(i, id, range, &mut patches);
        let prev = self.workbook.worksheets[i]
            .merge_cells
            .iter()
            .any(|r| r == &range);
        patches.push(Patch::SetMergedRange {
            sheet: id,
            range,
            merged,
            prev,
        });
        self.commit_local(patches);
        Ok(())
    }

    /// Sets or removes the comment on a cell.
    pub fn set_comment(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        comment: Option<(String, String)>,
    ) -> Result<(), String> {
        let i = self.check_cell(sheet, row, column)?;
        let (_, id) = self.sheet_of(sheet)?;
        let (at, mut patches) = self.resolve_cell(i, id, row, column);
        let prev = self.workbook.worksheets[i]
            .comments
            .iter()
            .find(|c| c.cell_ref == at)
            .cloned();
        let comment = comment.map(|(text, author_name)| Comment::<Stable> {
            text,
            author_name,
            author_id: None,
            cell_ref: at.clone(),
        });
        patches.push(Patch::SetComment {
            sheet: id,
            at,
            comment,
            prev,
        });
        self.commit_local(patches);
        Ok(())
    }

    /// Writes a workbook-global property.
    fn commit_workbook_property(&mut self, property: WorkbookProperty) {
        let prev = match property.kind() {
            WorkbookPropKind::Theme => {
                WorkbookProperty::Theme(Box::new(self.workbook.theme.clone()))
            }
            WorkbookPropKind::Locale => {
                WorkbookProperty::Locale(self.workbook.settings.locale.clone())
            }
            WorkbookPropKind::Timezone => {
                WorkbookProperty::Timezone(self.workbook.settings.tz.clone())
            }
        };
        self.commit_local(vec![Patch::SetWorkbookProperty {
            property,
            prev: Some(prev),
        }]);
    }

    /// Sets the workbook theme.
    pub fn set_theme(&mut self, theme: Theme) {
        self.commit_workbook_property(WorkbookProperty::Theme(Box::new(theme)));
        self.evaluate_conditional_formatting();
    }

    /// Sets the workbook locale.
    pub fn set_locale(&mut self, locale_id: &str) -> Result<(), String> {
        let Ok(locale) = get_locale(locale_id) else {
            return Err(format!("Invalid locale: {locale_id}"));
        };
        self.parser.set_locale(locale);
        self.locale = locale;
        self.commit_workbook_property(WorkbookProperty::Locale(locale_id.to_string()));
        self.evaluate();
        Ok(())
    }

    /// Sets the workbook timezone.
    pub fn set_timezone(&mut self, timezone: &str) -> Result<(), String> {
        let Ok(tz) = Tz::parse(timezone) else {
            return Err(format!("Invalid timezone: {timezone}"));
        };
        self.tz = tz;
        self.commit_workbook_property(WorkbookProperty::Timezone(timezone.to_string()));
        self.evaluate();
        Ok(())
    }
}

/// Structural edits, and the formula rewriting they drag along.
///
/// Rewriting happens on the author only: it turns into ordinary `SetCellValue`/`SetDefinedName`
/// patches shipped in the same commit as the structural ones, so a replica receiving them applies
/// patches and rewrites nothing.
impl CollabModel<'_> {
    /// Every formula cell as `(worksheet, sheet id, ordinal row, ordinal column, address, formula
    /// index)`, in ordinal order so the patches a rewrite emits do not depend on hash iteration.
    fn formula_cells(&self) -> Vec<(usize, SheetId, i32, i32, StableCellAddress, i32)> {
        let mut cells = Vec::new();
        for (i, ws) in self.workbook.worksheets.iter().enumerate() {
            for (row_key, row_data) in &ws.sheet_data {
                let Some(row) = Stable::row_ordinal(&ws.index, row_key) else {
                    continue;
                };
                for (col_key, cell) in row_data {
                    let (Some(column), Some(f)) =
                        (Stable::col_ordinal(&ws.index, col_key), cell.get_formula())
                    else {
                        continue;
                    };
                    let at = (row_key.clone(), col_key.clone());
                    cells.push((i, ws.sheet_id, row, column, at, f));
                }
            }
        }
        cells.sort_by_key(|(i, _, row, column, ..)| (*i, *row, *column));
        cells
    }

    /// Re-emits every formula whose stored text a structural edit changes.
    ///
    /// Two anchors are in play, and both matter. The A1 text a formula *means* is rendered against
    /// the anchor it sits at **now** — that is where its R1C1 offsets resolve, and what the
    /// displacement is computed over. The result is then parsed back against the anchor it will sit
    /// at **once the patches apply**, which `anchor` supplies and which drops the cells the edit
    /// removes. Ordinal addressing gets the same two anchors by moving the cell first and
    /// rewriting after; identity addressing has to name them.
    fn displace_formulas(
        &mut self,
        displace: &DisplaceData,
        anchor: impl Fn(usize, i32, i32) -> Option<(i32, i32)>,
    ) -> Vec<Patch> {
        let mut patches = Vec::new();
        for (i, id, row, column, at, f) in self.formula_cells() {
            let Some((moved_row, moved_column)) = anchor(i, row, column) else {
                continue;
            };
            let Some(node) = self
                .parsed_formulas
                .get(i)
                .and_then(|sheet| sheet.get(f as usize))
                .map(|(node, _)| node.clone())
            else {
                continue;
            };
            let context = CellReferenceRC {
                sheet: self.workbook.worksheets[i].get_name(),
                row,
                column,
            };
            let displaced = to_string_displaced(&node, &context, displace);
            let parsed = self.parse_at(i, moved_row, moved_column, &displaced);
            let formula = to_rc_format(&parsed);
            if self.workbook.worksheets[i].shared_formulas.get(f as usize) == Some(&formula) {
                continue;
            }
            let prev = self.cell_input(i, &at);
            patches.push(Patch::SetCellValue {
                sheet: id,
                at,
                value: Some(CellInput::Formula(formula)),
                prev: Box::new(prev),
            });
        }
        patches
    }

    /// Re-emits every defined name whose formula `displace` changes.
    fn displace_defined_names(&mut self, displace: &DisplaceData) -> Vec<Patch> {
        let context = self.defined_name_context();
        let names: Vec<(Option<SheetId>, String, String)> = self
            .workbook
            .defined_names
            .iter()
            .map(|dn| (dn.sheet_id, dn.name.clone(), dn.formula.clone()))
            .collect();
        let mut patches = Vec::new();
        for (scope, name, formula) in names {
            let body = formula.strip_prefix('=').unwrap_or(&formula).to_string();
            let node = self.parse_internal_formula(&body, &context);
            let displaced = to_string_displaced(&node, &context, displace);
            if displaced == body {
                continue;
            }
            patches.push(Patch::SetDefinedName {
                scope,
                name,
                formula: Some(displaced),
                prev: Some(formula),
            });
        }
        patches
    }

    /// The keys `count` rows or columns inserted at ordinal `at` take, together with whatever had
    /// to be materialized to reach that far. Empty when the axis has no room to name them.
    fn insert_keys(index: &FractionalIndex, at: i32, count: i32) -> Vec<FractionalKey> {
        let (at, count) = (at as usize, count as usize);
        if at > index.len() {
            return index.plan_virtual(at - 1 + count);
        }
        index.create_keys(at - 1, count).collect()
    }

    /// Everything the rows `keys` name is about to lose, so that undo can put it back.
    fn row_snapshots(&self, i: usize, keys: &[FractionalKey]) -> Vec<RowSnapshot> {
        let sheet = &self.workbook.worksheets[i];
        keys.iter()
            .map(|key| {
                let row = sheet.rows.iter().find(|r| &r.r == key);
                let state = match row {
                    Some(row) => RowState {
                        height: row.height,
                        hidden: row.hidden,
                        style: if row.custom_format {
                            self.workbook.styles.get_style(row.s).ok().map(Box::new)
                        } else {
                            None
                        },
                        custom_height: row.custom_height,
                        custom_format: row.custom_format,
                    },
                    None => RowState::default(),
                };
                let (cell_values, cell_styles) = self.cell_snapshots(
                    i,
                    sheet
                        .sheet_data
                        .get(key)
                        .into_iter()
                        .flatten()
                        .map(|(col, cell)| {
                            ((key.clone(), col.clone()), col.clone(), cell.get_style())
                        }),
                );
                RowSnapshot {
                    key: key.clone(),
                    state,
                    cell_values,
                    cell_styles,
                }
            })
            .collect()
    }

    /// [`Self::row_snapshots`] for columns; the cells are keyed by row instead.
    fn column_snapshots(&self, i: usize, keys: &[FractionalKey]) -> Vec<ColumnSnapshot> {
        let sheet = &self.workbook.worksheets[i];
        keys.iter()
            .map(|key| {
                let col = sheet.cols.iter().find(|c| &c.min == key && &c.max == key);
                let state = match col {
                    Some(col) => ColState {
                        width: col.width,
                        hidden: col.hidden,
                        style: match col.style {
                            Some(s) => self.workbook.styles.get_style(s).ok().map(Box::new),
                            None => None,
                        },
                        custom_width: col.custom_width,
                    },
                    None => ColState::default(),
                };
                let (cell_values, cell_styles) = self.cell_snapshots(
                    i,
                    sheet.sheet_data.iter().filter_map(|(row, cells)| {
                        let cell = cells.get(key)?;
                        Some(((row.clone(), key.clone()), row.clone(), cell.get_style()))
                    }),
                );
                ColumnSnapshot {
                    key: key.clone(),
                    state,
                    cell_values,
                    cell_styles,
                }
            })
            .collect()
    }

    /// The authored contents and the styles of the cells `cells` names, keyed and sorted by the
    /// axis the caller is snapshotting along.
    #[allow(clippy::type_complexity)]
    fn cell_snapshots(
        &self,
        i: usize,
        cells: impl Iterator<Item = (StableCellAddress, FractionalKey, i32)>,
    ) -> (Vec<(FractionalKey, CellInput)>, Vec<(FractionalKey, Style)>) {
        let mut values = Vec::new();
        let mut styles = Vec::new();
        let mut cells: Vec<_> = cells.collect();
        cells.sort_by(|(_, a_axis, _), (_, b_axis, _)| a_axis.cmp(b_axis));
        for (at, key, style) in cells {
            if let Some(input) = self.cell_input(i, &at) {
                values.push((key.clone(), input));
            }
            if style != 0 {
                if let Ok(style) = self.workbook.styles.get_style(style) {
                    styles.push((key, style));
                }
            }
        }
        (values, styles)
    }

    /// Inserts `row_count` rows above `row`, displacing the formulas that referenced across it.
    pub fn insert_rows(&mut self, sheet: u32, row: i32, row_count: i32) -> Result<(), String> {
        let (i, id) = self.sheet_of(sheet)?;
        if row_count <= 0 {
            return Err("Cannot add a negative number of cells :)".to_string());
        }
        if !(1..=LAST_ROW).contains(&row) {
            return Err(format!("Row number '{row}' is not valid."));
        }
        let keys = Self::insert_keys(&self.workbook.worksheets[i].index.rows, row, row_count);
        if keys.is_empty() {
            return Err("Cannot insert rows there".to_string());
        }
        let mut patches = vec![Patch::InsertRows { sheet: id, keys }];
        let displace = DisplaceData::Row {
            sheet,
            row,
            delta: row_count,
        };
        let moved = |s: usize, r: i32, c: i32| {
            Some((if s == i && r >= row { r + row_count } else { r }, c))
        };
        patches.extend(self.displace_formulas(&displace, moved));
        patches.extend(self.displace_defined_names(&displace));
        self.commit_local(patches);
        Ok(())
    }

    /// Deletes `row_count` rows starting at `row`, displacing the formulas that referenced them.
    pub fn delete_rows(&mut self, sheet: u32, row: i32, row_count: i32) -> Result<(), String> {
        let (i, id) = self.sheet_of(sheet)?;
        if row_count <= 0 {
            return Err("Please use insert rows instead".to_string());
        }
        if !(1..=LAST_ROW).contains(&row) {
            return Err(format!("Row number '{row}' is not valid."));
        }
        let index = &self.workbook.worksheets[i].index;
        let keys: Vec<FractionalKey> = (row..row + row_count)
            .filter_map(|r| Stable::row_at(index, r))
            .collect();
        let prev = self.row_snapshots(i, &keys);
        let mut patches = vec![Patch::DeleteRows {
            sheet: id,
            keys,
            prev,
        }];
        let displace = DisplaceData::Row {
            sheet,
            row,
            delta: -row_count,
        };
        let moved = |s: usize, r: i32, c: i32| match () {
            _ if s != i || r < row => Some((r, c)),
            _ if r < row + row_count => None,
            _ => Some((r - row_count, c)),
        };
        patches.extend(self.displace_formulas(&displace, moved));
        patches.extend(self.displace_defined_names(&displace));
        self.commit_local(patches);
        Ok(())
    }

    /// Inserts `column_count` columns before `column`.
    pub fn insert_columns(
        &mut self,
        sheet: u32,
        column: i32,
        column_count: i32,
    ) -> Result<(), String> {
        let (i, id) = self.sheet_of(sheet)?;
        if column_count <= 0 {
            return Err("Cannot add a negative number of cells :)".to_string());
        }
        if !(1..=LAST_COLUMN).contains(&column) {
            return Err(format!("Column number '{column}' is not valid."));
        }
        let keys = Self::insert_keys(
            &self.workbook.worksheets[i].index.cols,
            column,
            column_count,
        );
        if keys.is_empty() {
            return Err("Cannot insert columns there".to_string());
        }
        let mut patches = vec![Patch::InsertColumns { sheet: id, keys }];
        let displace = DisplaceData::Column {
            sheet,
            column,
            delta: column_count,
        };
        let moved = |s: usize, r: i32, c: i32| {
            Some((
                r,
                if s == i && c >= column {
                    c + column_count
                } else {
                    c
                },
            ))
        };
        patches.extend(self.displace_formulas(&displace, moved));
        patches.extend(self.displace_defined_names(&displace));
        self.commit_local(patches);
        Ok(())
    }

    /// Deletes `column_count` columns starting at `column`.
    pub fn delete_columns(
        &mut self,
        sheet: u32,
        column: i32,
        column_count: i32,
    ) -> Result<(), String> {
        let (i, id) = self.sheet_of(sheet)?;
        if column_count <= 0 {
            return Err("Please use insert columns instead".to_string());
        }
        if !(1..=LAST_COLUMN).contains(&column) {
            return Err(format!("Column number '{column}' is not valid."));
        }
        let index = &self.workbook.worksheets[i].index;
        let keys: Vec<FractionalKey> = (column..column + column_count)
            .filter_map(|c| Stable::col_at(index, c))
            .collect();
        let prev = self.column_snapshots(i, &keys);
        let mut patches = vec![Patch::DeleteColumns {
            sheet: id,
            keys,
            prev,
        }];
        let displace = DisplaceData::Column {
            sheet,
            column,
            delta: -column_count,
        };
        let moved = |s: usize, r: i32, c: i32| match () {
            _ if s != i || c < column => Some((r, c)),
            _ if c < column + column_count => None,
            _ => Some((r, c - column_count)),
        };
        patches.extend(self.displace_formulas(&displace, moved));
        patches.extend(self.displace_defined_names(&displace));
        self.commit_local(patches);
        Ok(())
    }

    /// Deletes a sheet by index. Fails if it is the last one.
    pub fn delete_sheet(&mut self, sheet: u32) -> Result<(), String> {
        if self.workbook.worksheets.len() == 1 {
            return Err("Cannot delete only sheet".to_string());
        }
        let (_, id) = self.sheet_of(sheet)?;
        self.commit_local(vec![Patch::DeleteSheet {
            sheet: id,
            prev: None,
        }]);
        Ok(())
    }

    /// Renames a sheet, rewriting every formula and defined name that named it.
    pub fn rename_sheet_by_index(&mut self, sheet: u32, new_name: &str) -> Result<(), String> {
        let (i, id) = self.sheet_of(sheet)?;
        if !is_valid_sheet_name(new_name) {
            return Err(format!("Invalid name for a sheet: '{new_name}'."));
        }
        if self
            .get_sheet_index_by_name(new_name)
            .is_some_and(|found| found != sheet)
        {
            return Err(format!("Sheet already exists: '{new_name}'."));
        }
        let prev = self.sheet_prev(i, SheetPropKind::Name);
        let mut patches = vec![Patch::SetSheetProperty {
            sheet: id,
            property: SheetProperty::Name(new_name.to_string()),
            prev,
        }];
        // Stored formulas are R1C1 and position-independent, so a rename is a rewrite of the text
        // alone — no anchor is involved.
        for (j, id, _, _, at, f) in self.formula_cells() {
            let Some(mut node) = self
                .parsed_formulas
                .get(j)
                .and_then(|s| s.get(f as usize))
                .map(|(node, _)| node.clone())
            else {
                continue;
            };
            rename_sheet_in_node(&mut node, sheet, new_name);
            let renamed = to_rc_format(&node);
            if self.workbook.worksheets[j].shared_formulas.get(f as usize) == Some(&renamed) {
                continue;
            }
            let prev = self.cell_input(j, &at);
            patches.push(Patch::SetCellValue {
                sheet: id,
                at,
                value: Some(CellInput::Formula(renamed)),
                prev: Box::new(prev),
            });
        }
        let context = self.defined_name_context();
        let names: Vec<(Option<SheetId>, String, String)> = self
            .workbook
            .defined_names
            .iter()
            .map(|dn| (dn.sheet_id, dn.name.clone(), dn.formula.clone()))
            .collect();
        for (scope, name, formula) in names {
            let body = formula.strip_prefix('=').unwrap_or(&formula).to_string();
            let mut node = self.parse_internal_formula(&body, &context);
            rename_sheet_in_node(&mut node, sheet, new_name);
            let renamed = to_english_string(&node, &context);
            if renamed == body {
                continue;
            }
            patches.push(Patch::SetDefinedName {
                scope,
                name,
                formula: Some(renamed),
                prev: Some(formula),
            });
        }
        self.commit_local(patches);
        Ok(())
    }

    /// Renames a sheet found by its current name.
    pub fn rename_sheet(&mut self, old_name: &str, new_name: &str) -> Result<(), String> {
        match self.get_sheet_index_by_name(old_name) {
            Some(sheet) => self.rename_sheet_by_index(sheet, new_name),
            None => Err(format!("Could not find sheet {old_name}")),
        }
    }
}

/// Defined names and conditional formatting.
impl CollabModel<'_> {
    /// The sheet id `scope` names, and an error if it names nothing.
    fn scope_id(&self, scope: Option<u32>) -> Result<Option<SheetId>, String> {
        match scope {
            Some(index) => Ok(Some(
                self.workbook
                    .worksheet(index)
                    .map_err(|_| "Scope: Invalid sheet index")?
                    .sheet_id,
            )),
            None => Ok(None),
        }
    }

    /// Adds a defined name, global when `scope` is `None`.
    pub fn new_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        formula: &str,
    ) -> Result<(), String> {
        if !is_valid_identifier(name) {
            return Err("Name: Invalid defined name".to_string());
        }
        let sheet_id = self.scope_id(scope)?;
        let upper = name.to_uppercase();
        if self
            .workbook
            .defined_names
            .iter()
            .any(|dn| dn.name.to_uppercase() == upper && dn.sheet_id == sheet_id)
        {
            return Err("Name: Defined name already exists".to_string());
        }
        let context = self.defined_name_context();
        let formula = self.user_formula_to_internal(formula, &context)?;
        self.commit_local(vec![Patch::SetDefinedName {
            scope: sheet_id,
            name: name.to_string(),
            formula: Some(formula),
            prev: None,
        }]);
        Ok(())
    }

    /// The stored `(name, scope, formula)` of the defined name `name` has in `scope`.
    fn defined_name(&self, name: &str, scope: Option<SheetId>) -> Option<(String, String)> {
        let upper = name.to_uppercase();
        self.workbook
            .defined_names
            .iter()
            .rev()
            .find(|dn| dn.name.to_uppercase() == upper && dn.sheet_id == scope)
            .map(|dn| (dn.name.clone(), dn.formula.clone()))
    }

    /// Deletes a defined name.
    pub fn delete_defined_name(&mut self, name: &str, scope: Option<u32>) -> Result<(), String> {
        let sheet_id = self.scope_id(scope)?;
        let Some((name, formula)) = self.defined_name(name, sheet_id) else {
            return Err("Defined name not found".to_string());
        };
        self.commit_local(vec![Patch::SetDefinedName {
            scope: sheet_id,
            name,
            formula: None,
            prev: Some(formula),
        }]);
        Ok(())
    }

    /// Updates a defined name. A rename is a delete of the old register and a write of the new one,
    /// so two peers renaming concurrently end up with both names.
    pub fn update_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        new_name: &str,
        new_scope: Option<u32>,
        new_formula: &str,
    ) -> Result<(), String> {
        if !is_valid_identifier(new_name) {
            return Err("Name: Invalid defined name".to_string());
        }
        let sheet_id = self.scope_id(scope)?;
        let new_sheet_id = self.scope_id(new_scope)?;
        let renaming = name.to_uppercase() != new_name.to_uppercase() || scope != new_scope;
        if renaming && self.defined_name(new_name, new_sheet_id).is_some() {
            return Err("Name: Defined name already exists".to_string());
        }
        let Some((old_name, old_formula)) = self.defined_name(name, sheet_id) else {
            return Err("Defined name not found".to_string());
        };
        let context = self.defined_name_context();
        let formula = self.user_formula_to_internal(new_formula, &context)?;
        let mut patches = Vec::new();
        if renaming {
            patches.push(Patch::SetDefinedName {
                scope: sheet_id,
                name: old_name,
                formula: None,
                prev: Some(old_formula.clone()),
            });
            // Every formula naming it has to follow, or it would point at a name that is gone.
            for (j, id, _, _, at, f) in self.formula_cells() {
                let Some(mut node) = self
                    .parsed_formulas
                    .get(j)
                    .and_then(|s| s.get(f as usize))
                    .map(|(node, _)| node.clone())
                else {
                    continue;
                };
                rename_defined_name_in_node(&mut node, name, scope, new_name);
                let renamed = to_rc_format(&node);
                if self.workbook.worksheets[j].shared_formulas.get(f as usize) == Some(&renamed) {
                    continue;
                }
                let prev = self.cell_input(j, &at);
                patches.push(Patch::SetCellValue {
                    sheet: id,
                    at,
                    value: Some(CellInput::Formula(renamed)),
                    prev: Box::new(prev),
                });
            }
        }
        patches.push(Patch::SetDefinedName {
            scope: new_sheet_id,
            name: new_name.to_string(),
            formula: Some(formula),
            prev: if !renaming { Some(old_formula) } else { None },
        });
        self.commit_local(patches);
        Ok(())
    }

    /// Adds a conditional formatting rule to `sheet`. The key it is minted with is both its
    /// identity and its priority, so the returned priority is its position in storage order.
    pub fn add_conditional_formatting(
        &mut self,
        sheet: u32,
        range: &str,
        rule: CfRuleInput,
    ) -> Result<u32, String> {
        let (i, id) = self.sheet_of(sheet)?;
        let ordinal = RangeRef::parse_sqref(range);
        if ordinal.is_empty() {
            return Err(format!("Invalid conditional formatting range: '{range}'"));
        }
        let mut rule = rule;
        self.cf_rule_input_to_internal(&mut rule, sheet)?;
        let rule = self.cf_rule_from_input(rule);
        let mut patches = Vec::new();
        let ranges = ordinal
            .iter()
            .map(|r| self.stable_range(i, id, r, &mut patches))
            .collect();
        let key = self.cf_key(i);
        patches.push(Patch::AddConditionalFormat {
            sheet: id,
            key: key.clone(),
            rule: Box::new(rule),
            ranges,
        });
        self.commit_local(patches);
        let order = &self.workbook.worksheets[i].index.registers.cf_order;
        Ok(order.binary_search(&key).map_or(0, |at| at as u32 + 1))
    }

    /// A rule key past every rule this sheet holds, suffixed with this replica's session.
    fn cf_key(&self, i: usize) -> FractionalKey {
        let order = &self.workbook.worksheets[i].index.registers.cf_order;
        let mut buf = KeyBuf::from(&(2 * (order.len() as u32 + 1)).to_be_bytes()[1..]);
        buf.extend_from_slice(&self.suffix());
        FractionalKey::try_from_bytes(&buf).expect("position and suffix are 7 bytes")
    }

    /// Removes the conditional formatting rule at `index`.
    pub fn delete_conditional_formatting(
        &mut self,
        sheet: u32,
        index: usize,
    ) -> Result<(), String> {
        let (i, id) = self.sheet_of(sheet)?;
        let ws = &self.workbook.worksheets[i];
        let Some(key) = ws.index.registers.cf_order.get(index).cloned() else {
            return Err(format!(
                "Conditional formatting index {index} out of bounds"
            ));
        };
        let prev = ws
            .conditional_formatting
            .get(index)
            .map(|cf| ConditionalFormatState {
                rule: cf.cf_rule.clone(),
                ranges: cf.ranges.clone(),
            })
            .map(Box::new);
        self.commit_local(vec![Patch::DeleteConditionalFormat {
            sheet: id,
            key,
            prev,
        }]);
        Ok(())
    }

    /// Replaces the range and the rule of the conditional formatting entry at `index`.
    pub fn update_conditional_formatting(
        &mut self,
        sheet: u32,
        index: usize,
        new_range: &str,
        new_rule: CfRuleInput,
    ) -> Result<(), String> {
        let (i, id) = self.sheet_of(sheet)?;
        let ordinal = RangeRef::parse_sqref(new_range);
        if ordinal.is_empty() {
            return Err(format!(
                "Invalid conditional formatting range: '{new_range}'"
            ));
        }
        let Some(key) = self.workbook.worksheets[i]
            .index
            .registers
            .cf_order
            .get(index)
            .cloned()
        else {
            return Err(format!(
                "Conditional formatting index {index} out of bounds"
            ));
        };
        let mut new_rule = new_rule;
        self.cf_rule_input_to_internal(&mut new_rule, sheet)?;
        let rule = self.cf_rule_from_input(new_rule);
        let mut patches = Vec::new();
        let ranges = ordinal
            .iter()
            .map(|r| self.stable_range(i, id, r, &mut patches))
            .collect();
        let old = &self.workbook.worksheets[i].conditional_formatting[index];
        let (old_rule, old_ranges) = (old.cf_rule.clone(), old.ranges.clone());
        patches.push(Patch::SetConditionalFormat {
            sheet: id,
            key: key.clone(),
            property: CfProperty::Rule(Box::new(rule)),
            prev: Some(CfProperty::Rule(Box::new(old_rule))),
        });
        patches.push(Patch::SetConditionalFormat {
            sheet: id,
            key,
            property: CfProperty::Ranges(ranges),
            prev: Some(CfProperty::Ranges(old_ranges)),
        });
        self.commit_local(patches);
        Ok(())
    }
}

/// Reads the ordinal model answers from plain worksheet fields, which stable addressing answers
/// the same way.
impl CollabModel<'_> {
    pub fn get_frozen_rows_count(&self, sheet: u32) -> Result<i32, String> {
        Ok(self.workbook.worksheet(sheet)?.frozen_rows)
    }

    pub fn get_frozen_columns_count(&self, sheet: u32) -> Result<i32, String> {
        Ok(self.workbook.worksheet(sheet)?.frozen_columns)
    }

    pub fn get_worksheets_properties(&self) -> Vec<SheetProperties> {
        self.workbook
            .worksheets
            .iter()
            .map(|worksheet| SheetProperties {
                name: worksheet.get_name(),
                state: worksheet.state.to_string(),
                color: worksheet.color.clone(),
                sheet_id: worksheet.sheet_id,
            })
            .collect()
    }

    pub fn get_named_style(&self, name: &str) -> Result<Style, String> {
        let xf_id = self.workbook.styles.get_style_index_by_name(name)?;
        self.workbook.styles.get_style(xf_id)
    }

    pub fn get_named_style_list(&self) -> Vec<String> {
        self.workbook.styles.get_named_style_list()
    }
}

/// Calls phase 5b does not answer. They validate nothing and change nothing: the caller gets an
/// error rather than an edit that would never reach a peer.
macro_rules! unsupported {
    (&self $( $name:ident ( $( $arg:ident : $ty:ty ),* ) -> $ret:ty; )*) => {
        impl CollabModel<'_> {
            $(
                #[allow(unused_variables, clippy::too_many_arguments)]
                pub fn $name(&self, $( $arg: $ty ),*) -> Result<$ret, String> {
                    Err(UNSUPPORTED.to_string())
                }
            )*
        }
    };
    (&mut self $( $name:ident ( $( $arg:ident : $ty:ty ),* ) -> $ret:ty; )*) => {
        impl CollabModel<'_> {
            $(
                #[allow(unused_variables, clippy::too_many_arguments)]
                pub fn $name(&mut self, $( $arg: $ty ),*) -> Result<$ret, String> {
                    Err(UNSUPPORTED.to_string())
                }
            )*
        }
    };
}

/// What every stub above returns.
const UNSUPPORTED: &str = "unsupported in collab mode";

unsupported! { &self
    get_sheet_markup(sheet: u32) -> String;
    get_dxf_for_conditional_formatting(sheet: u32, index: usize) -> Option<Dxf>;
    get_conditional_formatting_list(sheet: u32) -> Vec<ConditionalFormattingView>;
}

unsupported! { &mut self
    add_sheet(name: &str) -> ();
    insert_sheet(name: &str, index: u32, sheet_id: Option<u32>) -> ();
    delete_sheet_by_name(name: &str) -> ();
    duplicate_sheet(source: u32) -> (String, u32);
    set_language(language_id: &str) -> ();
    set_user_array_formula(sheet: u32, row: i32, column: i32, width: i32, height: i32, value: &str) -> ();
    range_clear_contents(area: &Area) -> ();
    range_clear_all(area: &Area) -> ();
    move_cell_value_to_area(value: &str, source: &CellReferenceIndex, target: &CellReferenceIndex, area: &Area) -> String;
    extend_to(sheet: u32, row: i32, column: i32, target_row: i32, target_column: i32) -> String;
    extend_copied_value(value: &str, source: &CellReferenceIndex, target: &CellReferenceIndex) -> String;
    move_rows_action(sheet: u32, row: i32, row_count: i32, delta: i32) -> ();
    move_columns_action(sheet: u32, column: i32, column_count: i32, delta: i32) -> ();
    delete_row_style(sheet: u32, row: i32) -> ();
    delete_column_style(sheet: u32, column: i32) -> ();
    copy_cell_style(source: (u32, i32, i32), destination: (u32, i32, i32)) -> ();
    create_named_style(name: &str, style: &Style) -> ();
    delete_named_style(name: &str) -> ();
    update_named_style(name: &str, new_name: &str, style: &Style) -> (i32, i32);
    raise_conditional_formatting_priority(sheet: u32, index: usize) -> ();
    lower_conditional_formatting_priority(sheet: u32, index: usize) -> ();
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::collab::log::{Commit, CommitId, Consumer, SessionId};
    use crate::collab::patch::invert_patches;

    /// Replays what a framework would transport: each [`LocalCommit`] as one commit, carrying the
    /// stamp its author minted.
    fn deliver(model: &mut CollabModel<'_>, session: SessionId, commits: &[LocalCommit]) {
        for (i, commit) in commits.iter().enumerate() {
            model
                .apply(Commit {
                    id: &CommitId::from([i as u8].as_slice()),
                    session: &session,
                    hlc: commit.hlc,
                    patches: &commit.patches,
                })
                .unwrap();
        }
    }

    /// The full loop: emit locally, ship, converge. Both replicas must end up with the same
    /// document — registers included — and evaluate to the same values.
    #[test]
    fn emission_loop() {
        let mut a = CollabModel::new(1);
        let (name, sheet) = a.new_sheet();
        assert_eq!((name.as_str(), sheet), ("Sheet1", 0));

        a.set_user_input(0, 1, 1, "10".to_string()).unwrap();
        a.set_user_input(0, 2, 1, "20".to_string()).unwrap();
        a.set_user_input(0, 3, 1, "text".to_string()).unwrap();
        a.set_user_input(0, 4, 1, "TRUE".to_string()).unwrap();
        a.set_user_input(0, 1, 2, "=A1+A2".to_string()).unwrap();
        a.set_user_input(0, 2, 2, "=B1*2".to_string()).unwrap();
        a.set_user_input(0, 3, 2, "=CONCAT(A3, \"!\")".to_string())
            .unwrap();
        // Far past anything materialized: the write carries the rows it needs with it.
        a.set_user_input(0, 500, 3, "=B1".to_string()).unwrap();
        a.update_cell_with_number(0, 5, 1, 7.5).unwrap();
        a.update_cell_with_text(0, 6, 1, "=not a formula").unwrap();
        a.update_cell_with_bool(0, 7, 1, false).unwrap();
        a.update_cell_with_formula(0, 8, 1, "=A1*3".to_string())
            .unwrap();
        a.set_user_input(0, 9, 1, "1".to_string()).unwrap();
        a.cell_clear_contents(0, 9, 1).unwrap();
        a.evaluate();

        // Ordinal reads: the drop-in accessors see what a user typed.
        assert_eq!(a.get_formatted_cell_value(0, 1, 2), Ok("30".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 2, 2), Ok("60".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 3, 2), Ok("text!".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 500, 3), Ok("30".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 4, 1), Ok("TRUE".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 5, 1), Ok("7.5".to_string()));
        // Quoted: it reads back as text, not as a formula.
        assert_eq!(
            a.get_formatted_cell_value(0, 6, 1),
            Ok("=not a formula".to_string())
        );
        assert_eq!(a.get_formatted_cell_value(0, 7, 1), Ok("FALSE".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 8, 1), Ok("30".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 9, 1), Ok("".to_string()));
        assert_eq!(a.workbook.worksheets[0].index.rows.len(), 500);
        assert_eq!(a.workbook.worksheets[0].index.cols.len(), 3);

        // Every mutator call is exactly one commit.
        let commits = a.flush();
        assert_eq!(commits.len(), 15);
        assert!(a.flush().is_empty());

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &commits);
        b.evaluate();
        assert_eq!(b.workbook, a.workbook);
        assert_eq!(b.workbook.meta, a.workbook.meta);
        for (row, column) in [(1, 2), (2, 2), (3, 2), (500, 3), (4, 1), (8, 1)] {
            assert_eq!(
                b.get_formatted_cell_value(0, row, column),
                a.get_formatted_cell_value(0, row, column),
                "value at ({row}, {column})"
            );
        }

        // Redelivering the same commits changes nothing.
        deliver(&mut b, 1, &commits);
        b.evaluate();
        assert_eq!(b.workbook, a.workbook);
    }

    /// Sheet ids are minted per session, not per state: two replicas holding the same document and
    /// creating a sheet at the same time must keep both sheets, not race for one id.
    #[test]
    fn concurrent_sheet_ids() {
        let mut a = CollabModel::new(1);
        let mut b = CollabModel::new(2);
        a.new_sheet();
        b.new_sheet();
        assert_ne!(
            a.workbook.worksheets[0].sheet_id,
            b.workbook.worksheets[0].sheet_id
        );
        // A second sheet on the same replica moves the slot on, so ids stay distinct locally too.
        a.new_sheet();
        assert_ne!(
            a.workbook.worksheets[0].sheet_id,
            a.workbook.worksheets[1].sheet_id
        );

        let (from_a, from_b) = (a.flush(), b.flush());
        deliver(&mut b, 1, &from_a);
        deliver(&mut a, 2, &from_b);
        assert_eq!(a.workbook.worksheets.len(), 3);
        assert_eq!(a.workbook, b.workbook);
        assert_eq!(a.workbook.meta, b.workbook.meta);
    }

    /// We use macro, since script is running against both `TestModel` and `CollabModel`, which atm.
    /// don't share common API.
    macro_rules! script {
        ($m:expr) => {{
            let m = &mut $m;
            for row in 1..=5 {
                m.set_user_input(0, row, 1, format!("{row}")).unwrap();
            }
            m.set_user_input(0, 1, 2, "=A1*2".to_string()).unwrap();
            // Spans the rows the insert splits.
            m.set_user_input(0, 5, 2, "=SUM(A1:A5)".to_string())
                .unwrap();
            m.set_user_input(0, 1, 3, "=A3+1".to_string()).unwrap();
            m.set_user_input(0, 1, 4, "100".to_string()).unwrap();
            // Points at the column the delete removes: it has to end up as #REF!.
            m.set_user_input(0, 1, 5, "=D1+1".to_string()).unwrap();
            // Absolute, and above every edit, so it means the same thing afterwards.
            m.new_defined_name("top", None, "Sheet1!$A$1").unwrap();
            m.set_user_input(0, 2, 5, "=top*10".to_string()).unwrap();

            m.insert_rows(0, 3, 2).unwrap();
            m.delete_columns(0, 4, 1).unwrap();
            m.evaluate();
        }};
    }

    #[test]
    fn structural_rewrite() {
        let mut oracle = crate::test::util::new_empty_model();
        script!(oracle);

        let mut a = CollabModel::new(1);
        a.new_sheet();
        script!(a);

        // The oracle populates every cell it touched; the collaborative model must agree on all
        // of them, and hold nothing the oracle does not.
        let cells = oracle.get_all_cells();
        assert!(cells.len() > 8);
        for cell in &cells {
            let (sheet, row, column) = (cell.index, cell.row, cell.column);
            assert_eq!(
                a.get_formatted_cell_value(sheet, row, column),
                oracle.get_formatted_cell_value(sheet, row, column),
                "value at ({sheet}, {row}, {column})"
            );
            assert_eq!(
                a.get_cell_formula(sheet, row, column),
                oracle.get_cell_formula(sheet, row, column),
                "formula at ({sheet}, {row}, {column})"
            );
        }
        assert_eq!(a.get_all_cells().len(), cells.len());
        // The rewrite really did happen: the reference across the insert grew, and the one into
        // the deleted column broke.
        assert_eq!(
            a.get_cell_formula(0, 7, 2),
            Ok(Some("=SUM(A1:A7)".to_string()))
        );
        assert_eq!(
            a.get_cell_formula(0, 1, 4),
            Ok(Some("=#REF!+1".to_string()))
        );
        assert_eq!(a.get_formatted_cell_value(0, 2, 4), Ok("10".to_string()));

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &a.flush());
        b.evaluate();
        assert_eq!(b.workbook, a.workbook);
    }

    /// What a replica shows, which is what an undo has to restore: contents and formatting by
    /// ordinal position, plus the sheet and workbook fields. Deliberately not the CRDT
    /// bookkeeping — guards, tombstones and the append-only intern tables only move forward.
    fn projection(model: &CollabModel<'_>) -> String {
        let mut out = format!("{:?}\n", model.workbook.defined_names);
        for (sheet, ws) in model.workbook.worksheets.iter().enumerate() {
            let sheet = sheet as u32;
            out += &format!(
                "{} {:?} {:?} {} {:?} {:?}\n",
                ws.name, ws.color, ws.state, ws.show_grid_lines, ws.merge_cells, ws.comments
            );
            for column in 1..=Stable::col_count(&ws.index) {
                out += &format!(
                    "c{column} {:?} {:?}\n",
                    model.get_column_width(sheet, column),
                    model.is_column_hidden(sheet, column)
                );
            }
            for row in 1..=Stable::row_count(&ws.index) {
                out += &format!(
                    "r{row} {:?} {:?}\n",
                    model.get_row_height(sheet, row),
                    model.is_row_hidden(sheet, row)
                );
                for column in 1..=Stable::col_count(&ws.index) {
                    out += &format!(
                        "{row},{column} {:?} {:?}\n",
                        model.get_localized_cell_content(sheet, row, column),
                        model.get_cell_style_or_none(sheet, row, column)
                    );
                }
            }
        }
        out
    }

    /// Undo is a new commit carrying the inverses, and it has to land the projection back where it
    /// started.
    #[test]
    fn undo_inverse() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "1".to_string()).unwrap();
        a.set_user_input(0, 2, 1, "=A1*2".to_string()).unwrap();
        a.set_row_height(0, 1, 30.0).unwrap();
        a.set_column_width(0, 1, 80.0).unwrap();
        a.new_defined_name("kept", None, "Sheet1!$A$1").unwrap();
        a.flush();
        let before = projection(&a);
        let (height, width) = (
            a.get_row_height(0, 1).unwrap(),
            a.get_column_width(0, 1).unwrap(),
        );

        let mut style = Style::default();
        style.font.b = true;
        a.set_user_input(0, 1, 1, "99".to_string()).unwrap();
        a.set_cell_style(0, 1, 1, &style).unwrap();
        a.update_cell_with_formula(0, 2, 1, "=A1*3".to_string())
            .unwrap();
        a.set_row_height(0, 1, 55.0).unwrap();
        a.set_column_width(0, 1, 120.0).unwrap();
        a.set_row_hidden(0, 2, true).unwrap();
        a.new_defined_name("temp", None, "Sheet1!$A$2").unwrap();
        a.update_defined_name("kept", None, "kept", None, "Sheet1!$A$2")
            .unwrap();
        let merged = RangeRef::parse_a1("A1:B2").unwrap();
        a.set_merged_range(0, &merged, true).unwrap();
        a.set_comment(0, 1, 1, Some(("note".to_string(), "me".to_string())))
            .unwrap();
        a.set_sheet_color(0, &Color::Rgb("#ff0000".to_string()))
            .unwrap();
        assert_ne!(projection(&a), before);

        // One commit, the inverses of every edit, newest first.
        let undo: Vec<Patch> = a
            .flush()
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(undo);

        assert_eq!(projection(&a), before);
        assert_eq!(a.workbook.defined_names.len(), 1);
        assert_eq!(a.get_row_height(0, 1), Ok(height));
        assert_eq!(a.get_column_width(0, 1), Ok(width));
        assert!(!a.is_row_hidden(0, 2).unwrap());
        a.evaluate();
        assert_eq!(a.get_formatted_cell_value(0, 2, 1), Ok("2".to_string()));
    }
}
