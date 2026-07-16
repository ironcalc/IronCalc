//! The collaboration session: translates between the `user_model` diff stream
//! and the replicated yrs document.
//!
//! Direction 1 (outbound): local edits run through [`UserModel`] exactly as
//! before; the session drains the send queue (the same seam
//! `flush_send_queue` uses), maps absolute indices to stable ids *at the time
//! each diff was produced* (the cached orders evolve diff by diff), and writes
//! the result into the document. Cell and property *values* are read back from
//! the post-batch model state (each touched location is written once with its
//! final value), which sidesteps reconstructing inputs from `Diff` payloads
//! and makes undo diffs translate for free.
//!
//! Direction 2 (inbound): a remote update is applied to the document, a fresh
//! [`Projection`] is diffed against the last applied one (the *shadow*), and
//! the differences are pushed into the model through `Model` methods —
//! bypassing history and the send queue, like `apply_external_diffs` does.
//!
//! The document is the convergence point; the model is a deterministic
//! projection of it. Formula results never enter the document: after every
//! reconcile the model re-evaluates.
//!
//! v1 scope notes:
//! * Styles, conditional formatting, named styles, sheet color/state/gridlines
//!   and themes are not replicated yet: those diffs are ignored.
//! * Row/column moves, defined names and locale/timezone changes return an
//!   error (loud, so tests catch scope creep).
//! * Formulas are replicated as text; structural edits concurrent with
//!   formula edits displace on one replica only (fixed in the id-ref phase).

use std::collections::{BTreeMap, BTreeSet};

use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, Map, ReadTxn, StateVector, Transact, TransactionMut, Update};

use crate::constants::{
    COLUMN_WIDTH_FACTOR, DEFAULT_COLUMN_WIDTH, DEFAULT_ROW_HEIGHT, ROW_HEIGHT_FACTOR,
};
use crate::types::Cell;
use crate::user_model::history::{Diff, DiffType, QueueDiffs};
use crate::UserModel;

use super::formula::{encode_formula, is_id_form, render_formula, RefResolver};
use super::ids::{EntityId, MAX_COLUMN, MAX_ROW};
use super::order::{original_position, unique_position, AxisOrder, ResolvedIndex};
use super::projection::{
    axis_key, cell_key, keep_key, keep_prefix, sheet_meta_key, Axis, Projection, SchemaMaps,
    SheetProj,
};

/// Resolves formula references against a consistent view of the replicated
/// state: sheet display names plus the row/column orders of every visible
/// sheet (cross-sheet references resolve on the *referenced* sheet's orders).
struct DocResolver {
    /// Visible sheets in display order with their model (display) names.
    sheets: Vec<(EntityId, String)>,
    rows: BTreeMap<EntityId, AxisOrder>,
    cols: BTreeMap<EntityId, AxisOrder>,
}

impl DocResolver {
    /// View for inbound rendering: everything comes from the projection; the
    /// names are the deduplicated display names the model uses.
    fn from_projection(proj: &Projection) -> DocResolver {
        let visible = proj.visible_sheets();
        let names = dedupe_names(&visible);
        let mut resolver = DocResolver {
            sheets: Vec::with_capacity(visible.len()),
            rows: BTreeMap::new(),
            cols: BTreeMap::new(),
        };
        for ((id, sp), name) in visible.iter().zip(names) {
            resolver.sheets.push((*id, name));
            resolver.rows.insert(*id, sp.axis_order(Axis::Rows));
            resolver.cols.insert(*id, sp.axis_order(Axis::Columns));
        }
        resolver
    }

    /// View for outbound encoding: orders from the (evolved) translation
    /// context, names from the post-batch model (index-aligned by invariant).
    fn from_ctx(ctx: &OrderCtx, um: &UserModel) -> Result<DocResolver, String> {
        let mut sheets = Vec::with_capacity(ctx.sheets.len());
        for (index, (id, _)) in ctx.sheets.iter().enumerate() {
            let name = um.model.workbook.worksheet(index as u32)?.get_name();
            sheets.push((*id, name));
        }
        Ok(DocResolver {
            sheets,
            rows: ctx.rows.clone(),
            cols: ctx.cols.clone(),
        })
    }

    /// View for bootstrap: `Original` sheet ids, pristine orders.
    fn pristine_from_model(um: &UserModel) -> DocResolver {
        let mut resolver = DocResolver {
            sheets: Vec::new(),
            rows: BTreeMap::new(),
            cols: BTreeMap::new(),
        };
        for (index, ws) in um.model.workbook.worksheets.iter().enumerate() {
            let id = EntityId::Original(index as u32);
            resolver.sheets.push((id, ws.get_name()));
            resolver.rows.insert(id, AxisOrder::new(MAX_ROW, Vec::new()));
            resolver.cols.insert(id, AxisOrder::new(MAX_COLUMN, Vec::new()));
        }
        resolver
    }
}

impl RefResolver for DocResolver {
    fn sheet_id_by_name(&self, name: &str) -> Option<EntityId> {
        self.sheets
            .iter()
            .find(|(_, n)| n == name)
            .map(|(id, _)| *id)
    }
    fn sheet_name_by_id(&self, id: EntityId) -> Option<String> {
        self.sheets
            .iter()
            .find(|(i, _)| *i == id)
            .map(|(_, n)| n.clone())
    }
    fn row_id_at(&self, sheet: EntityId, index: u32) -> Option<EntityId> {
        self.rows.get(&sheet)?.id_at(index)
    }
    fn column_id_at(&self, sheet: EntityId, index: u32) -> Option<EntityId> {
        self.cols.get(&sheet)?.id_at(index)
    }
    fn resolve_row(&self, sheet: EntityId, id: EntityId) -> ResolvedIndex {
        self.rows
            .get(&sheet)
            .map(|o| o.resolve(id))
            .unwrap_or(ResolvedIndex::Unknown)
    }
    fn resolve_column(&self, sheet: EntityId, id: EntityId) -> ResolvedIndex {
        self.cols
            .get(&sheet)
            .map(|o| o.resolve(id))
            .unwrap_or(ResolvedIndex::Unknown)
    }
}

/// Largest rectangle that a single diff is allowed to touch cell-by-cell.
const MAX_RECT_CELLS: i64 = 262_144;

enum JournalEntry {
    DeletedAxis {
        sheet: EntityId,
        axis: Axis,
        ids: Vec<(EntityId, String)>,
        /// The order of the *other* axis at deletion time. The fast undo path
        /// (model-authoritative resurrect) is only valid if it is unchanged:
        /// the model restores cells by their recorded indices, which map to
        /// the right ids only in an unchanged cross layout.
        cross: AxisOrder,
    },
    DeletedSheet {
        sheet: EntityId,
        pos: String,
    },
}

/// Cached display orders derived from the shadow, evolved diff-by-diff while
/// translating a local batch so every diff is interpreted against the state it
/// was produced in.
struct OrderCtx {
    /// Visible sheets in model order: `(id, position)`.
    sheets: Vec<(EntityId, String)>,
    rows: BTreeMap<EntityId, AxisOrder>,
    cols: BTreeMap<EntityId, AxisOrder>,
}

impl OrderCtx {
    fn from_projection(proj: &Projection) -> OrderCtx {
        let mut ctx = OrderCtx {
            sheets: Vec::new(),
            rows: BTreeMap::new(),
            cols: BTreeMap::new(),
        };
        for (id, sp) in proj.visible_sheets() {
            ctx.sheets.push((id, sp.pos.clone()));
            ctx.rows.insert(id, sp.axis_order(Axis::Rows));
            ctx.cols.insert(id, sp.axis_order(Axis::Columns));
        }
        ctx
    }

    fn sheet_at(&self, index: u32) -> Result<EntityId, String> {
        self.sheets
            .get(index as usize)
            .map(|(id, _)| *id)
            .ok_or_else(|| format!("collab: sheet index {index} out of range"))
    }

    fn sheet_index(&self, id: EntityId) -> Option<u32> {
        self.sheets
            .iter()
            .position(|(sid, _)| *sid == id)
            .map(|i| i as u32)
    }

    fn order(&self, sheet: EntityId, axis: Axis) -> Result<&AxisOrder, String> {
        let map = match axis {
            Axis::Rows => &self.rows,
            Axis::Columns => &self.cols,
        };
        map.get(&sheet)
            .ok_or_else(|| "collab: unknown sheet".to_string())
    }

    fn order_mut(&mut self, sheet: EntityId, axis: Axis) -> &mut AxisOrder {
        let (map, max) = match axis {
            Axis::Rows => (&mut self.rows, MAX_ROW),
            Axis::Columns => (&mut self.cols, MAX_COLUMN),
        };
        map.entry(sheet)
            .or_insert_with(|| AxisOrder::new(max, Vec::new()))
    }
}

/// Locations whose final (post-batch) value must be copied from the model to
/// the document in pass 2.
#[derive(Default)]
struct Touched {
    /// `(sheet, column, row)`
    cells: BTreeSet<(EntityId, EntityId, EntityId)>,
    row_props: BTreeSet<(EntityId, EntityId)>,
    col_props: BTreeSet<(EntityId, EntityId)>,
    sheet_meta: BTreeSet<EntityId>,
    /// Sheets whose whole content must be pushed (duplicate/undelete).
    full_sheets: BTreeSet<EntityId>,
    keep_rows: BTreeSet<(EntityId, EntityId)>,
    keep_cols: BTreeSet<(EntityId, EntityId)>,
}

/// A live collaboration session for one [`UserModel`].
pub struct CollabSession {
    doc: Doc,
    maps: SchemaMaps,
    client_id: u64,
    counter: u32,
    sent_sv: StateVector,
    shadow: Projection,
    journal: Vec<JournalEntry>,
}

impl CollabSession {
    /// Attaches a session to a model, bootstrapping the document from the
    /// current workbook state.
    ///
    /// Bootstrap uses deterministic `Original` ids and positions derived from
    /// the workbook, so two replicas that attach to the *same* file produce
    /// convergent documents even if they both bootstrap.
    pub fn attach(um: &mut UserModel, client_id: u64) -> Result<CollabSession, String> {
        // Absorb pending local diffs into the state we bootstrap from.
        let _ = um.flush_send_queue();
        let doc = Doc::with_client_id(client_id);
        let maps = SchemaMaps::attach(&doc);
        {
            let resolver = DocResolver::pristine_from_model(um);
            let mut txn = doc.transact_mut();
            let sheet_count = um.model.workbook.worksheets.len() as u32;
            for index in 0..sheet_count {
                bootstrap_sheet(
                    &mut txn,
                    &maps,
                    um,
                    index,
                    EntityId::Original(index),
                    &resolver,
                )?;
            }
        }
        let shadow = Projection::from_doc(&doc, &maps);
        Ok(CollabSession {
            doc,
            maps,
            client_id,
            counter: 0,
            sent_sv: StateVector::default(),
            shadow,
            journal: Vec::new(),
        })
    }

    /// Translates pending local edits into the document and returns the
    /// encoded update to broadcast (everything peers have not been sent yet).
    pub fn flush_local(&mut self, um: &mut UserModel) -> Result<Vec<u8>, String> {
        self.translate_queue(um)?;
        let txn = self.doc.transact();
        let update = txn.encode_state_as_update_v1(&self.sent_sv);
        self.sent_sv = txn.state_vector();
        Ok(update)
    }

    /// Applies a remote update: pending local edits are merged into the
    /// document first, then the update is applied and the model is reconciled
    /// with the merged document state.
    pub fn apply_remote(&mut self, um: &mut UserModel, update: &[u8]) -> Result<(), String> {
        self.translate_queue(um)?;
        {
            let mut txn = self.doc.transact_mut();
            let update = Update::decode_v1(update).map_err(|e| e.to_string())?;
            txn.apply_update(update).map_err(|e| e.to_string())?;
        }
        self.reconcile(um)
    }

    /// The document state vector (v1 encoding), for sync handshakes.
    pub fn state_vector(&self) -> Vec<u8> {
        self.doc.transact().state_vector().encode_v1()
    }

    /// Everything a peer with state vector `sv` is missing (v1 encodings).
    pub fn encode_state_since(&self, sv: &[u8]) -> Result<Vec<u8>, String> {
        let sv = StateVector::decode_v1(sv).map_err(|e| e.to_string())?;
        Ok(self.doc.transact().encode_state_as_update_v1(&sv))
    }

    /// The full document as a single update (for a late joiner).
    pub fn full_state(&self) -> Vec<u8> {
        self.doc
            .transact()
            .encode_state_as_update_v1(&StateVector::default())
    }

    /// Test hook: the last projection applied to the model. Two synced
    /// replicas must have equal shadows (document-level convergence).
    #[cfg(test)]
    pub(crate) fn shadow_for_tests(&self) -> &Projection {
        &self.shadow
    }

    /// Test hook: asserts the model matches the shadow projection cell by
    /// cell over a window (the model must be a faithful rendering of the
    /// document at all times).
    #[cfg(test)]
    pub(crate) fn assert_model_matches_shadow(&self, um: &UserModel, label: &str) {
        let resolver = DocResolver::from_projection(&self.shadow);
        let visible = self.shadow.visible_sheets();
        for (index, (sheet_id, sp)) in visible.iter().enumerate() {
            let sheet = index as u32;
            let rows = sp.axis_order(Axis::Rows);
            let cols = sp.axis_order(Axis::Columns);
            for row in 1..=40u32 {
                for column in 1..=15u32 {
                    let expected = match (rows.id_at(row), cols.id_at(column)) {
                        (Some(row_id), Some(col_id)) => {
                            let raw = sp
                                .cells
                                .get(&(col_id, row_id))
                                .cloned()
                                .unwrap_or_default();
                            if is_id_form(&raw) {
                                render_formula(&raw, *sheet_id, &resolver)
                                    .unwrap_or_else(|e| format!("<render error: {e}>"))
                            } else {
                                raw
                            }
                        }
                        _ => String::new(),
                    };
                    let actual = um
                        .get_cell_content(sheet, row as i32, column as i32)
                        .unwrap_or_default();
                    if actual != expected {
                        for c in 1..=15u32 {
                            let m = um
                                .get_cell_content(sheet, row as i32, c as i32)
                                .unwrap_or_default();
                            let d = match (rows.id_at(row), cols.id_at(c)) {
                                (Some(r), Some(cc)) => {
                                    sp.cells.get(&(cc, r)).cloned().unwrap_or_default()
                                }
                                _ => String::new(),
                            };
                            if !(m.is_empty() && d.is_empty()) {
                                eprintln!("  R{row}C{c}: model={m:?} doc={d:?}");
                            }
                        }
                        panic!(
                            "{label}: model deviates from document at sheet {sheet} R{row}C{column}: model {actual:?} doc {expected:?}"
                        );
                    }
                }
            }
        }
    }

    // ---- outbound ----

    fn translate_queue(&mut self, um: &mut UserModel) -> Result<(), String> {
        let bytes = um.flush_send_queue();
        let queue: Vec<QueueDiffs> =
            bitcode::decode(&bytes).map_err(|e| format!("collab: cannot decode queue: {e}"))?;
        if queue.iter().all(|q| q.list.is_empty()) {
            return Ok(());
        }
        // Pass 1 walks every diff with an order context that evolves in step
        // with the model's own history, so ids are resolved against the state
        // each diff was produced in. Pass 2 then reads values with the FINAL
        // layout: the model already contains the whole batch, so a location
        // is read exactly where it ended up.
        let mut ctx = OrderCtx::from_projection(&self.shadow);
        let mut touched = Touched::default();
        let mut repair: BTreeSet<EntityId> = BTreeSet::new();
        self.counter += 1;
        let op_counter = self.counter;
        {
            let mut txn = self.doc.transact_mut();
            let mut pass1 = Pass1 {
                txn: &mut txn,
                maps: &self.maps,
                shadow: &self.shadow,
                ctx: &mut ctx,
                touched: &mut touched,
                journal: &mut self.journal,
                repair: &mut repair,
                client_id: self.client_id,
                counter: &mut self.counter,
            };
            for entry in &queue {
                match entry.r#type {
                    DiffType::Redo => {
                        for diff in &entry.list {
                            pass1.translate(diff, false)?;
                        }
                    }
                    // Undo applies inverses in reverse order (see
                    // `apply_undo_diff_list`); mirror that.
                    DiffType::Undo => {
                        for diff in entry.list.iter().rev() {
                            pass1.translate(diff, true)?;
                        }
                    }
                }
            }
            write_final_state(&mut txn, &self.maps, um, &ctx, &touched, self.client_id, op_counter)?;
        }
        self.shadow = Projection::from_doc(&self.doc, &self.maps);
        // An undo that resurrected rows/columns at a drifted position leaves
        // the model misaligned with the document; rebuild those sheets from
        // the document (which is authoritative).
        for sheet_id in repair {
            self.repair_sheet_from_shadow(um, sheet_id)?;
        }
        Ok(())
    }

    /// Rebuilds one sheet of the model from the (authoritative) document
    /// projection. Needed after an undo resurrects rows/columns: the model's
    /// index-based undo may have re-inserted them at a stale position if
    /// remote structural changes arrived in between, and masked cells may
    /// have been updated remotely while the rows were deleted.
    fn repair_sheet_from_shadow(&self, um: &mut UserModel, sheet_id: EntityId) -> Result<(), String> {
        let visible = self.shadow.visible_sheets();
        let Some(index) = visible.iter().position(|(id, _)| *id == sheet_id) else {
            return Ok(());
        };
        let sheet = index as u32;
        let Some(sp) = self.shadow.sheets.get(&sheet_id) else {
            return Ok(());
        };
        {
            let ws = um.model.workbook.worksheet_mut(sheet)?;
            ws.sheet_data.clear();
            ws.rows.clear();
            ws.cols.clear();
        }
        let resolver = DocResolver::from_projection(&self.shadow);
        apply_full_sheet(um, sheet, sheet_id, sp, &resolver)?;
        um.model.evaluate();
        Ok(())
    }

    // ---- inbound ----

    fn reconcile(&mut self, um: &mut UserModel) -> Result<(), String> {
        let new_proj = Projection::from_doc(&self.doc, &self.maps);
        if new_proj == self.shadow {
            return Ok(());
        }
        let old_proj = std::mem::take(&mut self.shadow);

        let old_sheets = old_proj.visible_sheets();
        let new_sheets = new_proj.visible_sheets();
        let old_ids: Vec<EntityId> = old_sheets.iter().map(|(id, _)| *id).collect();
        let new_ids: Vec<EntityId> = new_sheets.iter().map(|(id, _)| *id).collect();

        // Deterministic display names: on a duplicate-name collision the later
        // sheet (by position/id) gets a numeric suffix on every replica.
        let display_names = dedupe_names(&new_sheets);

        // Remove sheets that are no longer visible (descending indices).
        for (index, id) in old_ids.iter().enumerate().rev() {
            if !new_ids.contains(id) {
                um.model.delete_sheet(index as u32)?;
            }
        }
        // Insert new sheets at their final indices (ascending).
        for (index, (id, _)) in new_sheets.iter().enumerate() {
            if !old_ids.contains(id) {
                um.model.insert_sheet(&display_names[index], index as u32, None)?;
            }
        }
        // Names and frozen panes.
        for (index, (_, sp_new)) in new_sheets.iter().enumerate() {
            let sheet = index as u32;
            let current_name = um.model.workbook.worksheet(sheet)?.get_name();
            if current_name != display_names[index] {
                um.model
                    .rename_sheet_by_index(sheet, &display_names[index])?;
            }
            let ws = um.model.workbook.worksheet(sheet)?;
            let (fr, fc) = (ws.frozen_rows, ws.frozen_columns);
            if fr != sp_new.frozen_rows {
                um.model.set_frozen_rows(sheet, sp_new.frozen_rows)?;
            }
            if fc != sp_new.frozen_columns {
                um.model.set_frozen_columns(sheet, sp_new.frozen_columns)?;
            }
        }
        // Content. First find the sheets whose row/column order changed: any
        // such change can shift the *rendering* of id-form formulas on every
        // sheet (cross-sheet references), so even sheets on the fast delta
        // path must re-render their formulas.
        let resolver = DocResolver::from_projection(&new_proj);
        let mut structural: Vec<bool> = Vec::with_capacity(new_sheets.len());
        for (id, sp_new) in &new_sheets {
            let changed = match old_proj.sheets.get(id).filter(|_| old_ids.contains(id)) {
                Some(sp_old) => {
                    sp_old.axis_order(Axis::Rows) != sp_new.axis_order(Axis::Rows)
                        || sp_old.axis_order(Axis::Columns) != sp_new.axis_order(Axis::Columns)
                }
                None => true,
            };
            structural.push(changed);
        }
        let rerender_all = structural.iter().any(|s| *s);
        for (index, (id, sp_new)) in new_sheets.iter().enumerate() {
            let sheet = index as u32;
            match old_proj.sheets.get(id).filter(|_| old_ids.contains(id)) {
                Some(sp_old) => reconcile_sheet(
                    um,
                    sheet,
                    *id,
                    sp_old,
                    sp_new,
                    &resolver,
                    structural[index],
                    rerender_all,
                )?,
                None => apply_full_sheet(um, sheet, *id, sp_new, &resolver)?,
            }
        }
        um.model.evaluate();
        self.shadow = new_proj;
        Ok(())
    }
}

// ---- bootstrap ----

fn bootstrap_sheet(
    txn: &mut TransactionMut,
    maps: &SchemaMaps,
    um: &UserModel,
    sheet_index: u32,
    sheet_id: EntityId,
    resolver: &DocResolver,
) -> Result<(), String> {
    let ws = um.model.workbook.worksheet(sheet_index)?;
    maps.meta.insert(
        txn,
        sheet_meta_key(sheet_id, "name"),
        ws.get_name().as_str(),
    );
    maps.meta.insert(
        txn,
        sheet_meta_key(sheet_id, "pos"),
        original_position(sheet_index + 1).as_str(),
    );
    if ws.frozen_rows != 0 {
        maps.meta
            .insert(txn, sheet_meta_key(sheet_id, "fr"), ws.frozen_rows as i64);
    }
    if ws.frozen_columns != 0 {
        maps.meta
            .insert(txn, sheet_meta_key(sheet_id, "fc"), ws.frozen_columns as i64);
    }
    for row in &ws.rows {
        if row.r < 1 {
            continue;
        }
        let id = EntityId::Original(row.r as u32);
        if row.custom_height {
            // The document carries UI units (what get_row_height returns).
            maps.rows.insert(
                txn,
                axis_key(sheet_id, id, "h"),
                row.height * ROW_HEIGHT_FACTOR,
            );
        }
        if row.hidden {
            maps.rows.insert(txn, axis_key(sheet_id, id, "x"), true);
        }
    }
    for col in &ws.cols {
        for c in col.min..=col.max {
            if c < 1 {
                continue;
            }
            let id = EntityId::Original(c as u32);
            if col.custom_width {
                maps.cols.insert(
                    txn,
                    axis_key(sheet_id, id, "h"),
                    col.width * COLUMN_WIDTH_FACTOR,
                );
            }
            if col.hidden {
                maps.cols.insert(txn, axis_key(sheet_id, id, "x"), true);
            }
        }
    }
    for (row, row_cells) in &ws.sheet_data {
        for (column, cell) in row_cells {
            if matches!(cell, Cell::EmptyCell { .. } | Cell::SpillCell { .. }) {
                continue;
            }
            let content = read_cell_for_doc(um, sheet_index, *row, *column, sheet_id, resolver)?;
            if content.is_empty() {
                continue;
            }
            let (row_id, col_id) = (
                EntityId::Original(*row as u32),
                EntityId::Original(*column as u32),
            );
            maps.cells
                .insert(txn, cell_key(sheet_id, col_id, row_id), content.as_str());
        }
    }
    Ok(())
}

// ---- outbound pass 1 ----

struct Pass1<'a, 'doc> {
    txn: &'a mut TransactionMut<'doc>,
    maps: &'a SchemaMaps,
    shadow: &'a Projection,
    ctx: &'a mut OrderCtx,
    touched: &'a mut Touched,
    journal: &'a mut Vec<JournalEntry>,
    /// Sheets whose model state must be rebuilt from the document after this
    /// action (undo resurrects are doc-authoritative).
    repair: &'a mut BTreeSet<EntityId>,
    client_id: u64,
    counter: &'a mut u32,
}

impl Pass1<'_, '_> {
    fn new_id(&mut self) -> EntityId {
        *self.counter += 1;
        EntityId::Inserted {
            client: self.client_id,
            counter: *self.counter,
        }
    }

    fn translate(&mut self, diff: &Diff, invert: bool) -> Result<(), String> {
        match diff {
            // Cell content: mark the location; pass 2 reads the final value
            // from the model (works identically for do and undo).
            Diff::SetCellValue {
                sheet, row, column, ..
            } => self.touch_cell(*sheet, *row, *column),
            Diff::SetArrayValue {
                sheet,
                row,
                column,
                width,
                height,
                ..
            } => self.touch_rect(*sheet, *row, *column, *width, *height, true),
            Diff::RangeClearContents {
                sheet,
                row,
                column,
                width,
                height,
                ..
            }
            | Diff::RangeClearAll {
                sheet,
                row,
                column,
                width,
                height,
                ..
            } => self.touch_rect(*sheet, *row, *column, *width, *height, false),

            // Row/column properties.
            Diff::SetRowHeight { sheet, row, .. } | Diff::SetRowHidden { sheet, row, .. } => {
                self.touch_axis_props(*sheet, Axis::Rows, *row)
            }
            Diff::SetColumnWidth { sheet, column, .. }
            | Diff::SetColumnHidden { sheet, column, .. } => {
                self.touch_axis_props(*sheet, Axis::Columns, *column)
            }

            // Structure.
            Diff::InsertRows { sheet, row, count } => {
                if invert {
                    self.delete_axis(*sheet, Axis::Rows, *row, *count, false)
                } else {
                    self.insert_axis(*sheet, Axis::Rows, *row, *count)
                }
            }
            Diff::DeleteRows {
                sheet,
                row,
                count,
                old_data,
            } => {
                if invert {
                    let columns: Vec<Vec<i32>> = old_data
                        .iter()
                        .map(|rd| rd.data.keys().copied().collect())
                        .collect();
                    self.undo_delete_axis(*sheet, Axis::Rows, *row, *count, &columns)
                } else {
                    self.delete_axis(*sheet, Axis::Rows, *row, *count, true)
                }
            }
            Diff::InsertColumns {
                sheet,
                column,
                count,
            } => {
                if invert {
                    self.delete_axis(*sheet, Axis::Columns, *column, *count, false)
                } else {
                    self.insert_axis(*sheet, Axis::Columns, *column, *count)
                }
            }
            Diff::DeleteColumns {
                sheet,
                column,
                count,
                old_data,
            } => {
                if invert {
                    let rows: Vec<Vec<i32>> = old_data
                        .iter()
                        .map(|cd| cd.data.keys().copied().collect())
                        .collect();
                    self.undo_delete_axis(*sheet, Axis::Columns, *column, *count, &rows)
                } else {
                    self.delete_axis(*sheet, Axis::Columns, *column, *count, true)
                }
            }

            // Sheets.
            Diff::NewSheet { index, .. } => {
                if invert {
                    self.delete_sheet_at(*index, false)
                } else {
                    self.new_sheet_at(*index, false)
                }
            }
            Diff::DeleteSheet { sheet, .. } => {
                if invert {
                    self.undo_delete_sheet(*sheet)
                } else {
                    self.delete_sheet_at(*sheet, true)
                }
            }
            Diff::DuplicateSheet {
                source_index,
                new_index,
            } => {
                if invert {
                    self.delete_sheet_at(*new_index, false)
                } else {
                    let _ = source_index;
                    self.new_sheet_at(*new_index, true)
                }
            }
            Diff::RenameSheet { index, .. } => {
                let sheet_id = self.ctx.sheet_at(*index)?;
                self.touched.sheet_meta.insert(sheet_id);
                Ok(())
            }
            Diff::SetFrozenRowsCount { sheet, .. } | Diff::SetFrozenColumnsCount { sheet, .. } => {
                let sheet_id = self.ctx.sheet_at(*sheet)?;
                self.touched.sheet_meta.insert(sheet_id);
                Ok(())
            }

            // Not replicated in v1: purely visual state.
            Diff::SetCellStyle { .. }
            | Diff::ApplyNamedStyle { .. }
            | Diff::CellClearFormatting { .. }
            | Diff::SetColumnStyle { .. }
            | Diff::SetRowStyle { .. }
            | Diff::DeleteColumnStyle { .. }
            | Diff::DeleteRowStyle { .. }
            | Diff::CreateNamedStyle { .. }
            | Diff::DeleteNamedStyle { .. }
            | Diff::UpdateNamedStyle { .. }
            | Diff::AddConditionalFormatting { .. }
            | Diff::DeleteConditionalFormatting { .. }
            | Diff::UpdateConditionalFormatting { .. }
            | Diff::SwapConditionalFormattingPriority { .. }
            | Diff::SetSheetColor { .. }
            | Diff::SetSheetState { .. }
            | Diff::SetShowGridLines { .. }
            | Diff::SetTheme { .. } => Ok(()),

            // Semantic operations not yet supported: fail loudly.
            Diff::MoveRows { .. } | Diff::MoveColumns { .. } => {
                Err("collab: move rows/columns is not supported yet".to_string())
            }
            Diff::CreateDefinedName { .. }
            | Diff::DeleteDefinedName { .. }
            | Diff::UpdateDefinedName { .. } => {
                Err("collab: defined names are not supported yet".to_string())
            }
            Diff::SetLocale { .. } | Diff::SetTimezone { .. } => {
                Err("collab: locale/timezone changes are not supported yet".to_string())
            }
        }
    }

    fn touch_cell(&mut self, sheet: u32, row: i32, column: i32) -> Result<(), String> {
        let sheet_id = self.ctx.sheet_at(sheet)?;
        let row_id = self
            .ctx
            .order(sheet_id, Axis::Rows)?
            .id_at(row as u32)
            .ok_or_else(|| format!("collab: row {row} out of range"))?;
        let col_id = self
            .ctx
            .order(sheet_id, Axis::Columns)?
            .id_at(column as u32)
            .ok_or_else(|| format!("collab: column {column} out of range"))?;
        self.touched.cells.insert((sheet_id, col_id, row_id));
        self.touched.keep_rows.insert((sheet_id, row_id));
        self.touched.keep_cols.insert((sheet_id, col_id));
        Ok(())
    }

    fn touch_rect(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        width: i32,
        height: i32,
        include_empty: bool,
    ) -> Result<(), String> {
        let cells = (width as i64) * (height as i64);
        if include_empty && cells <= MAX_RECT_CELLS {
            for r in row..row + height {
                for c in column..column + width {
                    self.touch_cell(sheet, r, c)?;
                }
            }
            return Ok(());
        }
        // Large or clear-only rectangles: only existing cells can change, so
        // it is enough to touch the cells the document already has (cells
        // created earlier in this batch are already in the touched set).
        let sheet_id = self.ctx.sheet_at(sheet)?;
        let Some(sp) = self.shadow.sheets.get(&sheet_id) else {
            return Ok(());
        };
        let rows = self.ctx.order(sheet_id, Axis::Rows)?;
        let cols = self.ctx.order(sheet_id, Axis::Columns)?;
        let mut hits: Vec<(EntityId, EntityId)> = Vec::new();
        for (col_id, row_id) in sp.cells.keys() {
            let (Some(r), Some(c)) = (rows.index_of(*row_id), cols.index_of(*col_id)) else {
                continue;
            };
            let (r, c) = (r as i32, c as i32);
            if r >= row && r < row + height && c >= column && c < column + width {
                hits.push((*col_id, *row_id));
            }
        }
        for (col_id, row_id) in hits {
            self.touched.cells.insert((sheet_id, col_id, row_id));
            self.touched.keep_rows.insert((sheet_id, row_id));
            self.touched.keep_cols.insert((sheet_id, col_id));
        }
        Ok(())
    }

    fn touch_axis_props(&mut self, sheet: u32, axis: Axis, index: i32) -> Result<(), String> {
        let sheet_id = self.ctx.sheet_at(sheet)?;
        let id = self
            .ctx
            .order(sheet_id, axis)?
            .id_at(index as u32)
            .ok_or_else(|| format!("collab: index {index} out of range"))?;
        match axis {
            Axis::Rows => {
                self.touched.row_props.insert((sheet_id, id));
                self.touched.keep_rows.insert((sheet_id, id));
            }
            Axis::Columns => {
                self.touched.col_props.insert((sheet_id, id));
                self.touched.keep_cols.insert((sheet_id, id));
            }
        }
        Ok(())
    }

    fn insert_axis(&mut self, sheet: u32, axis: Axis, at: i32, count: i32) -> Result<(), String> {
        let sheet_id = self.ctx.sheet_at(sheet)?;
        let axis_map = self.maps.axis(axis).0.clone();
        for i in 0..count {
            let index = (at + i) as u32;
            let id = self.new_id();
            let (lo, hi) = self.ctx.order_mut(sheet_id, axis).insert_bounds(index);
            let EntityId::Inserted { client, counter } = id else {
                unreachable!("new_id always allocates an Inserted id");
            };
            let pos = unique_position(lo.as_deref(), hi.as_deref(), client, counter);
            axis_map.insert(&mut *self.txn, axis_key(sheet_id, id, "p"), pos.as_str());
            self.ctx.order_mut(sheet_id, axis).insert(id, pos);
            match axis {
                Axis::Rows => self.touched.keep_rows.insert((sheet_id, id)),
                Axis::Columns => self.touched.keep_cols.insert((sheet_id, id)),
            };
        }
        self.touch_all_formulas();
        Ok(())
    }

    /// Interim measure while formulas are replicated as text: a structural
    /// edit displaces formula references in the local model, so the rewritten
    /// text must be pushed for every formula cell (on every sheet — references
    /// cross sheets). The id-based formula phase removes this fan-out.
    fn touch_all_formulas(&mut self) {
        for (sheet_id, sp) in &self.shadow.sheets {
            for ((col_id, row_id), text) in &sp.cells {
                if text.starts_with('=') {
                    self.touched.cells.insert((*sheet_id, *col_id, *row_id));
                }
            }
        }
    }

    fn delete_axis(
        &mut self,
        sheet: u32,
        axis: Axis,
        at: i32,
        count: i32,
        push_journal: bool,
    ) -> Result<(), String> {
        let sheet_id = self.ctx.sheet_at(sheet)?;
        let order = self.ctx.order(sheet_id, axis)?;
        let mut ids: Vec<(EntityId, String)> = Vec::with_capacity(count as usize);
        for i in 0..count {
            let index = (at + i) as u32;
            let id = order
                .id_at(index)
                .ok_or_else(|| format!("collab: index {index} out of range"))?;
            let pos = order
                .position_of(id)
                .ok_or_else(|| "collab: inconsistent order".to_string())?;
            ids.push((id, pos));
        }
        let (axis_map, keep_map) = {
            let (a, k) = self.maps.axis(axis);
            (a.clone(), k.clone())
        };
        for (id, _) in &ids {
            axis_map.insert(&mut *self.txn, axis_key(sheet_id, *id, "d"), true);
            // Clear every keep entry this replica has seen; entries added
            // concurrently elsewhere survive and keep the row/column alive
            // (update-wins).
            let prefix = keep_prefix(sheet_id, *id);
            let seen: Vec<String> = keep_map
                .iter(&*self.txn)
                .filter(|(key, _)| key.starts_with(prefix.as_str()))
                .map(|(key, _)| key.to_string())
                .collect();
            for key in seen {
                keep_map.remove(&mut *self.txn, &key);
            }
            self.ctx.order_mut(sheet_id, axis).remove(*id);
        }
        if push_journal {
            let cross_axis = match axis {
                Axis::Rows => Axis::Columns,
                Axis::Columns => Axis::Rows,
            };
            let cross = self.ctx.order(sheet_id, cross_axis)?.clone();
            self.journal.push(JournalEntry::DeletedAxis {
                sheet: sheet_id,
                axis,
                ids,
                cross,
            });
        }
        self.touch_all_formulas();
        Ok(())
    }

    /// Undo of a local row/column deletion: resurrect the same ids so masked
    /// cells and remote references stay attached to them.
    fn undo_delete_axis(
        &mut self,
        sheet: u32,
        axis: Axis,
        at: i32,
        count: i32,
        cross_indices: &[Vec<i32>],
    ) -> Result<(), String> {
        let sheet_id = self.ctx.sheet_at(sheet)?;
        let matches = matches!(
            self.journal.last(),
            Some(JournalEntry::DeletedAxis { sheet: s, axis: a, ids, .. })
                if *s == sheet_id && *a == axis && ids.len() == count as usize
        );
        let mut cross_at_delete = None;
        let ids: Vec<(EntityId, String)> = if matches {
            match self.journal.pop() {
                Some(JournalEntry::DeletedAxis { ids, cross, .. }) => {
                    cross_at_delete = Some(cross);
                    ids
                }
                _ => unreachable!("journal entry checked above"),
            }
        } else {
            // Fallback (e.g. session attached mid-history): fresh ids.
            self.insert_axis(sheet, axis, at, count)?;
            let order = self.ctx.order(sheet_id, axis)?;
            let mut fresh = Vec::with_capacity(count as usize);
            for i in 0..count {
                let id = order
                    .id_at((at + i) as u32)
                    .ok_or_else(|| "collab: inconsistent order".to_string())?;
                let pos = order
                    .position_of(id)
                    .ok_or_else(|| "collab: inconsistent order".to_string())?;
                fresh.push((id, pos));
            }
            fresh
        };
        let axis_map = self.maps.axis(axis).0.clone();
        if matches {
            // An id may already be visible again: a concurrent positive op
            // resurrected it (update-wins). The document then treats this undo
            // as a no-op for that id — but the model's own undo has inserted a
            // duplicate row/column, so the model must be repaired.
            let mut already_visible = false;
            for (id, pos) in &ids {
                let visible = self.ctx.order(sheet_id, axis)?.index_of(*id).is_some();
                axis_map.remove(&mut *self.txn, &axis_key(sheet_id, *id, "d"));
                if visible {
                    already_visible = true;
                } else {
                    self.ctx.order_mut(sheet_id, axis).insert(*id, pos.clone());
                }
                match axis {
                    Axis::Rows => self.touched.keep_rows.insert((sheet_id, *id)),
                    Axis::Columns => self.touched.keep_cols.insert((sheet_id, *id)),
                };
            }
            // The model's own undo restored rows/cells by their *recorded
            // indices*. That matches the document only if (a) the resurrected
            // ids landed back at the same slot and (b) the other axis has the
            // same layout it had at deletion time. Structural changes in
            // between (local or remote) break either.
            let cross_axis = match axis {
                Axis::Rows => Axis::Columns,
                Axis::Columns => Axis::Rows,
            };
            let slot_moved = self
                .ctx
                .order(sheet_id, axis)?
                .index_of(ids[0].0)
                .is_none_or(|actual| actual != at as u32);
            let cross_changed = match &cross_at_delete {
                Some(cross) => cross != self.ctx.order(sheet_id, cross_axis)?,
                None => true,
            };
            let drifted = already_visible || slot_moved || cross_changed;
            if drifted {
                // The document is authoritative: repair the model from it
                // after this action. Pushing the (misplaced) model state would
                // corrupt the document, so no cell/prop/formula marks here.
                self.repair.insert(sheet_id);
            } else {
                // Model and document agree on the layout: the model's restored
                // content and re-displaced formulas are the truth to publish.
                for (offset, (id, _)) in ids.iter().enumerate() {
                    self.mark_restored_line(sheet_id, axis, *id, cross_indices.get(offset))?;
                }
                self.touch_all_formulas();
            }
            return Ok(());
        }
        // Fallback path (e.g. session attached mid-history): fresh ids were
        // inserted; the restored content only exists in the model.
        for (offset, (id, _)) in ids.iter().enumerate() {
            self.mark_restored_line(sheet_id, axis, *id, cross_indices.get(offset))?;
        }
        self.touch_all_formulas();
        Ok(())
    }

    /// Marks a resurrected row/column (and its restored cells) so pass 2
    /// publishes the model state for it.
    fn mark_restored_line(
        &mut self,
        sheet_id: EntityId,
        axis: Axis,
        id: EntityId,
        cross_indices: Option<&Vec<i32>>,
    ) -> Result<(), String> {
        match axis {
            Axis::Rows => {
                self.touched.keep_rows.insert((sheet_id, id));
                self.touched.row_props.insert((sheet_id, id));
            }
            Axis::Columns => {
                self.touched.keep_cols.insert((sheet_id, id));
                self.touched.col_props.insert((sheet_id, id));
            }
        }
        let Some(cross) = cross_indices else {
            return Ok(());
        };
        let cross_axis = match axis {
            Axis::Rows => Axis::Columns,
            Axis::Columns => Axis::Rows,
        };
        for cross_index in cross {
            let Some(cross_id) = self
                .ctx
                .order(sheet_id, cross_axis)?
                .id_at(*cross_index as u32)
            else {
                continue;
            };
            let (col_id, row_id) = match axis {
                Axis::Rows => (cross_id, id),
                Axis::Columns => (id, cross_id),
            };
            self.touched.cells.insert((sheet_id, col_id, row_id));
            self.touched.keep_rows.insert((sheet_id, row_id));
            self.touched.keep_cols.insert((sheet_id, col_id));
        }
        Ok(())
    }

    fn new_sheet_at(&mut self, index: u32, full_content: bool) -> Result<(), String> {
        let id = self.new_id();
        let lower = if index >= 1 {
            self.ctx
                .sheets
                .get(index as usize - 1)
                .map(|(_, pos)| pos.clone())
        } else {
            None
        };
        let upper = self.ctx.sheets.get(index as usize).map(|(_, pos)| pos.clone());
        let EntityId::Inserted { client, counter } = id else {
            unreachable!("new_id always allocates an Inserted id");
        };
        let pos = unique_position(lower.as_deref(), upper.as_deref(), client, counter);
        self.maps
            .meta
            .insert(&mut *self.txn, sheet_meta_key(id, "pos"), pos.as_str());
        self.ctx.sheets.insert(index as usize, (id, pos));
        self.ctx
            .rows
            .insert(id, AxisOrder::new(MAX_ROW, Vec::new()));
        self.ctx
            .cols
            .insert(id, AxisOrder::new(MAX_COLUMN, Vec::new()));
        // Name (and frozen panes) are read from the model in pass 2.
        self.touched.sheet_meta.insert(id);
        if full_content {
            self.touched.full_sheets.insert(id);
        }
        Ok(())
    }

    fn delete_sheet_at(&mut self, index: u32, push_journal: bool) -> Result<(), String> {
        if index as usize >= self.ctx.sheets.len() {
            return Err(format!("collab: sheet index {index} out of range"));
        }
        let (id, pos) = self.ctx.sheets.remove(index as usize);
        self.maps
            .meta
            .insert(&mut *self.txn, sheet_meta_key(id, "del"), true);
        if push_journal {
            self.journal
                .push(JournalEntry::DeletedSheet { sheet: id, pos });
        }
        Ok(())
    }

    fn undo_delete_sheet(&mut self, index: u32) -> Result<(), String> {
        match self.journal.pop() {
            Some(JournalEntry::DeletedSheet { sheet, pos }) => {
                self.maps
                    .meta
                    .remove(&mut *self.txn, &sheet_meta_key(sheet, "del"));
                self.ctx.sheets.insert(index as usize, (sheet, pos));
                let (rows, cols) = match self.shadow.sheets.get(&sheet) {
                    Some(sp) => (sp.axis_order(Axis::Rows), sp.axis_order(Axis::Columns)),
                    None => (
                        AxisOrder::new(MAX_ROW, Vec::new()),
                        AxisOrder::new(MAX_COLUMN, Vec::new()),
                    ),
                };
                self.ctx.rows.insert(sheet, rows);
                self.ctx.cols.insert(sheet, cols);
                self.touched.sheet_meta.insert(sheet);
                self.touched.full_sheets.insert(sheet);
                Ok(())
            }
            other => {
                if let Some(entry) = other {
                    self.journal.push(entry);
                }
                // Fallback: a fresh sheet with the restored content.
                self.new_sheet_at(index, true)
            }
        }
    }
}

// ---- outbound pass 2 ----

/// Reads the final (post-batch) model state for every touched location and
/// writes it into the document. Writing final values once makes intermediate
/// states within a batch irrelevant and handles undo uniformly.
#[allow(clippy::too_many_arguments)]
fn write_final_state(
    txn: &mut TransactionMut,
    maps: &SchemaMaps,
    um: &UserModel,
    ctx: &OrderCtx,
    touched: &Touched,
    client_id: u64,
    op_counter: u32,
) -> Result<(), String> {
    // Formula encoding resolves against the final (post-batch) orders and the
    // post-batch model sheet names.
    let resolver = DocResolver::from_ctx(ctx, um)?;

    // Keep-set entries first (only for ids still visible: an id deleted later
    // in the same batch must not be resurrected by its own earlier edit).
    for (set, axis) in [
        (&touched.keep_rows, Axis::Rows),
        (&touched.keep_cols, Axis::Columns),
    ] {
        let keep_map = maps.axis(axis).1;
        for (sheet_id, id) in set {
            if ctx.sheet_index(*sheet_id).is_none() {
                continue;
            }
            if ctx.order(*sheet_id, axis)?.index_of(*id).is_none() {
                continue;
            }
            keep_map.insert(
                txn,
                keep_key(*sheet_id, *id, client_id),
                op_counter as i64,
            );
        }
    }

    for (sheet_id, col_id, row_id) in &touched.cells {
        let Some(sheet) = ctx.sheet_index(*sheet_id) else {
            continue;
        };
        let (Some(row), Some(column)) = (
            ctx.order(*sheet_id, Axis::Rows)?.index_of(*row_id),
            ctx.order(*sheet_id, Axis::Columns)?.index_of(*col_id),
        ) else {
            continue; // masked by a later structural op in the same batch
        };
        let content =
            read_cell_for_doc(um, sheet, row as i32, column as i32, *sheet_id, &resolver)?;
        let key = cell_key(*sheet_id, *col_id, *row_id);
        if content.is_empty() {
            maps.cells.remove(txn, &key);
        } else {
            maps.cells.insert(txn, key, content.as_str());
        }
    }

    for sheet_id in &touched.full_sheets {
        let Some(sheet) = ctx.sheet_index(*sheet_id) else {
            continue;
        };
        let ws = um.model.workbook.worksheet(sheet)?;
        let rows_order = ctx.order(*sheet_id, Axis::Rows)?;
        let cols_order = ctx.order(*sheet_id, Axis::Columns)?;
        for (row, row_cells) in &ws.sheet_data {
            for (column, cell) in row_cells {
                if matches!(cell, Cell::EmptyCell { .. } | Cell::SpillCell { .. }) {
                    continue;
                }
                let content =
                    read_cell_for_doc(um, sheet, *row, *column, *sheet_id, &resolver)?;
                if content.is_empty() {
                    continue;
                }
                let (Some(_), Some(_)) = (
                    rows_order.index_of(EntityId::Original(*row as u32)),
                    cols_order.index_of(EntityId::Original(*column as u32)),
                ) else {
                    continue;
                };
                let (row_id, col_id) = (
                    EntityId::Original(*row as u32),
                    EntityId::Original(*column as u32),
                );
                maps.cells
                    .insert(txn, cell_key(*sheet_id, col_id, row_id), content.as_str());
            }
        }
        for row in &ws.rows {
            if row.r < 1 {
                continue;
            }
            let id = EntityId::Original(row.r as u32);
            if row.custom_height {
                maps.rows.insert(
                    txn,
                    axis_key(*sheet_id, id, "h"),
                    row.height * ROW_HEIGHT_FACTOR,
                );
            }
            if row.hidden {
                maps.rows.insert(txn, axis_key(*sheet_id, id, "x"), true);
            }
        }
    }

    for (sheet_id, row_id) in &touched.row_props {
        let Some(sheet) = ctx.sheet_index(*sheet_id) else {
            continue;
        };
        let Some(row) = ctx.order(*sheet_id, Axis::Rows)?.index_of(*row_id) else {
            continue;
        };
        let ws = um.model.workbook.worksheet(sheet)?;
        let entry = ws.rows.iter().find(|r| r.r == row as i32);
        let height_key = axis_key(*sheet_id, *row_id, "h");
        match entry {
            Some(r) if r.custom_height => {
                maps.rows
                    .insert(txn, height_key, r.height * ROW_HEIGHT_FACTOR);
            }
            _ => {
                maps.rows.remove(txn, &height_key);
            }
        }
        let hidden_key = axis_key(*sheet_id, *row_id, "x");
        match entry {
            Some(r) if r.hidden => {
                maps.rows.insert(txn, hidden_key, true);
            }
            _ => {
                maps.rows.remove(txn, &hidden_key);
            }
        }
    }

    for (sheet_id, col_id) in &touched.col_props {
        let Some(sheet) = ctx.sheet_index(*sheet_id) else {
            continue;
        };
        let Some(column) = ctx.order(*sheet_id, Axis::Columns)?.index_of(*col_id) else {
            continue;
        };
        let ws = um.model.workbook.worksheet(sheet)?;
        let entry = ws
            .cols
            .iter()
            .find(|c| c.min <= column as i32 && column as i32 <= c.max);
        let width_key = axis_key(*sheet_id, *col_id, "h");
        match entry {
            Some(c) if c.custom_width => {
                maps.cols
                    .insert(txn, width_key, c.width * COLUMN_WIDTH_FACTOR);
            }
            _ => {
                maps.cols.remove(txn, &width_key);
            }
        }
        let hidden_key = axis_key(*sheet_id, *col_id, "x");
        match entry {
            Some(c) if c.hidden => {
                maps.cols.insert(txn, hidden_key, true);
            }
            _ => {
                maps.cols.remove(txn, &hidden_key);
            }
        }
    }

    for sheet_id in &touched.sheet_meta {
        let Some(sheet) = ctx.sheet_index(*sheet_id) else {
            continue;
        };
        let ws = um.model.workbook.worksheet(sheet)?;
        maps.meta.insert(
            txn,
            sheet_meta_key(*sheet_id, "name"),
            ws.get_name().as_str(),
        );
        let fr_key = sheet_meta_key(*sheet_id, "fr");
        if ws.frozen_rows != 0 {
            maps.meta.insert(txn, fr_key, ws.frozen_rows as i64);
        } else {
            maps.meta.remove(txn, &fr_key);
        }
        let fc_key = sheet_meta_key(*sheet_id, "fc");
        if ws.frozen_columns != 0 {
            maps.meta.insert(txn, fc_key, ws.frozen_columns as i64);
        } else {
            maps.meta.remove(txn, &fc_key);
        }
    }

    Ok(())
}

/// The replicated form of a cell: empty for blank and spill cells (spills are
/// recomputed downstream, never shipped), id-form for formulas (canonical
/// text with stable-id reference tokens), plain input text otherwise.
///
/// A formula the codec cannot represent (structured references, …) falls back
/// to plain localized text; the structural-op fan-out keeps those convergent.
fn read_cell_for_doc(
    um: &UserModel,
    sheet: u32,
    row: i32,
    column: i32,
    sheet_id: EntityId,
    resolver: &DocResolver,
) -> Result<String, String> {
    let ws = um.model.workbook.worksheet(sheet)?;
    match ws.cell(row, column) {
        None | Some(Cell::EmptyCell { .. }) | Some(Cell::SpillCell { .. }) => Ok(String::new()),
        Some(cell) => {
            if cell.get_formula().is_some() {
                if let Some(canonical) = um.model.get_english_cell_formula(sheet, row, column)? {
                    if let Ok(id_form) = encode_formula(&canonical, sheet_id, resolver) {
                        return Ok(id_form);
                    }
                }
            }
            um.get_cell_content(sheet, row, column)
        }
    }
}

// ---- inbound helpers ----

fn dedupe_names(sheets: &[(EntityId, &SheetProj)]) -> Vec<String> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut names = Vec::with_capacity(sheets.len());
    for (_, sp) in sheets {
        let base = if sp.name.is_empty() { "Sheet" } else { &sp.name };
        let mut candidate = base.to_string();
        let mut n = 1;
        while seen.contains(&candidate.to_lowercase()) {
            n += 1;
            candidate = format!("{base} ({n})");
        }
        seen.insert(candidate.to_lowercase());
        names.push(candidate);
    }
    names
}

#[allow(clippy::too_many_arguments)]
fn reconcile_sheet(
    um: &mut UserModel,
    sheet: u32,
    sheet_id: EntityId,
    sp_old: &SheetProj,
    sp_new: &SheetProj,
    resolver: &DocResolver,
    structural: bool,
    rerender_all: bool,
) -> Result<(), String> {
    if !structural {
        // Cell deltas only: same coordinates on both sides. When any *other*
        // sheet changed structurally, id-form formulas here may render
        // differently (cross-sheet references), so they are re-set as well.
        for (key, new_value) in &sp_new.cells {
            let changed = sp_old.cells.get(key) != Some(new_value);
            let rerender = rerender_all && is_id_form(new_value);
            if changed || rerender {
                set_projected_cell(um, sheet, sheet_id, resolver, key, new_value)?;
            }
        }
        for key in sp_old.cells.keys() {
            if sp_new.cells.contains_key(key) {
                continue;
            }
            set_projected_cell(um, sheet, sheet_id, resolver, key, "")?;
        }
        let rows_new = resolver
            .rows
            .get(&sheet_id)
            .ok_or("collab: unknown sheet in resolver")?;
        let cols_new = resolver
            .cols
            .get(&sheet_id)
            .ok_or("collab: unknown sheet in resolver")?;
        // Property deltas.
        let row_ids: BTreeSet<EntityId> = sp_old.rows.keys().chain(sp_new.rows.keys()).copied().collect();
        for id in row_ids {
            let old_entry = sp_old.rows.get(&id);
            let new_entry = sp_new.rows.get(&id);
            let old_props = old_entry.map(|e| (e.size, e.hidden)).unwrap_or((None, false));
            let new_props = new_entry.map(|e| (e.size, e.hidden)).unwrap_or((None, false));
            if old_props == new_props {
                continue;
            }
            let Some(row) = rows_new.index_of(id) else {
                continue;
            };
            apply_row_props(um, sheet, row as i32, new_props.0, new_props.1)?;
        }
        let col_ids: BTreeSet<EntityId> = sp_old.cols.keys().chain(sp_new.cols.keys()).copied().collect();
        for id in col_ids {
            let old_entry = sp_old.cols.get(&id);
            let new_entry = sp_new.cols.get(&id);
            let old_props = old_entry.map(|e| (e.size, e.hidden)).unwrap_or((None, false));
            let new_props = new_entry.map(|e| (e.size, e.hidden)).unwrap_or((None, false));
            if old_props == new_props {
                continue;
            }
            let Some(column) = cols_new.index_of(id) else {
                continue;
            };
            apply_column_props(um, sheet, column as i32, new_props.0, new_props.1)?;
        }
        return Ok(());
    }

    // Structural change: conservative rebuild. The model's cells do not move
    // by themselves (we never replay insert/delete on remote), so shifting is
    // simulated by clearing every old location and writing every new one.
    let rows_old = sp_old.axis_order(Axis::Rows);
    let cols_old = sp_old.axis_order(Axis::Columns);
    for key in sp_old.cells.keys() {
        let (col_id, row_id) = key;
        let (Some(row), Some(column)) = (rows_old.index_of(*row_id), cols_old.index_of(*col_id))
        else {
            continue;
        };
        um.model
            .set_user_input(sheet, row as i32, column as i32, String::new())?;
    }
    // Reset old row/column properties.
    let mut prop_rows: Vec<i32> = Vec::new();
    for (id, e) in &sp_old.rows {
        if e.size.is_some() || e.hidden {
            if let Some(row) = rows_old.index_of(*id) {
                prop_rows.push(row as i32);
            }
        }
    }
    if !prop_rows.is_empty() {
        let ws = um.model.workbook.worksheet_mut(sheet)?;
        ws.rows.retain(|r| !prop_rows.contains(&r.r));
    }
    for (id, e) in &sp_old.cols {
        if e.size.is_some() || e.hidden {
            if let Some(column) = cols_old.index_of(*id) {
                apply_column_props(um, sheet, column as i32, None, false)?;
            }
        }
    }
    // Write the new state.
    apply_sheet_content(um, sheet, sheet_id, sp_new, resolver)
}

fn apply_full_sheet(
    um: &mut UserModel,
    sheet: u32,
    sheet_id: EntityId,
    sp: &SheetProj,
    resolver: &DocResolver,
) -> Result<(), String> {
    apply_sheet_content(um, sheet, sheet_id, sp, resolver)
}

fn apply_sheet_content(
    um: &mut UserModel,
    sheet: u32,
    sheet_id: EntityId,
    sp: &SheetProj,
    resolver: &DocResolver,
) -> Result<(), String> {
    for (key, value) in &sp.cells {
        set_projected_cell(um, sheet, sheet_id, resolver, key, value)?;
    }
    let rows = resolver
        .rows
        .get(&sheet_id)
        .ok_or("collab: unknown sheet in resolver")?;
    let cols = resolver
        .cols
        .get(&sheet_id)
        .ok_or("collab: unknown sheet in resolver")?;
    for (id, e) in &sp.rows {
        if e.size.is_none() && !e.hidden {
            continue;
        }
        let Some(row) = rows.index_of(*id) else {
            continue;
        };
        apply_row_props(um, sheet, row as i32, e.size, e.hidden)?;
    }
    for (id, e) in &sp.cols {
        if e.size.is_none() && !e.hidden {
            continue;
        }
        let Some(column) = cols.index_of(*id) else {
            continue;
        };
        apply_column_props(um, sheet, column as i32, e.size, e.hidden)?;
    }
    Ok(())
}

fn set_projected_cell(
    um: &mut UserModel,
    sheet: u32,
    sheet_id: EntityId,
    resolver: &DocResolver,
    key: &(EntityId, EntityId),
    value: &str,
) -> Result<(), String> {
    let (col_id, row_id) = key;
    let (Some(rows), Some(cols)) = (resolver.rows.get(&sheet_id), resolver.cols.get(&sheet_id))
    else {
        return Err("collab: unknown sheet in resolver".to_string());
    };
    let (Some(row), Some(column)) = (rows.index_of(*row_id), cols.index_of(*col_id)) else {
        return Ok(()); // masked: its row or column is currently deleted
    };
    let text = if is_id_form(value) {
        render_formula(value, sheet_id, resolver)?
    } else {
        value.to_string()
    };
    um.model
        .set_user_input(sheet, row as i32, column as i32, text)
}

fn apply_row_props(
    um: &mut UserModel,
    sheet: u32,
    row: i32,
    height: Option<f64>,
    hidden: bool,
) -> Result<(), String> {
    match height {
        Some(h) => um.model.set_row_height(sheet, row, h)?,
        None => {
            // No custom height: drop the row record and re-add what is needed.
            let ws = um.model.workbook.worksheet_mut(sheet)?;
            ws.rows.retain(|r| r.r != row);
        }
    }
    um.model.set_row_hidden(sheet, row, hidden)?;
    if height.is_none() && !hidden {
        // set_row_hidden(false) may have materialized a default row record.
        let ws = um.model.workbook.worksheet_mut(sheet)?;
        ws.rows.retain(|r| r.r != row);
    }
    Ok(())
}

fn apply_column_props(
    um: &mut UserModel,
    sheet: u32,
    column: i32,
    width: Option<f64>,
    hidden: bool,
) -> Result<(), String> {
    um.model
        .set_column_width(sheet, column, width.unwrap_or(DEFAULT_COLUMN_WIDTH))?;
    um.model.set_column_hidden(sheet, column, hidden)?;
    Ok(())
}

// Silence an unused-constant warning until row-height defaults are needed.
const _: f64 = DEFAULT_ROW_HEIGHT;
