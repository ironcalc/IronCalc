//! Local edits: a mutator call turned into patches.
//!
//! Every mutator validates first — ordinal semantics, so a bad row or a bad name errors before a
//! single patch exists — then resolves ordinals to keys, materializing missing rows and columns as
//! `InsertRows`/`InsertColumns` in the same commit as the write, and hands the lot to
//! [`CollabModel::commit_local`]. There is no second path into the document: for [`Stable`] every
//! mutation is a patch.

use crate::cf_types::{CfRuleInput, ConditionalFormattingView};
use crate::collab::apply::free_sheet_name;
use crate::collab::fractional_index::{CreateKeys, FractionalIndex, FractionalKey, KeyBuf};
use crate::collab::hlc::Hlc;
use crate::collab::log::Timestamp;
use crate::collab::model::{CollabModel, LocalCommit, Stable, StableCellAddress, StableRange};
use crate::collab::patch::{
    CellInput, CfProperty, ColPropKind, ColProperty, ColState, ColumnSnapshot,
    ConditionalFormatState, Patch, RowPropKind, RowProperty, RowSnapshot, RowState, SheetContent,
    SheetId, SheetPropKind, SheetProperty, SheetRestore, WorkbookPropKind, WorkbookProperty,
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
use crate::language::get_default_language;
use crate::locale::{get_default_locale, get_locale};
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
        self.resync_derived(&patches);
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

    fn sheet_ordering_key(&self, at: usize) -> Option<&FractionalKey> {
        let sheet_id = &self.workbook.worksheets.get(at)?.sheet_id;
        let (key, _) = self.workbook.meta.sheet_positions.get(sheet_id)?;
        Some(key)
    }

    /// A tab-order key landing a sheet at index `at`, between the sheets it comes to sit among.
    fn insert_position(&self, at: usize) -> Result<FractionalKey, String> {
        if at >= self.workbook.worksheets.len() {
            return Ok(self.sheet_position());
        }
        let nil = FractionalKey::NULL;
        let hi = self.sheet_ordering_key(at).unwrap_or(&nil);
        // Inserting at the front has no sheet below it, and NULL is that open lower end.
        let lo = at
            .checked_sub(1)
            .and_then(|below| self.sheet_ordering_key(below))
            .unwrap_or(&nil);
        let no_room = || format!("No room for a sheet at index {at}");
        let (mut buf, _) = CreateKeys::plan(lo.position(), hi.position(), 1).ok_or_else(no_room)?;
        buf.extend_from_slice(&self.suffix());
        FractionalKey::try_from_bytes(&buf).map_err(|_| no_room())
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

    /// The name written to a sheet's name register, which its display name is derived from.
    fn authored_name(&self, sheet: SheetId) -> Option<&String> {
        self.workbook
            .meta
            .sheet_names
            .get(&sheet)
            .map(|(name, _)| name)
    }

    /// The sheet property `kind` currently holds, as the property a write would replace.
    fn sheet_prev(&self, i: usize, kind: SheetPropKind) -> Option<SheetProperty> {
        let sheet = &self.workbook.worksheets[i];
        Some(match kind {
            // The authored name, not the displayed one: undo has to restore what was written.
            SheetPropKind::Name => SheetProperty::Name(self.authored_name(sheet.sheet_id)?.clone()),
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
        match input {
            Some(input) => {
                let prev = self.cell_input(i, &at);
                patches.push(Patch::SetCellValue {
                    sheet: id,
                    at: at.clone(),
                    value: Some(input),
                    ts: None,
                    prev: Box::new(prev),
                });
                let stored = self.get_cell_style_or_none(sheet, row, column)?;
                if Some(&style) != stored.as_ref() {
                    patches.push(Patch::SetCellStyle {
                        sheet: id,
                        at,
                        style: Some(Box::new(style)),
                        ts: None,
                        prev: Box::new(stored),
                    });
                }
                Ok(patches)
            }
            None => {
                // clear cell
                patches.extend(self.clear_patches(i, id, at, false));
                Ok(patches)
            }
        }
    }

    fn cell_style_at(&self, i: usize, at: &StableCellAddress) -> Option<Style> {
        let cell = self.workbook.worksheets[i]
            .sheet_data
            .get(&at.0)?
            .get(&at.1)?;
        self.workbook.styles.get_style(cell.get_style()).ok()
    }

    /// The patches clearing the cell at `at`: the value tombstone, and the style either written
    /// back (the value write takes the cell with it) or cleared alongside it when `all`.
    ///
    /// Both registers are always written, even where there is nothing to clear: the stamps are what
    /// make a concurrent older write to the cell lose whichever order it arrives in.
    fn clear_patches(&self, i: usize, id: SheetId, at: StableCellAddress, all: bool) -> Vec<Patch> {
        let prev = self.cell_input(i, &at);
        let stored = self.cell_style_at(i, &at);
        vec![
            Patch::SetCellValue {
                sheet: id,
                at: at.clone(),
                value: None,
                ts: None,
                prev: Box::new(prev),
            },
            Patch::SetCellStyle {
                sheet: id,
                at,
                style: if all { None } else { stored.clone() }.map(Box::new),
                ts: None,
                prev: Box::new(stored),
            },
        ]
    }

    /// The patches clearing every cell the `area` actually holds, styles included when `all`.
    ///
    /// Only cells that exist are written, so a clear outranks a concurrent older write only where it
    /// could see one; a write to a coordinate absent here has no clear patch to lose to, anywhere.
    fn range_clear_patches(&self, area: &Area, all: bool) -> Result<Vec<Patch>, String> {
        if !self.can_clear_range(area)? {
            return Err("Cannot clear the range because it contains array formulas".to_string());
        }
        let (i, id) = self.sheet_of(area.sheet)?;
        let sheet = &self.workbook.worksheets[i];
        let mut patches = Vec::new();
        for row in area.row..area.row + area.height {
            let Some(row_key) = Stable::row_at(&sheet.index, row) else {
                continue;
            };
            for column in area.column..area.column + area.width {
                let Some(col_key) = Stable::col_at(&sheet.index, column) else {
                    continue;
                };
                if sheet.cell(row, column).is_none() {
                    continue;
                }
                patches.extend(self.clear_patches(i, id, (row_key.clone(), col_key), all));
            }
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
        self.evaluate();
        (name, at)
    }

    pub fn add_sheet(&mut self, name: &str) -> Result<(), String> {
        self.insert_sheet(name, self.workbook.worksheets.len() as u32, None)
    }

    pub fn insert_sheet(
        &mut self,
        name: &str,
        sheet_index: u32,
        sheet_id: Option<u32>,
    ) -> Result<(), String> {
        if !is_valid_sheet_name(name) {
            return Err(format!("Invalid name for a sheet: '{name}'"));
        }
        if self
            .workbook
            .get_worksheet_names()
            .iter()
            .map(|s| s.to_uppercase())
            .any(|x| x == name.to_uppercase())
        {
            return Err("A worksheet already exists with that name".to_string());
        }
        if sheet_index as usize > self.workbook.worksheets.len() {
            return Err("Sheet index out of range".to_string());
        }
        let id = match sheet_id {
            // Existence guards outlive their sheet, so a dead id can never be handed out again.
            Some(id)
                if id == 0
                    || self.workbook.worksheets.iter().any(|ws| ws.sheet_id == id)
                    || self.workbook.meta.sheet_existence.contains_key(&id) =>
            {
                return Err(format!("Sheet id {id} is not available"));
            }
            Some(id) => id,
            None => self.new_sheet_id(),
        };
        let position = self.insert_position(sheet_index as usize)?;
        self.commit_local(vec![Patch::AddSheet {
            id,
            name: name.to_string(),
            position,
            content: None,
        }]);
        self.evaluate();
        Ok(())
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
        patches.extend(self.clear_patches(i, id, at, true));
        self.commit_local(patches);
        Ok(())
    }

    /// Removes the content of every cell in the range but leaves the style.
    ///
    /// A coordinate that held nothing stays absent, where upstream materializes an `EmptyCell` for
    /// it: display-identical, but the two models hold different cell counts.
    pub fn range_clear_contents(&mut self, area: &Area) -> Result<(), String> {
        let patches = self.range_clear_patches(area, false)?;
        if !patches.is_empty() {
            self.commit_local(patches);
        }
        Ok(())
    }

    /// Removes both content and style from every cell in the range.
    pub fn range_clear_all(&mut self, area: &Area) -> Result<(), String> {
        let patches = self.range_clear_patches(area, true)?;
        if !patches.is_empty() {
            self.commit_local(patches);
        }
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
            ts: None,
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
            ts: None,
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
            ts: None,
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
            ts: None,
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
            ts: None,
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
            .merged_cells
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
            let displaced =
                to_string_displaced(&node, &context, displace, self.locale, self.language);
            let parsed = self.parse_at(i, moved_row, moved_column, &displaced);
            if let Some(patch) = self.formula_patch(i, id, at, f, to_rc_format(&parsed)) {
                patches.push(patch);
            }
        }
        patches
    }

    /// The patch re-emitting the formula at `at` as `formula`, `None` when the stored text already
    /// says that.
    fn formula_patch(
        &self,
        i: usize,
        id: SheetId,
        at: StableCellAddress,
        f: i32,
        formula: String,
    ) -> Option<Patch> {
        if self.workbook.worksheets[i].shared_formulas.get(f as usize) == Some(&formula) {
            return None;
        }
        let prev = self.cell_input(i, &at);
        Some(Patch::SetCellValue {
            sheet: id,
            at,
            value: Some(CellInput::Formula(formula)),
            ts: None,
            prev: Box::new(prev),
        })
    }

    /// [`Self::displace_formulas`] for a move, which upstream composes as one step per row or
    /// column: each step displaces the references *and* carries the anchor along with the cells it
    /// shifts, so the whole block is a fold over the steps.
    fn displace_moves(&mut self, i: usize, steps: &[DisplaceData]) -> Vec<Patch> {
        let mut patches = Vec::new();
        for (j, id, row, column, at, f) in self.formula_cells() {
            let Some(mut node) = self
                .parsed_formulas
                .get(j)
                .and_then(|sheet| sheet.get(f as usize))
                .map(|(node, _)| node.clone())
            else {
                continue;
            };
            let (mut row, mut column) = (row, column);
            for step in steps {
                let context = CellReferenceRC {
                    sheet: self.workbook.worksheets[j].get_name(),
                    row,
                    column,
                };
                let displaced =
                    to_string_displaced(&node, &context, step, self.locale, self.language);
                if j == i {
                    (row, column) = move_anchor(step, row, column);
                }
                node = self.parse_at(j, row, column, &displaced);
            }
            if let Some(patch) = self.formula_patch(j, id, at, f, to_rc_format(&node)) {
                patches.push(patch);
            }
        }
        patches
    }

    /// Re-emits every defined name whose formula `steps` change, folded in order.
    fn displace_defined_names(&mut self, steps: &[DisplaceData]) -> Vec<Patch> {
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
            // Defined names are stored in the English internal form, so render the displaced
            // formula in the default locale/language to compare against and store.
            let mut displaced = body.clone();
            for step in steps {
                let node = self.parse_internal_formula(&displaced, &context);
                displaced = to_string_displaced(
                    &node,
                    &context,
                    step,
                    get_default_locale(),
                    get_default_language(),
                );
            }
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

    /// Everything the rows `keys` name is about to lose, so that undo can put it back — including
    /// the stamp each register holds, which is what the restore replays at.
    fn row_snapshots(&self, i: usize, keys: &[FractionalKey]) -> Vec<RowSnapshot> {
        let sheet = &self.workbook.worksheets[i];
        let registers = &sheet.index.registers;
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
                let prop_ts = [RowPropKind::Style, RowPropKind::Height, RowPropKind::Hidden]
                    .into_iter()
                    .filter_map(|kind| {
                        let ts = registers.rows.get(&(key.clone(), kind))?;
                        Some((kind, *ts))
                    })
                    .collect();
                RowSnapshot {
                    key: key.clone(),
                    state,
                    cell_values,
                    cell_styles,
                    prop_ts,
                }
            })
            .collect()
    }

    /// [`Self::row_snapshots`] for columns; the cells are keyed by row instead.
    fn column_snapshots(&self, i: usize, keys: &[FractionalKey]) -> Vec<ColumnSnapshot> {
        let sheet = &self.workbook.worksheets[i];
        let registers = &sheet.index.registers;
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
                let span = (key.clone(), key.clone());
                let prop_ts = [ColPropKind::Style, ColPropKind::Width, ColPropKind::Hidden]
                    .into_iter()
                    .filter_map(|kind| {
                        let ts = registers.col_spans.get(&(span.clone(), kind))?;
                        Some((kind, *ts))
                    })
                    .collect();
                ColumnSnapshot {
                    key: key.clone(),
                    state,
                    cell_values,
                    cell_styles,
                    prop_ts,
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
    ) -> (
        Vec<(FractionalKey, CellInput, Timestamp)>,
        Vec<(FractionalKey, Style, Timestamp)>,
    ) {
        let registers = &self.workbook.worksheets[i].index.registers;
        let mut values = Vec::new();
        let mut styles = Vec::new();
        let mut cells: Vec<_> = cells.collect();
        cells.sort_by(|(_, a_axis, _), (_, b_axis, _)| a_axis.cmp(b_axis));
        for (at, key, style) in cells {
            if let Some(input) = self.cell_input(i, &at) {
                let ts = registers.cell_values.get(&at).copied().unwrap_or_default();
                values.push((key.clone(), input, ts));
            }
            if style != 0 {
                if let Ok(style) = self.workbook.styles.get_style(style) {
                    let ts = registers.cell_styles.get(&at).copied().unwrap_or_default();
                    styles.push((key, style, ts));
                }
            }
        }
        (values, styles)
    }

    /// The whole of worksheet `i` as a payload: what an `AddSheet` seeds a copy from, and what the
    /// undo of a delete puts back. Keys are carried as they are — the copy addresses the same rows.
    fn sheet_content(&self, i: usize) -> SheetContent {
        let sheet = &self.workbook.worksheets[i];
        let mut cell_values = Vec::new();
        let mut cell_styles = Vec::new();
        for (row, cells) in &sheet.sheet_data {
            for (col, cell) in cells {
                let at = (row.clone(), col.clone());
                if let Some(input) = self.cell_input(i, &at) {
                    cell_values.push((at.clone(), input));
                }
                // Only a cell holding a style of its own: 0 is the table's default entry.
                if cell.get_style() != 0 {
                    if let Ok(style) = self.workbook.styles.get_style(cell.get_style()) {
                        cell_styles.push((at, style));
                    }
                }
            }
        }
        // `sheet_data` is a hash map, and the payload travels: sort so it does not depend on it.
        cell_values.sort_by(|(a, _), (b, _)| a.cmp(b));
        cell_styles.sort_by(|(a, _), (b, _)| a.cmp(b));
        SheetContent {
            state: sheet.state.clone(),
            color: sheet.color.clone(),
            show_grid_lines: sheet.show_grid_lines,
            frozen_rows: sheet.frozen_rows,
            frozen_columns: sheet.frozen_columns,
            rows: sheet
                .rows
                .iter()
                .map(|row| {
                    let state = RowState {
                        height: row.height,
                        hidden: row.hidden,
                        style: match row.custom_format {
                            true => self.workbook.styles.get_style(row.s).ok().map(Box::new),
                            false => None,
                        },
                        custom_height: row.custom_height,
                        custom_format: row.custom_format,
                    };
                    (row.r.clone(), state)
                })
                .collect(),
            columns: sheet
                .cols
                .iter()
                .map(|col| {
                    let state = ColState {
                        width: col.width,
                        hidden: col.hidden,
                        style: col
                            .style
                            .and_then(|s| self.workbook.styles.get_style(s).ok())
                            .map(Box::new),
                        custom_width: col.custom_width,
                    };
                    ((col.min.clone(), col.max.clone()), state)
                })
                .collect(),
            cell_values,
            cell_styles,
            merge_cells: sheet.merge_cells.clone(),
            comments: sheet.comments.clone(),
            // `cf_order` is kept aligned with the rules themselves, entry by entry.
            conditional_formatting: sheet
                .index
                .registers
                .cf_order
                .iter()
                .cloned()
                .zip(
                    sheet
                        .conditional_formatting
                        .iter()
                        .map(|cf| ConditionalFormatState {
                            rule: cf.cf_rule.clone(),
                            ranges: cf.ranges.clone(),
                        }),
                )
                .collect(),
        }
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
        patches.extend(self.displace_defined_names(&[displace]));
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
        patches.extend(self.displace_defined_names(&[displace]));
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
        patches.extend(self.displace_defined_names(&[displace]));
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
        patches.extend(self.displace_defined_names(&[displace]));
        self.commit_local(patches);
        Ok(())
    }

    /// `plan_move`'s inputs for a block of `count` elements at ordinal `first` moving by `delta`:
    /// the source positions, and the destination in the pre-drain coordinates it reads.
    fn move_span(first: i32, count: i32, delta: i32) -> (std::ops::Range<usize>, usize) {
        let source = (first - 1) as usize..(first - 1 + count) as usize;
        // Past the block, the drained entries are what the planner subtracts back off.
        let dest = if delta > 0 {
            first + count - 1 + delta
        } else {
            first + delta - 1
        };
        (source, dest as usize)
    }

    /// Moves the rows `[row, row + row_count)` by `delta`, rewriting the formulas that referenced
    /// across them. Content, properties and ranges are keyed by identity, so they follow the rows
    /// without a patch of their own.
    ///
    /// Arrays are a later phase: upstream's array-split guard is deferred with them.
    pub fn move_rows_action(
        &mut self,
        sheet: u32,
        row: i32,
        row_count: i32,
        delta: i32,
    ) -> Result<(), String> {
        if row_count <= 0 || delta == 0 {
            return Ok(());
        }
        let target_first = row + delta;
        let target_last = row + row_count - 1 + delta;
        if !(1..=LAST_ROW).contains(&target_first) || !(1..=LAST_ROW).contains(&target_last) {
            return Err("Target row out of boundaries".to_string());
        }
        if !(1..=LAST_ROW).contains(&row) || !(1..=LAST_ROW).contains(&(row + row_count - 1)) {
            return Err("Initial row out of boundaries".to_string());
        }
        let (i, id) = self.sheet_of(sheet)?;
        let index = &self.workbook.worksheets[i].index.rows;
        let (source, dest) = Self::move_span(row, row_count, delta);
        // `plan_move` rebuilds this same tail internally, deterministically, to plan over it.
        let keys = index.plan_virtual(target_last.max(row + row_count - 1) as usize);
        let moves = index.plan_move(source, dest);
        if moves.is_empty() {
            return Err("Cannot move rows there".to_string());
        }
        // A row named past the tail holds nothing yet, so its inverse files it back onto its identity.
        let prev = moves
            .iter()
            .map(|(id, _)| index.held_key(id).unwrap_or_else(|| id.clone()))
            .collect();
        let mut patches = Vec::new();
        if !keys.is_empty() {
            patches.push(Patch::InsertRows { sheet: id, keys });
        }
        patches.push(Patch::MoveRows {
            sheet: id,
            moves,
            prev,
        });
        let steps = Self::move_steps(row, row_count, delta, |r| DisplaceData::RowMove {
            sheet,
            row: r,
            delta,
        });
        patches.extend(self.displace_moves(i, &steps));
        patches.extend(self.displace_defined_names(&steps));
        self.commit_local(patches);
        Ok(())
    }

    /// [`Self::move_rows_action`] for columns.
    pub fn move_columns_action(
        &mut self,
        sheet: u32,
        column: i32,
        column_count: i32,
        delta: i32,
    ) -> Result<(), String> {
        if column_count <= 0 || delta == 0 {
            return Ok(());
        }
        let target_first = column + delta;
        let target_last = column + column_count - 1 + delta;
        if !(1..=LAST_COLUMN).contains(&target_first) || !(1..=LAST_COLUMN).contains(&target_last) {
            return Err("Target column out of boundaries".to_string());
        }
        if !(1..=LAST_COLUMN).contains(&column)
            || !(1..=LAST_COLUMN).contains(&(column + column_count - 1))
        {
            return Err("Initial column out of boundaries".to_string());
        }
        let (i, id) = self.sheet_of(sheet)?;
        let index = &self.workbook.worksheets[i].index.cols;
        let (source, dest) = Self::move_span(column, column_count, delta);
        // `plan_move` rebuilds this same tail internally, deterministically, to plan over it.
        let keys = index.plan_virtual(target_last.max(column + column_count - 1) as usize);
        let moves = index.plan_move(source, dest);
        if moves.is_empty() {
            return Err("Cannot move columns there".to_string());
        }
        // A column named past the tail holds nothing yet, so its inverse files it back onto its identity.
        let prev = moves
            .iter()
            .map(|(id, _)| index.held_key(id).unwrap_or_else(|| id.clone()))
            .collect();
        let mut patches = Vec::new();
        if !keys.is_empty() {
            patches.push(Patch::InsertColumns { sheet: id, keys });
        }
        patches.push(Patch::MoveColumns {
            sheet: id,
            moves,
            prev,
        });
        let steps = Self::move_steps(column, column_count, delta, |c| DisplaceData::ColumnMove {
            sheet,
            column: c,
            delta,
        });
        patches.extend(self.displace_moves(i, &steps));
        patches.extend(self.displace_defined_names(&steps));
        self.commit_local(patches);
        Ok(())
    }

    /// The single-element steps a block move is composed of, in the order upstream applies them:
    /// from the far end of the block, so each step meets the ordinals the last one left.
    fn move_steps(
        first: i32,
        count: i32,
        delta: i32,
        step: impl Fn(i32) -> DisplaceData,
    ) -> Vec<DisplaceData> {
        let range = first..first + count;
        if delta > 0 {
            range.rev().map(step).collect()
        } else {
            range.map(step).collect()
        }
    }

    /// Deletes a sheet by index. Fails if it is the last one.
    pub fn delete_sheet(&mut self, sheet: u32) -> Result<(), String> {
        if self.workbook.worksheets.len() == 1 {
            return Err("Cannot delete only sheet".to_string());
        }
        let (i, id) = self.sheet_of(sheet)?;
        // The authored name and the filed key, so the undo revives the sheet as it was filed.
        let prev = match (
            self.authored_name(id).cloned(),
            self.workbook.meta.sheet_positions.get(&id),
        ) {
            (Some(name), Some((position, _))) => Some(Box::new(SheetRestore {
                name,
                position: position.clone(),
                content: Some(Box::new(self.sheet_content(i))),
            })),
            _ => None,
        };
        self.commit_local(vec![Patch::DeleteSheet { sheet: id, prev }]);
        self.evaluate();
        Ok(())
    }

    /// Deletes a sheet by name. Fails if it does not exist or it is the last one.
    pub fn delete_sheet_by_name(&mut self, name: &str) -> Result<(), String> {
        match self.get_sheet_index_by_name(name) {
            Some(sheet_index) => self.delete_sheet(sheet_index),
            None => Err("Sheet not found".to_string()),
        }
    }

    pub fn duplicate_sheet(&mut self, source: u32) -> Result<(String, u32), String> {
        let (i, source_id) = self.sheet_of(source)?;
        // The repair pass names sheets; asking it here gives the copy the name it would keep
        // anyway. A peer authoring the same name concurrently falls through to that pass.
        let mut taken = self
            .workbook
            .worksheets
            .iter()
            .map(|ws| ws.name.to_uppercase())
            .collect();
        let new_name = free_sheet_name(self.workbook.worksheets[i].name.clone(), &mut taken);
        let id = self.new_sheet_id();
        let position = self.insert_position(i + 1)?;

        // The cached nodes were parsed in the source's context, so an implicit reference already
        // resolves to it — and, carrying no name, follows the copy once it hosts the formula.
        let source_formulas = self.workbook.worksheets[i].shared_formulas.clone();
        let retargeted: Vec<String> = self
            .parsed_formulas
            .get(i)
            .into_iter()
            .flatten()
            .map(|(node, _)| {
                let mut node = node.clone();
                rename_sheet_in_node(&mut node, source, &new_name);
                to_rc_format(&node)
            })
            .collect();
        let mut content = self.sheet_content(i);
        for (_, input) in &mut content.cell_values {
            if let CellInput::Formula(formula) = input {
                if let Some(k) = source_formulas.iter().position(|f| f == formula) {
                    if let Some(text) = retargeted.get(k) {
                        *formula = text.clone();
                    }
                }
            }
        }
        let mut patches = vec![Patch::AddSheet {
            id,
            name: new_name.clone(),
            position,
            content: Some(Box::new(content)),
        }];

        // Names local to the source are always copied; a global one only when it names the source.
        // Locals come first so that, with the de-dup below, they win over a global of the same name.
        let mut names: Vec<(bool, String, String)> = self
            .workbook
            .defined_names
            .iter()
            .filter(|dn| dn.sheet_id == Some(source_id) || dn.sheet_id.is_none())
            .map(|dn| (dn.sheet_id.is_none(), dn.name.clone(), dn.formula.clone()))
            .collect();
        names.sort_by_key(|(is_global, _, _)| *is_global);
        let context = self.defined_name_context();
        let mut copied: Vec<String> = Vec::new();
        for (is_global, name, formula) in names {
            if copied.iter().any(|n| n.eq_ignore_ascii_case(&name)) {
                continue;
            }
            let had_equals = formula.trim_start().starts_with('=');
            let body = formula.strip_prefix('=').unwrap_or(&formula).to_string();
            let mut node = self.parse_internal_formula(&body, &context);
            let before = to_english_string(&node, &context);
            rename_sheet_in_node(&mut node, source, &new_name);
            let after = to_english_string(&node, &context);
            if is_global && before == after {
                continue;
            }
            copied.push(name.clone());
            patches.push(Patch::SetDefinedName {
                scope: Some(id),
                name,
                formula: Some(if had_equals {
                    format!("={after}")
                } else {
                    after
                }),
                prev: None,
            });
        }
        self.commit_local(patches);
        let at = self.get_sheet_index_by_sheet_id(id).unwrap_or_default();
        self.evaluate();
        Ok((new_name, at))
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
                ts: None,
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
        self.evaluate();
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
                    ts: None,
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

/// Where one move step lands the ordinal `at`, `pivot` being the element it moves: the mapping
/// [`to_string_displaced`] applies to references, and the one the cells themselves follow.
fn move_shift(at: i32, pivot: i32, delta: i32) -> i32 {
    if at == pivot {
        at + delta
    } else if delta > 0 && at > pivot && at <= pivot + delta {
        at - 1
    } else if delta < 0 && at < pivot && at >= pivot + delta {
        at + 1
    } else {
        at
    }
}

/// [`move_shift`] over the anchor of a cell, along whichever axis `step` moves.
fn move_anchor(step: &DisplaceData, row: i32, column: i32) -> (i32, i32) {
    match step {
        DisplaceData::RowMove {
            row: pivot, delta, ..
        } => (move_shift(row, *pivot, *delta), column),
        DisplaceData::ColumnMove {
            column: pivot,
            delta,
            ..
        } => (row, move_shift(column, *pivot, *delta)),
        _ => (row, column),
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
    set_language(language_id: &str) -> ();
    set_user_array_formula(sheet: u32, row: i32, column: i32, width: i32, height: i32, value: &str) -> ();
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
            // Both blocks cross formulas already in the sheet, so the rewrites are exercised.
            // Neither crosses A1, which `top` names: upstream leaves defined names alone on a
            // structural edit and emission does not, so the script keeps off that divergence.
            m.move_rows_action(0, 2, 2, 4).unwrap();
            m.move_columns_action(0, 2, 1, 1).unwrap();
            m.move_rows_action(0, 6, 2, 3).unwrap();
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
        // The rewrite really did happen. The reference across the insert grew to A1:A7; the row
        // move then sent row 7 to 5 and pulled the range's top end down with it, and the column
        // move carried the formula itself from B to C.
        assert_eq!(
            a.get_cell_formula(0, 5, 3),
            Ok(Some("=SUM(A1:A5)".to_string()))
        );
        assert_eq!(a.get_formatted_cell_value(0, 5, 3), Ok("13".to_string()));
        // The one into the deleted column broke, and column D was left where it was.
        assert_eq!(
            a.get_cell_formula(0, 1, 4),
            Ok(Some("=#REF!+1".to_string()))
        );
        // The last move reaches past every materialized row, so rows 6 and 7 land on 9 and 10 —
        // ordinals that only exist because the commit carried them.
        assert_eq!(a.get_formatted_cell_value(0, 9, 4), Ok("10".to_string()));
        // The moved rows carried their contents: row 2 held 2, and now row 9 does.
        assert_eq!(a.get_formatted_cell_value(0, 9, 1), Ok("2".to_string()));
        // A reference into the block follows the row it names, not the ordinal it had.
        assert_eq!(a.get_cell_formula(0, 1, 2), Ok(Some("=A3+1".to_string())));

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &a.flush());
        b.evaluate();
        assert_eq!(b.workbook, a.workbook);
    }

    /// A move racing an edit inside the block it moves. B addresses the cell by the ordinal it
    /// still sees, and identity addressing has to land that edit on the row's new ordinal.
    #[test]
    fn concurrent_move_and_edit() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        for row in 1..=4 {
            a.set_user_input(0, row, 1, format!("{row}")).unwrap();
        }
        let setup = a.flush();
        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);

        // Rows 1 and 2 go to 3 and 4, landing on the last ordinal; B, still on the old view, edits
        // the second of them.
        a.move_rows_action(0, 1, 2, 2).unwrap();
        b.set_user_input(0, 2, 1, "edited".to_string()).unwrap();

        let (from_a, from_b) = (a.flush(), b.flush());
        deliver(&mut b, 1, &from_a);
        deliver(&mut a, 2, &from_b);
        a.evaluate();
        b.evaluate();

        // The edit traveled with the row rather than staying on the ordinal it named.
        for m in [&a, &b] {
            assert_eq!(
                m.get_formatted_cell_value(0, 4, 1),
                Ok("edited".to_string())
            );
            assert_eq!(m.get_formatted_cell_value(0, 3, 1), Ok("1".to_string()));
            assert_eq!(m.get_formatted_cell_value(0, 1, 1), Ok("3".to_string()));
            assert_eq!(m.get_formatted_cell_value(0, 2, 1), Ok("4".to_string()));
        }
        assert_eq!(b.workbook, a.workbook);

        // The move above left minted keys on the tail. A second move reaching *past* that tail has
        // to materialize rows that still sort last, and a write far below has to land where it says.
        a.move_rows_action(0, 1, 1, 4).unwrap();
        a.set_user_input(0, 8, 1, "far".to_string()).unwrap();
        deliver(&mut b, 1, &a.flush());
        a.evaluate();
        b.evaluate();

        for m in [&a, &b] {
            // Row 1 went to 5, the rest shifted up one.
            assert_eq!(m.get_formatted_cell_value(0, 1, 1), Ok("4".to_string()));
            assert_eq!(m.get_formatted_cell_value(0, 2, 1), Ok("1".to_string()));
            assert_eq!(
                m.get_formatted_cell_value(0, 3, 1),
                Ok("edited".to_string())
            );
            assert_eq!(m.get_formatted_cell_value(0, 4, 1), Ok("".to_string()));
            assert_eq!(m.get_formatted_cell_value(0, 5, 1), Ok("3".to_string()));
            // Named row 8, reads back at row 8 — not at whatever ordinal a stale mint would give.
            assert_eq!(m.get_formatted_cell_value(0, 8, 1), Ok("far".to_string()));
            assert_eq!(m.workbook.worksheets[0].index.rows.len(), 8);
        }
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
                ws.name, ws.color, ws.state, ws.show_grid_lines, ws.merged_cells, ws.comments
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

    #[test]
    fn delete_beats_older_write() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 2, 1, "anchor".to_string()).unwrap();
        let setup = a.flush();

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);

        // Stamped below the deletes, but reaching A only after them.
        b.set_user_input(0, 2, 2, "old".to_string()).unwrap();
        // Row 3 holds nothing outside the doomed column, so B's map for it empties.
        b.set_user_input(0, 3, 1, "old-col".to_string()).unwrap();
        a.delete_rows(0, 2, 1).unwrap();
        a.delete_columns(0, 1, 1).unwrap();

        let late = b.flush();
        let deletes = a.flush();
        deliver(&mut b, 1, &deletes);
        deliver(&mut a, 2, &late);

        // Shared strings are interned per replica, so only the sheet itself has to match.
        let (a_ws, b_ws) = (&a.workbook.worksheets[0], &b.workbook.worksheets[0]);
        assert!(a_ws.sheet_data.is_empty()); // the deletes outrank the writes on both sides
        assert_eq!(b_ws.sheet_data, a_ws.sheet_data);
        assert_eq!(b_ws.index, a_ws.index);
    }

    /// Undoing a delete replays the restores at the stamps they were captured with, so a
    /// concurrent newer edit to a restored cell outlives the undo.
    #[test]
    fn undo_delete_concurrent() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 2, 1, "keep".to_string()).unwrap();
        a.set_user_input(0, 2, 2, "edit-me".to_string()).unwrap();
        let setup = a.flush();

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);

        a.delete_rows(0, 2, 1).unwrap();
        // Later in program order, so the shared clock stamps it above the delete and the restores.
        b.set_user_input(0, 2, 2, "999".to_string()).unwrap();

        // Undo: one commit carrying the inverses of the delete, newest first.
        let deletes = a.flush();
        let undo: Vec<Patch> = deletes
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(undo);

        let undone = a.flush();
        deliver(&mut b, 1, &deletes);
        // The surviving edit is dead but invisible until the undo puts its row back.
        assert_eq!(b.get_formatted_cell_value(0, 2, 2), Ok("".to_string()));
        deliver(&mut b, 1, &undone);
        deliver(&mut a, 2, &b.flush());
        a.evaluate();
        b.evaluate();

        // The uncontested cell came back; the contested one kept B's newer edit.
        assert_eq!(a.get_formatted_cell_value(0, 2, 1), Ok("keep".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 2, 2), Ok("999".to_string()));
        assert_eq!(
            b.workbook.worksheets[0].index.registers,
            a.workbook.worksheets[0].index.registers
        );
        assert_eq!(b.get_formatted_cell_value(0, 2, 1), Ok("keep".to_string()));
        assert_eq!(b.get_formatted_cell_value(0, 2, 2), Ok("999".to_string()));
        assert_eq!(b.workbook, a.workbook);
    }

    #[test]
    fn undo_move_concurrent() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "1".to_string()).unwrap(); // A1=1
        a.set_user_input(0, 2, 1, "2".to_string()).unwrap(); // A2=2
        a.set_user_input(0, 3, 1, "3".to_string()).unwrap(); // A3=3
        a.set_user_input(0, 1, 2, "=A2*10".to_string()).unwrap(); // B1
        let cell_count = a.get_all_cells().len();
        let setup = a.flush();

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);

        // Peer A: move row A2 past the tail (to A6)
        a.move_rows_action(0, 2, 1, 4).unwrap();
        a.evaluate();
        assert_eq!(a.get_cell_formula(0, 1, 2), Ok(Some("=A6*10".to_string())));
        assert_eq!(a.get_formatted_cell_value(0, 6, 1), Ok("2".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 1, 2), Ok("20".to_string()));

        // Peer B never saw the move: the edit names the row's identity and must follow it both ways
        b.set_user_input(0, 2, 1, "20".to_string()).unwrap(); // A2=20

        // Peer A: undo recent move
        let moved = a.flush();
        let undo: Vec<Patch> = moved
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(undo);
        let undone = a.flush();

        deliver(&mut b, 1, &moved);
        deliver(&mut b, 1, &undone);
        deliver(&mut a, 2, &b.flush());
        a.evaluate();
        b.evaluate();

        for m in [&a, &b] {
            // A2=20 -> Peer B change is still in effect
            assert_eq!(m.get_formatted_cell_value(0, 2, 1), Ok("20".to_string()));
            // B1=A2*10 -> Peer A undo move (A2->A6) is in effect
            assert_eq!(m.get_cell_formula(0, 1, 2), Ok(Some("=A2*10".to_string())));
            assert_eq!(m.get_formatted_cell_value(0, 1, 2), Ok("200".to_string()));
            // A6 (move destination) has no value
            assert_eq!(m.get_formatted_cell_value(0, 6, 1), Ok("".to_string()));
        }
        assert_eq!(a.get_all_cells().len(), cell_count);
        assert_eq!(b.workbook, a.workbook);

        // Redo: the inverse of the inverse.
        let redo: Vec<Patch> = undone
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(redo);
        deliver(&mut b, 1, &a.flush());
        a.evaluate();
        b.evaluate();

        for m in [&a, &b] {
            // Peer A redo moves A2 back to A6, but that doesn't conflict with Peer's B change A2=20
            assert_eq!(m.get_formatted_cell_value(0, 6, 1), Ok("20".to_string()));
            assert_eq!(m.get_cell_formula(0, 1, 2), Ok(Some("=A6*10".to_string())));
            assert_eq!(m.get_formatted_cell_value(0, 1, 2), Ok("200".to_string()));
        }
        assert_eq!(b.workbook, a.workbook);
    }

    fn area(row: i32, column: i32, width: i32, height: i32) -> Area {
        Area {
            sheet: 0,
            row,
            column,
            width,
            height,
        }
    }

    fn bold() -> Style {
        let mut style = Style::default();
        style.font.b = true;
        style
    }

    fn italic() -> Style {
        let mut style = Style::default();
        style.font.i = true;
        style
    }

    /// How many patches of one kind a commit carries.
    fn count(commit: &LocalCommit, kind: fn(&Patch) -> bool) -> usize {
        commit.patches.iter().filter(|p| kind(p)).count()
    }

    /// Clearing a range: contents go, formatting stays or goes with them, and the coordinates the
    /// range names but the sheet does not hold are left alone — nothing is materialized.
    #[test]
    fn range_clear() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "1".to_string()).unwrap(); // A1=1
        a.set_user_input(0, 2, 1, "=A1*2".to_string()).unwrap(); // A2=A1*2
        a.set_cell_style(0, 1, 2, &bold()).unwrap(); // B1=(bold)
        a.set_user_input(0, 2, 2, "keep".to_string()).unwrap(); // B2="keep"
        a.set_cell_style(0, 2, 2, &italic()).unwrap(); // B2=(italic)
        a.set_user_input(0, 1, 3, "100".to_string()).unwrap(); // C1=100
        a.flush();

        // A1:B4 — rows 3 and 4 do not exist yet, and A3..B4 were never written.
        a.range_clear_contents(&area(1, 1, 2, 4)).unwrap();
        a.evaluate();

        let commits = a.flush();
        assert_eq!(commits.len(), 1); // one user action, one commit
        let clear = &commits[0];
        // Only the four cells the sheet holds are written, and nothing is materialized for the rest.
        assert_eq!(count(clear, |p| matches!(p, Patch::SetCellValue { .. })), 4);
        assert_eq!(count(clear, |p| matches!(p, Patch::SetCellStyle { .. })), 4);
        assert_eq!(clear.patches.len(), 8);
        assert_eq!(a.workbook.worksheets[0].index.rows.len(), 2);

        // Contents gone, formatting kept.
        for (row, column) in [(1, 1), (2, 1), (2, 2)] {
            assert_eq!(
                a.get_formatted_cell_value(0, row, column),
                Ok(String::new()),
                "value at ({row}, {column})"
            );
        }
        assert_eq!(a.get_cell_formula(0, 2, 1), Ok(None));
        assert_eq!(a.get_cell_style_or_none(0, 1, 2), Ok(Some(bold())));
        assert_eq!(a.get_cell_style_or_none(0, 2, 2), Ok(Some(italic())));
        // Outside the range nothing moved.
        assert_eq!(a.get_formatted_cell_value(0, 1, 3), Ok("100".to_string()));

        // The clear left the four cells behind, holding just their styles, so they are cleared again.
        a.range_clear_all(&area(1, 1, 2, 4)).unwrap();
        let commits = a.flush();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].patches.len(), 8);
        assert_eq!(a.get_cell_style_or_none(0, 1, 2), Ok(None));
        assert_eq!(a.get_cell_style_or_none(0, 2, 2), Ok(None));
        // Nothing left in the range at all — only C1 survives.
        assert_eq!(a.get_all_cells().len(), 1);
        assert_eq!(a.get_formatted_cell_value(0, 1, 3), Ok("100".to_string()));

        // A range the sheet holds nothing in is not an edit at all: no commit, no rows minted.
        a.range_clear_all(&area(6, 4, 2, 2)).unwrap();
        a.range_clear_contents(&area(1, 1, 2, 4)).unwrap();
        assert!(a.flush().is_empty());
        assert_eq!(a.workbook.worksheets[0].index.rows.len(), 2);
        assert_eq!(a.workbook.worksheets[0].index.cols.len(), 3);

        // Upstream's validation, error string included; a rejected call emits nothing.
        let bad = "Row or column is outside valid range.".to_string();
        assert_eq!(a.range_clear_contents(&area(0, 1, 1, 1)), Err(bad.clone()));
        assert_eq!(a.range_clear_all(&area(1, 0, 1, 1)), Err(bad));
        let off_sheet = Area {
            sheet: 9,
            row: 1,
            column: 1,
            width: 1,
            height: 1,
        };
        assert_eq!(
            a.range_clear_all(&off_sheet),
            Err("Invalid sheet index".to_string())
        );
        assert!(a.flush().is_empty());
    }

    #[test]
    fn range_clear_converges() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "1".to_string()).unwrap();
        a.set_user_input(0, 1, 2, "2".to_string()).unwrap();
        let setup = a.flush();

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);

        // Both stamped below the clear and reaching A only after it. A1 is a cell the clear can
        // see; B3 is not — row 3 does not exist when the clear is authored.
        b.set_user_input(0, 1, 1, "11".to_string()).unwrap();
        b.set_user_input(0, 3, 2, "33".to_string()).unwrap();
        a.range_clear_contents(&area(1, 1, 2, 3)).unwrap();
        // Later in program order, so the shared clock stamps it above the clear.
        b.set_user_input(0, 1, 2, "42".to_string()).unwrap();

        let clear = a.flush();
        let from_b = b.flush();
        deliver(&mut b, 1, &clear);
        deliver(&mut a, 2, &from_b);
        a.evaluate();
        b.evaluate();

        for m in [&a, &b] {
            // The older write to a cell the clear saw lost; the newer one to the same cell won.
            assert_eq!(m.get_formatted_cell_value(0, 1, 1), Ok(String::new()));
            assert_eq!(m.get_formatted_cell_value(0, 1, 2), Ok("42".to_string()));
            // The clear never named B3, so nothing there was ever contested.
            assert_eq!(m.get_formatted_cell_value(0, 3, 2), Ok("33".to_string()));
        }
        assert_eq!(b.workbook, a.workbook);
    }

    #[test]
    fn range_clear_undo_concurrent() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "1".to_string()).unwrap();
        a.set_cell_style(0, 1, 1, &bold()).unwrap();
        a.set_user_input(0, 2, 1, "edit-me".to_string()).unwrap();
        a.set_user_input(0, 3, 1, "3".to_string()).unwrap();
        let setup = a.flush();

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);

        // A clear is nothing but register writes, so its inverse is nothing but register writes.
        a.range_clear_all(&area(1, 1, 1, 3)).unwrap();
        b.set_user_input(0, 2, 1, "999".to_string()).unwrap();

        let cleared = a.flush();
        let undo: Vec<Patch> = cleared
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(undo);

        let undone = a.flush();
        deliver(&mut b, 1, &cleared);
        deliver(&mut b, 1, &undone);
        deliver(&mut a, 2, &b.flush());
        a.evaluate();
        b.evaluate();

        for m in [&a, &b] {
            // The uncontested cells came back, style included.
            assert_eq!(m.get_formatted_cell_value(0, 1, 1), Ok("1".to_string()));
            assert_eq!(m.get_cell_style_or_none(0, 1, 1), Ok(Some(bold())));
            assert_eq!(m.get_formatted_cell_value(0, 3, 1), Ok("3".to_string()));
            // The contested one too: the undo's stamp is above B's edit, so the restore wins.
            //TODO: should contesting undo win over concurrent edit?
            assert_eq!(
                m.get_formatted_cell_value(0, 2, 1),
                Ok("edit-me".to_string())
            );
        }
        assert_eq!(b.workbook, a.workbook);
    }

    /// The tab order a replica shows.
    fn names(model: &CollabModel<'_>) -> Vec<String> {
        model
            .get_worksheets_properties()
            .into_iter()
            .map(|p| p.name)
            .collect()
    }

    /// Sheet lifecycle: named creation, insertion anywhere in the tab order, deletion by name and
    /// the undo of an add — all of it converging on a second replica.
    #[test]
    fn sheet_lifecycle() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.add_sheet("Beta").unwrap();
        a.add_sheet("Gamma").unwrap();
        assert_eq!(names(&a), ["Sheet1", "Beta", "Gamma"]);
        // A named add lands at the end, and the ordinal writers can address it there.
        a.set_user_input(2, 1, 1, "gamma".to_string()).unwrap();

        a.insert_sheet("Mid", 1, None).unwrap();
        assert_eq!(names(&a), ["Sheet1", "Mid", "Beta", "Gamma"]);
        a.insert_sheet("First", 0, None).unwrap();
        a.insert_sheet("Last", 5, None).unwrap();
        assert_eq!(
            names(&a),
            ["First", "Sheet1", "Mid", "Beta", "Gamma", "Last"]
        );
        // Two inserts into the same gap still order by the index each named.
        a.insert_sheet("Mid2", 3, None).unwrap();
        assert_eq!(
            names(&a),
            ["First", "Sheet1", "Mid", "Mid2", "Beta", "Gamma", "Last"]
        );

        // Validation, with the ordinal writers' messages verbatim.
        assert_eq!(
            a.add_sheet("bad/name"),
            Err("Invalid name for a sheet: 'bad/name'".to_string())
        );
        assert_eq!(
            a.add_sheet("bETA"),
            Err("A worksheet already exists with that name".to_string())
        );
        assert_eq!(
            a.insert_sheet("Nope", 99, None),
            Err("Sheet index out of range".to_string())
        );
        // An id already spoken for is refused: reusing one would address that sheet, not create one.
        let taken = a.workbook.worksheets[0].sheet_id;
        assert_eq!(
            a.insert_sheet("Nope", 0, Some(taken)),
            Err(format!("Sheet id {taken} is not available"))
        );
        assert_eq!(
            a.insert_sheet("Nope", 0, Some(0)),
            Err("Sheet id 0 is not available".to_string())
        );
        a.insert_sheet("Chosen", 0, Some(4242)).unwrap();
        assert_eq!(a.workbook.worksheets[0].sheet_id, 4242);

        // Deletion by name is case insensitive, and an unknown name is an error, not a no-op.
        a.delete_sheet_by_name("mid2").unwrap();
        assert_eq!(
            a.delete_sheet_by_name("Mid2"),
            Err("Sheet not found".to_string())
        );
        assert_eq!(
            names(&a),
            ["Chosen", "First", "Sheet1", "Mid", "Beta", "Gamma", "Last"]
        );

        // A dead sheet's key is still filed, so an insert into the gap it left must not re-mint it.
        a.insert_sheet("Revived", 3, None).unwrap();
        assert_eq!(names(&a)[3], "Revived");
        assert_eq!(a.workbook.worksheets.len(), 8);

        // Undo of an add removes the sheet again.
        let setup = a.flush();
        a.add_sheet("Doomed").unwrap();
        assert_eq!(names(&a).last().map(String::as_str), Some("Doomed"));
        let added = a.flush();
        let undo: Vec<Patch> = added
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(undo);
        assert!(!names(&a).contains(&"Doomed".to_string()));
        assert_eq!(a.workbook.worksheets.len(), 8);

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);
        deliver(&mut b, 1, &added);
        deliver(&mut b, 1, &a.flush());
        a.evaluate();
        b.evaluate();
        assert_eq!(names(&b), names(&a));
        let gamma = names(&b).iter().position(|n| n == "Gamma").unwrap() as u32;
        assert_eq!(
            b.get_formatted_cell_value(gamma, 1, 1),
            Ok("gamma".to_string())
        );
        assert_eq!(b.workbook, a.workbook);

        // Concurrent inserts at the same index: both survive, in the same order on both replicas.
        a.insert_sheet("FromA", 1, None).unwrap();
        b.insert_sheet("FromB", 1, None).unwrap();
        let (from_a, from_b) = (a.flush(), b.flush());
        deliver(&mut b, 1, &from_a);
        deliver(&mut a, 2, &from_b);
        assert!(names(&a).contains(&"FromA".to_string()));
        assert!(names(&a).contains(&"FromB".to_string()));
        assert_eq!(names(&b), names(&a));
        assert_eq!(b.workbook, a.workbook);
    }

    /// The sheet each display name ended up on. Equality of this across replicas is the real
    /// convergence claim: agreeing on the *set* of names would not stop two replicas swapping them.
    fn naming<'a>(model: &'a CollabModel<'_>) -> Vec<(&'a str, SheetId)> {
        model
            .workbook
            .worksheets
            .iter()
            .map(|ws| (ws.name.as_str(), ws.sheet_id))
            .collect()
    }

    /// The id of the sheet a single-`AddSheet` commit created.
    fn added_id(commits: &[LocalCommit]) -> SheetId {
        match commits.first().and_then(|c| c.patches.first()) {
            Some(Patch::AddSheet { id, .. }) => *id,
            other => panic!("expected an AddSheet, got {other:?}"),
        }
    }

    /// The display name `model` gave the sheet with this id.
    fn sheet_name<'a>(model: &'a CollabModel<'_>, id: SheetId) -> &'a str {
        model
            .workbook
            .worksheets
            .iter()
            .find(|ws| ws.sheet_id == id)
            .map(|ws| ws.name.as_str())
            .unwrap()
    }

    fn sorted_names(model: &CollabModel<'_>) -> Vec<String> {
        let mut out = names(model);
        out.sort();
        out
    }

    /// Concurrent adds and renames can author the same name on different replicas. Excel needs them
    /// case-insensitively unique, so the losers are renumbered — the same way everywhere.
    #[test]
    fn sheet_name_repair() {
        // One Sheet1, shared by three replicas.
        let mut p1 = CollabModel::new(1);
        p1.new_sheet();
        let setup = p1.flush();
        let mut p2 = CollabModel::new(2);
        let mut p3 = CollabModel::new(3);
        deliver(&mut p2, 1, &setup);
        deliver(&mut p3, 1, &setup);

        // All three add "Data" without having seen each other. Local validation cannot see it.
        p1.add_sheet("Data").unwrap();
        p2.add_sheet("Data").unwrap();
        p3.add_sheet("Data").unwrap();
        let (c1, c2, c3) = (p1.flush(), p2.flush(), p3.flush());
        let (id1, id2, id3) = (added_id(&c1), added_id(&c2), added_id(&c3));

        deliver(&mut p1, 2, &c2);
        deliver(&mut p1, 3, &c3);
        deliver(&mut p2, 1, &c1);
        deliver(&mut p2, 3, &c3);
        deliver(&mut p3, 1, &c1);
        deliver(&mut p3, 2, &c2);

        let expected = ["Data", "Data (1)", "Data (2)", "Sheet1"];
        for peer in [&p1, &p2, &p3] {
            assert_eq!(peer.workbook.worksheets.len(), 4);
            assert_eq!(sorted_names(peer), expected);
            // First write wins: p1 authored first, so p1's sheet keeps the name unsuffixed.
            assert_eq!(sheet_name(peer, id1), "Data");
        }
        // Same names *on the same sheets*, not merely the same set of names.
        assert_eq!(naming(&p1), naming(&p2));
        assert_eq!(naming(&p2), naming(&p3));

        // Commutativity: delivery order cannot change what a replica ends up showing.
        let authored = [(1u32, &c1), (2, &c2), (3, &c3)];
        let orders = [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ];
        let replays: Vec<CollabModel<'static>> = orders
            .iter()
            .map(|order| {
                let mut consumer = CollabModel::new(9);
                deliver(&mut consumer, 1, &setup);
                for &i in order {
                    let (session, commits) = authored[i];
                    deliver(&mut consumer, session, commits);
                }
                consumer
            })
            .collect();
        for (order, replay) in orders.iter().zip(&replays) {
            assert_eq!(naming(replay), naming(&replays[0]), "order {order:?}");
            assert_eq!(replay.workbook, replays[0].workbook, "order {order:?}");
        }
        assert_eq!(naming(&replays[0]), naming(&p1));

        // A replica that never saw the collision authors "Data (1)" itself. The trailing " (1)" is
        // a number to continue, not a base to suffix again, so it lands on "Data (3)".
        let mut p4 = CollabModel::new(4);
        deliver(&mut p4, 1, &setup);
        p4.add_sheet("Data (1)").unwrap();
        let c4 = p4.flush();
        let id4 = added_id(&c4);
        deliver(&mut p4, 1, &c1);
        deliver(&mut p4, 2, &c2);
        deliver(&mut p4, 3, &c3);
        deliver(&mut p1, 4, &c4);
        for peer in [&p1, &p4] {
            assert_eq!(
                sorted_names(peer),
                ["Data", "Data (1)", "Data (2)", "Data (3)", "Sheet1"]
            );
            assert_eq!(sheet_name(peer, id4), "Data (3)");
            assert_eq!(sheet_name(peer, id1), "Data");
        }
        assert_eq!(naming(&p1), naming(&p4));
        // The other two sheets kept whatever the first pass gave them.
        assert_eq!(sheet_name(&p1, id2), sheet_name(&p2, id2));
        assert_eq!(sheet_name(&p1, id3), sheet_name(&p2, id3));

        // Two replicas rename *different* sheets to the same name.
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.add_sheet("Other").unwrap();
        let base = a.flush();
        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &base);
        let (sheet1, other) = (
            a.workbook.worksheets[0].sheet_id,
            a.workbook.worksheets[1].sheet_id,
        );
        a.rename_sheet_by_index(0, "X").unwrap();
        b.rename_sheet_by_index(1, "X").unwrap();
        let (ra, rb) = (a.flush(), b.flush());
        deliver(&mut a, 2, &rb);
        deliver(&mut b, 1, &ra);
        for peer in [&a, &b] {
            assert_eq!(sorted_names(peer), ["X", "X (1)"]);
            assert_eq!(sheet_name(peer, sheet1), "X");
            assert_eq!(sheet_name(peer, other), "X (1)");
        }
        assert_eq!(naming(&a), naming(&b));

        // The auto-generated name races too: both replicas mint "Sheet2" off the same tab list.
        let mut c = CollabModel::new(1);
        c.new_sheet();
        let one = c.flush();
        let mut d = CollabModel::new(2);
        deliver(&mut d, 1, &one);
        assert_eq!(c.new_sheet().0, "Sheet2");
        assert_eq!(d.new_sheet().0, "Sheet2");
        let (two_c, two_d) = (c.flush(), d.flush());
        deliver(&mut c, 2, &two_d);
        deliver(&mut d, 1, &two_c);
        for peer in [&c, &d] {
            assert_eq!(sorted_names(peer), ["Sheet1", "Sheet2", "Sheet2 (1)"]);
            assert_eq!(sheet_name(peer, added_id(&two_c)), "Sheet2");
        }
        assert_eq!(naming(&c), naming(&d));
    }

    /// Every defined name, as `(name, scope, formula)`.
    fn defined(model: &CollabModel<'_>) -> Vec<(String, Option<SheetId>, String)> {
        model
            .workbook
            .defined_names
            .iter()
            .map(|dn| (dn.name.clone(), dn.sheet_id, dn.formula.clone()))
            .collect()
    }

    /// A copy carries the whole sheet: contents, formatting, sheet properties and the defined names
    /// that named it — with every reference to the original retargeted at the copy.
    #[test]
    fn duplicate_sheet_scenario() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.add_sheet("Other").unwrap();
        a.set_user_input(1, 1, 1, "5".to_string()).unwrap();

        a.set_user_input(0, 1, 1, "10".to_string()).unwrap();
        a.set_user_input(0, 2, 1, "=Sheet1!A1*2".to_string())
            .unwrap();
        a.set_user_input(0, 3, 1, "=A1+1".to_string()).unwrap();
        a.set_user_input(0, 4, 1, "=Other!A1".to_string()).unwrap();
        let mut style = Style::default();
        style.font.b = true;
        a.set_cell_style(0, 1, 1, &style).unwrap();
        a.set_row_height(0, 2, 42.0).unwrap();
        let merged = RangeRef::parse_a1("B1:C2").unwrap();
        a.set_merged_range(0, &merged, true).unwrap();
        a.new_defined_name("loc", Some(0), "Sheet1!$A$1").unwrap();
        a.new_defined_name("glob", None, "Sheet1!$A$1").unwrap();
        a.new_defined_name("elsewhere", None, "Other!$A$1").unwrap();
        a.evaluate();

        let (name, at) = a.duplicate_sheet(0).unwrap();
        assert_eq!((name.as_str(), at), ("Sheet1 (1)", 1));
        // The copy sits right after the original, which keeps its own name.
        assert_eq!(names(&a), ["Sheet1", "Sheet1 (1)", "Other"]);
        a.evaluate();

        // An explicit self-reference now names the copy; an implicit one follows its host, and a
        // reference to a third sheet is left where it pointed.
        assert_eq!(
            a.get_cell_formula(1, 2, 1),
            Ok(Some("='Sheet1 (1)'!A1*2".to_string()))
        );
        assert_eq!(a.get_cell_formula(1, 3, 1), Ok(Some("=A1+1".to_string())));
        assert_eq!(
            a.get_cell_formula(1, 4, 1),
            Ok(Some("=Other!A1".to_string()))
        );
        // The source is untouched.
        assert_eq!(
            a.get_cell_formula(0, 2, 1),
            Ok(Some("=Sheet1!A1*2".to_string()))
        );
        for sheet in [0, 1] {
            assert_eq!(
                a.get_formatted_cell_value(sheet, 1, 1),
                Ok("10".to_string())
            );
            assert_eq!(
                a.get_formatted_cell_value(sheet, 2, 1),
                Ok("20".to_string())
            );
            assert_eq!(
                a.get_formatted_cell_value(sheet, 3, 1),
                Ok("11".to_string())
            );
            assert_eq!(a.get_formatted_cell_value(sheet, 4, 1), Ok("5".to_string()));
        }
        // Formatting, row properties and merges came along.
        assert_eq!(a.get_cell_style_or_none(1, 1, 1), Ok(Some(style)));
        assert_eq!(a.get_row_height(1, 2), Ok(42.0));
        assert_eq!(
            a.workbook.worksheets[1].merge_cells,
            a.workbook.worksheets[0].merge_cells
        );

        // Upstream's naming rules: the local name is copied as a local of the copy, the global one
        // that named the source gets a local copy beside it, the unrelated global is left alone.
        let copy_id = a.workbook.worksheets[1].sheet_id;
        let source_id = a.workbook.worksheets[0].sheet_id;
        assert_eq!(
            defined(&a),
            [
                (
                    "loc".to_string(),
                    Some(source_id),
                    "Sheet1!$A$1".to_string()
                ),
                ("glob".to_string(), None, "Sheet1!$A$1".to_string()),
                ("elsewhere".to_string(), None, "Other!$A$1".to_string()),
                (
                    "loc".to_string(),
                    Some(copy_id),
                    "'Sheet1 (1)'!$A$1".to_string()
                ),
                (
                    "glob".to_string(),
                    Some(copy_id),
                    "'Sheet1 (1)'!$A$1".to_string()
                ),
            ]
        );

        // The two sheets are independent documents from here on.
        a.set_user_input(0, 1, 1, "100".to_string()).unwrap();
        a.set_user_input(1, 1, 1, "7".to_string()).unwrap();
        a.evaluate();
        assert_eq!(a.get_formatted_cell_value(0, 2, 1), Ok("200".to_string()));
        assert_eq!(a.get_formatted_cell_value(1, 2, 1), Ok("14".to_string()));

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &a.flush());
        b.evaluate();
        assert_eq!(names(&b), names(&a));
        assert_eq!(defined(&b), defined(&a));
        for (sheet, row) in [(0, 1), (0, 2), (1, 1), (1, 2), (1, 3), (1, 4)] {
            assert_eq!(
                b.get_formatted_cell_value(sheet, row, 1),
                a.get_formatted_cell_value(sheet, row, 1),
                "value at ({sheet}, {row})"
            );
        }
        assert_eq!(b.workbook, a.workbook);

        // Out of range is an error, with the ordinal writer's message.
        assert_eq!(
            a.duplicate_sheet(99),
            Err("Invalid sheet index".to_string())
        );
    }

    #[test]
    fn delete_sheet_undo() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.add_sheet("Tail").unwrap();
        a.insert_sheet("Data", 1, None).unwrap();
        a.set_user_input(1, 1, 1, "10".to_string()).unwrap();
        a.set_user_input(1, 2, 1, "=A1*2".to_string()).unwrap();
        let mut style = Style::default();
        style.font.b = true;
        a.set_cell_style(1, 1, 1, &style).unwrap();
        a.set_row_height(1, 1, 42.0).unwrap();
        a.set_sheet_color(1, &Color::Rgb("#00ff00".to_string()))
            .unwrap();
        // Scoped to another sheet: the delete must leave it alone.
        a.new_defined_name("kept", Some(0), "Sheet1!$A$1").unwrap();
        a.evaluate();
        let data_id = a.workbook.worksheets[1].sheet_id;
        let data_key = a.workbook.meta.sheet_positions[&data_id].0.clone();
        let before = projection(&a);
        let setup = a.flush();

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);

        // B edits the sheet A is deleting. Reaching A after the delete, the edit is dropped there —
        // and the restore holds what A captured, so it is lost on B too rather than resurrected.
        b.set_user_input(1, 1, 1, "999".to_string()).unwrap();
        a.delete_sheet(1).unwrap();
        let deleted = a.flush();
        deliver(&mut a, 2, &b.flush());
        deliver(&mut b, 1, &deleted);
        assert_eq!(names(&a), ["Sheet1", "Tail"]);
        assert_eq!(names(&b), names(&a));
        assert_eq!(b.workbook, a.workbook);

        // Undo: everything the sheet held comes back.
        let undo: Vec<Patch> = deleted
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(undo);
        let undone = a.flush();
        deliver(&mut b, 1, &undone);
        a.evaluate();
        b.evaluate();
        assert_eq!(projection(&a), before);
        assert_eq!(a.workbook.worksheets[1].sheet_id, data_id);
        assert_eq!(a.get_formatted_cell_value(1, 1, 1), Ok("10".to_string()));
        assert_eq!(a.get_formatted_cell_value(1, 2, 1), Ok("20".to_string()));
        assert_eq!(names(&b), names(&a));
        assert_eq!(b.workbook, a.workbook);

        // Redo takes it away again.
        let redo: Vec<Patch> = undone
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(redo);
        let redone = a.flush();
        deliver(&mut b, 1, &redone);
        assert_eq!(names(&a), ["Sheet1", "Tail"]);
        assert_eq!(names(&b), names(&a));
        assert_eq!(b.workbook, a.workbook);

        // While it is dead B takes its name, and A fills the gap it left in the tab order.
        b.add_sheet("Data").unwrap();
        let grabbed = b.flush();
        a.insert_sheet("Filler", 1, None).unwrap();
        let filler_id = a.workbook.worksheets[1].sheet_id;
        // The gap re-mints the very key the dead sheet still holds: only the id separates them.
        assert_eq!(a.workbook.meta.sheet_positions[&filler_id].0, data_key);
        let filled = a.flush();
        deliver(&mut a, 2, &grabbed);
        deliver(&mut b, 1, &filled);

        let undo: Vec<Patch> = redone
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(undo);
        let revived = a.flush();
        deliver(&mut b, 1, &revived);

        // A third replica meets the two sheets sharing a key in the opposite order, which only the
        // id tiebreak keeps from showing them in the opposite order too.
        let mut c = CollabModel::new(3);
        for (session, commits) in [
            (1, &setup),
            (1, &deleted),
            (1, &undone),
            (1, &redone),
            (2, &grabbed),
            (1, &revived),
            (1, &filled),
        ] {
            deliver(&mut c, session, commits);
        }
        a.evaluate();
        b.evaluate();
        c.evaluate();
        for peer in [&a, &b, &c] {
            // The revival authors the name anew, so B's earlier "Data" keeps it and the sheet
            // coming back gives way.
            assert_eq!(
                sorted_names(peer),
                ["Data", "Data (1)", "Filler", "Sheet1", "Tail"]
            );
            assert_eq!(sheet_name(peer, data_id), "Data (1)");
            let data = peer.get_sheet_index_by_sheet_id(data_id).unwrap();
            assert_eq!(
                peer.get_formatted_cell_value(data, 2, 1),
                Ok("20".to_string())
            );
        }
        assert_eq!(naming(&a), naming(&b));
        assert_eq!(naming(&a), naming(&c));
        assert_eq!(b.workbook, a.workbook);
    }

    #[test]
    fn late_edit_to_revived_sheet() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.add_sheet("Data").unwrap();
        a.set_user_input(1, 1, 1, "10".to_string()).unwrap(); // A: Data!A1=10
        let setup = a.flush();
        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);

        b.set_user_input(1, 5, 1, "999".to_string()).unwrap(); // B: Data!A5=999
        let early = b.flush();
        a.delete_sheet(1).unwrap(); // A: delete 'Data'
        let deleted = a.flush();
        b.set_user_input(1, 6, 1, "888".to_string()).unwrap(); // B (concurrently): Data!A6=888
        let late = b.flush();
        let undo: Vec<Patch> = deleted
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(undo);
        let undone = a.flush();

        // A meets both edits after the revive; B meets the delete after making them.
        deliver(&mut a, 2, &early);
        deliver(&mut a, 2, &late);
        deliver(&mut b, 1, &deleted);
        deliver(&mut b, 1, &undone);
        a.evaluate();
        b.evaluate();
        // Dropped on both, not merely agreed on: the revive stamp outranks either edit's.
        assert_eq!(a.get_formatted_cell_value(1, 5, 1), Ok("".to_string()));
        assert_eq!(b.get_formatted_cell_value(1, 5, 1), Ok("".to_string()));
        assert_eq!(a.get_formatted_cell_value(1, 1, 1), Ok("10".to_string()));
        // A resurrected row would re-enter the index right after the snapshot's, so the ordinal the
        // edit was authored at says nothing: what proves it gone is the sheet still being one row.
        let rows = |m: &CollabModel<'_>| m.workbook.worksheets[1].sheet_data.len();
        assert_eq!((rows(&a), rows(&b)), (1, 1));
        assert_eq!(b.workbook, a.workbook);

        // Only writes from before the revive die: new work on the revived sheet applies as usual.
        b.set_user_input(1, 5, 1, "42".to_string()).unwrap();
        deliver(&mut a, 2, &b.flush());
        a.evaluate();
        b.evaluate();
        assert_eq!(a.get_formatted_cell_value(1, 5, 1), Ok("42".to_string()));
        assert_eq!(b.get_formatted_cell_value(1, 5, 1), Ok("42".to_string()));
        assert_eq!(b.workbook, a.workbook);
    }

    #[test]
    fn new_sheet_reevaluates_stale_refs() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "7".to_string()).unwrap();
        a.set_user_input(0, 2, 1, "=Sheet2!C3".to_string()).unwrap();
        a.evaluate();
        // No sheet named Sheet2 yet: the reference is broken, as upstream agrees.
        assert_eq!(a.get_formatted_cell_value(0, 2, 1), Ok("#REF!".to_string()));

        // Creating "Sheet2" resolves the reference: `new_sheet` re-evaluated, as upstream's does.
        a.new_sheet();
        assert_eq!(a.get_formatted_cell_value(0, 2, 1), Ok("0".to_string()));
    }
}
