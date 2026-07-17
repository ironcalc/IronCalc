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
//! Current scope: cell content and styles (content-addressed pool), formulas
//! in id-form, rows/columns (insert/delete/move, props, styles, update-wins
//! keep-sets), sheets (CRUD, keep-sets, settings), defined names, named-style
//! definitions, workbook locale/timezone. Not replicated yet: conditional
//! formatting, borders as shared edges, merged cells, themes.
//!
//! Known styles limitation: applying a *named* style links the cell locally
//! but replicates the flattened result, so updating a named style definition
//! re-resolves cells only where links exist; the flattened doc entries catch
//! up when the linking replica pushes (conservative re-marking keeps the
//! originator consistent).

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

use super::formula::{
    encode_formula, needs_reencode, is_id_form, render_formula, RefResolver,
};
use super::ids::{EntityId, MAX_COLUMN, MAX_ROW};
use super::order::{original_position, unique_position, AxisOrder, ResolvedIndex};
use super::projection::{
    axis_key, cell_key, keep_key, keep_prefix, name_key, parse_name_key, sheet_keep_key,
    sheet_keep_prefix, sheet_meta_key, Axis, Projection, SchemaMaps, SheetProj,
};

/// Ensures a style body is in the pool and returns its hash.
fn ensure_style_in_pool(
    txn: &mut TransactionMut,
    maps: &SchemaMaps,
    style: &Style,
) -> String {
    let bytes = bitcode::encode(style);
    let hash = style_pool_hash(&bytes);
    if maps.styles.get(&*txn, &hash).is_none() {
        maps.styles
            .insert(txn, hash.as_str(), yrs::Any::from(bytes));
    }
    hash
}

/// Decodes a style from the projection's pool.
fn style_from_pool(proj: &Projection, hash: &str) -> Result<Style, String> {
    let bytes = proj
        .styles
        .get(hash)
        .ok_or("collab: missing style pool entry")?;
    bitcode::decode(bytes).map_err(|e| format!("collab: corrupt style body: {e}"))
}
use crate::types::{Color, SheetState, Style, StyleIncludes};

/// Deterministic 128-bit FNV-1a over the style's bitcode bytes — the key of
/// the content-addressed style pool. Interning is content-based, so two
/// replicas defining the same style converge on the same key by construction
/// (no shared `xf_id` allocation ever crosses the wire).
fn style_pool_hash(bytes: &[u8]) -> String {
    const OFFSET: u128 = 0x6c62272e07bb014262b821756295c58d;
    const PRIME: u128 = 0x0000000001000000000000000000013b;
    let mut hash = OFFSET;
    for byte in bytes {
        hash ^= *byte as u128;
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{hash:032x}")
}

/// Session codec for tab colors (no serde_json in non-dev deps).
fn color_to_doc(color: &Color) -> Option<String> {
    match color {
        Color::None => None,
        Color::Rgb(rgb) => Some(format!("r{rgb}")),
        Color::Theme(index, tint) => Some(format!("t{index};{tint}")),
    }
}

fn color_from_doc(text: &str) -> Result<Color, String> {
    if let Some(rgb) = text.strip_prefix('r') {
        return Ok(Color::Rgb(rgb.to_string()));
    }
    if let Some(rest) = text.strip_prefix('t') {
        let (index, tint) = rest
            .split_once(';')
            .ok_or("collab: malformed theme color")?;
        return Ok(Color::Theme(
            index.parse().map_err(|_| "collab: malformed theme color")?,
            tint.parse().map_err(|_| "collab: malformed theme color")?,
        ));
    }
    Err("collab: malformed color".to_string())
}

fn state_to_doc(state: &SheetState) -> Option<String> {
    match state {
        SheetState::Visible => None,
        SheetState::Hidden => Some("hidden".to_string()),
        SheetState::VeryHidden => Some("veryHidden".to_string()),
    }
}

fn state_from_doc(text: &str) -> Result<SheetState, String> {
    match text {
        "hidden" => Ok(SheetState::Hidden),
        "veryHidden" => Ok(SheetState::VeryHidden),
        _ => Err("collab: malformed sheet state".to_string()),
    }
}

/// Sentinel "own sheet" used when encoding/rendering defined-name formulas:
/// their references must be sheet-qualified, so a sheet-less reference
/// resolves against this unknown sheet and degrades deterministically
/// (encode → plain-text fallback, render → `#REF!`).
const NAME_SCOPE_SENTINEL: EntityId = EntityId::Inserted {
    client: u64::MAX,
    counter: u32::MAX,
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

/// Name-less resolver over the translation context, for the overflow scan
/// (payload sheet fields carry ids, so no name resolution is needed).
struct CtxResolver<'a> {
    ctx: &'a OrderCtx,
}

impl RefResolver for CtxResolver<'_> {
    fn sheet_id_by_name(&self, _name: &str) -> Option<EntityId> {
        None
    }
    fn sheet_name_by_id(&self, _id: EntityId) -> Option<String> {
        None
    }
    fn row_id_at(&self, sheet: EntityId, index: u32) -> Option<EntityId> {
        self.ctx.rows.get(&sheet)?.id_at(index)
    }
    fn column_id_at(&self, sheet: EntityId, index: u32) -> Option<EntityId> {
        self.ctx.cols.get(&sheet)?.id_at(index)
    }
    fn resolve_row(&self, sheet: EntityId, id: EntityId) -> ResolvedIndex {
        self.ctx
            .rows
            .get(&sheet)
            .map(|o| o.resolve(id))
            .unwrap_or(ResolvedIndex::Unknown)
    }
    fn resolve_column(&self, sheet: EntityId, id: EntityId) -> ResolvedIndex {
        self.ctx
            .cols
            .get(&sheet)
            .map(|o| o.resolve(id))
            .unwrap_or(ResolvedIndex::Unknown)
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
    /// Re-sync the defined-name map from the post-batch model (set by
    /// defined-name diffs and by structural ops, which displace name
    /// formulas in the model).
    names: bool,
    /// Re-sync workbook-level settings (locale, timezone).
    workbook: bool,
    /// `(sheet, column, row)` whose *style* must be re-read (independent of
    /// content: concurrent style and content edits both survive).
    cell_styles: BTreeSet<(EntityId, EntityId, EntityId)>,
    /// Re-sync the named-style definitions from the post-batch model.
    named_styles: bool,
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
            sync_names(&mut txn, &maps, um, &resolver)?;
            sync_named_styles(&mut txn, &maps, um)?;
            let settings = &um.model.workbook.settings;
            maps.meta
                .insert(&mut txn, "wb.locale", settings.locale.as_str());
            maps.meta.insert(&mut txn, "wb.tz", settings.tz.as_str());
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
            write_final_state(
                &mut txn,
                &self.maps,
                um,
                &ctx,
                &touched,
                &self.shadow,
                self.client_id,
                op_counter,
            )?;
        }
        self.shadow = Projection::from_doc(&self.doc, &self.maps);
        // An undo that resurrected rows/columns at a drifted position leaves
        // the model misaligned with the document; rebuild those sheets from
        // the document (which is authoritative).
        for sheet_id in repair {
            self.repair_sheet_from_shadow(um, sheet_id)?;
        }
        // A local sheet creation/deletion can change the deterministic
        // display names (e.g. dissolve a duplicate-name suffix).
        let display_names = dedupe_names(&self.shadow.visible_sheets());
        align_sheet_display_names(um, &display_names)?;
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
        apply_full_sheet(um, sheet, sheet_id, sp, &resolver, &self.shadow)?;
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

        // Align the model's sheet list with the document's visible list.
        // `working` tracks model indices by id throughout.
        let mut working: Vec<EntityId> = old_ids.clone();
        let mut deferred: Vec<EntityId> = Vec::new();
        for id in old_ids.iter().rev() {
            if new_ids.contains(id) {
                continue;
            }
            let Some(index) = working.iter().position(|x| x == id) else {
                continue;
            };
            match um.model.delete_sheet(index as u32) {
                Ok(()) => {
                    working.remove(index);
                }
                // The engine refuses to delete the last sheet; retry after
                // the insertions below (the document guarantees at least one
                // visible sheet).
                Err(_) => deferred.push(*id),
            }
        }
        if new_ids.iter().any(|id| !working.contains(id)) || !deferred.is_empty() {
            for (i, id) in new_ids.iter().enumerate() {
                if working.contains(id) {
                    continue;
                }
                // Insert right after its closest predecessor already present.
                let at = new_ids[..i]
                    .iter()
                    .rev()
                    .find_map(|p| working.iter().position(|x| x == p))
                    .map_or(0, |p| p + 1);
                um.model
                    .insert_sheet(&format!("collab-new-{i}"), at as u32, None)?;
                working.insert(at, *id);
            }
            for id in deferred {
                let Some(index) = working.iter().position(|x| *x == id) else {
                    continue;
                };
                um.model.delete_sheet(index as u32)?;
                working.remove(index);
            }
        }
        debug_assert_eq!(working, new_ids, "sheet alignment failed");
        align_sheet_display_names(um, &display_names)?;
        // Frozen panes and per-sheet settings.
        for (index, (_, sp_new)) in new_sheets.iter().enumerate() {
            let sheet = index as u32;
            let ws = um.model.workbook.worksheet(sheet)?;
            let (fr, fc) = (ws.frozen_rows, ws.frozen_columns);
            if fr != sp_new.frozen_rows {
                um.model.set_frozen_rows(sheet, sp_new.frozen_rows)?;
            }
            if fc != sp_new.frozen_columns {
                um.model.set_frozen_columns(sheet, sp_new.frozen_columns)?;
            }
            let desired_color = match &sp_new.color {
                Some(text) => color_from_doc(text)?,
                None => Color::None,
            };
            let desired_state = match &sp_new.state {
                Some(text) => state_from_doc(text)?,
                None => SheetState::Visible,
            };
            let desired_grid = sp_new.grid_lines.unwrap_or(true);
            let ws = um.model.workbook.worksheet(sheet)?;
            if ws.color != desired_color {
                um.model.set_sheet_color(sheet, &desired_color)?;
            }
            let ws = um.model.workbook.worksheet(sheet)?;
            if ws.state != desired_state {
                um.model.set_sheet_state(sheet, desired_state)?;
            }
            let ws = um.model.workbook.worksheet(sheet)?;
            if ws.show_grid_lines != desired_grid {
                um.model.set_show_grid_lines(sheet, desired_grid)?;
            }
        }
        // Workbook-level registers.
        if let Some(locale) = &new_proj.locale {
            if um.model.workbook.settings.locale != *locale {
                um.model.set_locale(locale)?;
            }
        }
        if let Some(timezone) = &new_proj.timezone {
            if um.model.workbook.settings.tz != *timezone {
                um.model.set_timezone(timezone)?;
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
        // Defined names: apply doc state to the model when the map changed or
        // any structural change shifted the rendering of id-form formulas.
        if old_proj.names != new_proj.names || rerender_all {
            reconcile_names(um, &new_proj, &resolver)?;
        }
        if old_proj.named_styles != new_proj.named_styles {
            reconcile_named_styles(um, &new_proj)?;
        }
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
                    &new_proj,
                    structural[index],
                    rerender_all,
                )?,
                None => apply_full_sheet(um, sheet, *id, sp_new, &resolver, &new_proj)?,
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
    if let Some(color) = color_to_doc(&ws.color) {
        maps.meta
            .insert(txn, sheet_meta_key(sheet_id, "color"), color.as_str());
    }
    if let Some(state) = state_to_doc(&ws.state) {
        maps.meta
            .insert(txn, sheet_meta_key(sheet_id, "state"), state.as_str());
    }
    if !ws.show_grid_lines {
        maps.meta
            .insert(txn, sheet_meta_key(sheet_id, "grid"), false);
    }
    let default_style = um.model.workbook.styles.get_style(0)?;
    let mut row_style_writes: Vec<(EntityId, Style)> = Vec::new();
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
        if row.custom_format {
            row_style_writes.push((id, um.model.workbook.styles.get_style(row.s)?));
        }
    }
    let mut col_style_writes: Vec<(EntityId, Style)> = Vec::new();
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
            if let Some(index) = col.style {
                col_style_writes.push((id, um.model.workbook.styles.get_style(index)?));
            }
        }
    }
    for (id, style) in row_style_writes {
        let hash = ensure_style_in_pool(txn, maps, &style);
        maps.rows
            .insert(txn, axis_key(sheet_id, id, "sty"), hash.as_str());
    }
    for (id, style) in col_style_writes {
        let hash = ensure_style_in_pool(txn, maps, &style);
        maps.cols
            .insert(txn, axis_key(sheet_id, id, "sty"), hash.as_str());
    }
    let mut cell_style_writes: Vec<(String, Style)> = Vec::new();
    for (row, row_cells) in &ws.sheet_data {
        for (column, cell) in row_cells {
            if matches!(cell, Cell::SpillCell { .. }) {
                continue;
            }
            let (row_id, col_id) = (
                EntityId::Original(*row as u32),
                EntityId::Original(*column as u32),
            );
            if !matches!(cell, Cell::EmptyCell { .. }) {
                let content =
                    read_cell_for_doc(um, sheet_index, *row, *column, sheet_id, resolver)?;
                if !content.is_empty() {
                    maps.cells
                        .insert(txn, cell_key(sheet_id, col_id, row_id), content.as_str());
                }
            }
            let style = um.model.get_style_for_cell(sheet_index, *row, *column)?;
            if style != default_style {
                cell_style_writes.push((cell_key(sheet_id, col_id, row_id), style));
            }
        }
    }
    for (key, style) in cell_style_writes {
        let hash = ensure_style_in_pool(txn, maps, &style);
        maps.cell_styles.insert(txn, key, hash.as_str());
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
            } => self.touch_rect(*sheet, *row, *column, *width, *height, false),
            Diff::RangeClearAll {
                sheet,
                row,
                column,
                width,
                height,
                ..
            } => {
                self.touch_rect(*sheet, *row, *column, *width, *height, false)?;
                self.touch_rect_styles(*sheet, *row, *column, *width, *height)
            }

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
            Diff::RenameSheet { index, .. }
            | Diff::SetSheetColor { index, .. }
            | Diff::SetSheetState { index, .. } => {
                let sheet_id = self.ctx.sheet_at(*index)?;
                self.touched.sheet_meta.insert(sheet_id);
                Ok(())
            }
            Diff::SetFrozenRowsCount { sheet, .. }
            | Diff::SetFrozenColumnsCount { sheet, .. }
            | Diff::SetShowGridLines { sheet, .. } => {
                let sheet_id = self.ctx.sheet_at(*sheet)?;
                self.touched.sheet_meta.insert(sheet_id);
                Ok(())
            }
            // Workbook-level LWW registers, read from the post-batch model.
            Diff::SetLocale { .. } | Diff::SetTimezone { .. } => {
                self.touched.workbook = true;
                Ok(())
            }

            // Styles: like cell content, pass 2 reads the final resolved
            // style from the model. `ApplyNamedStyle` replicates the
            // *resolved* style; the link to the named style stays local
            // (documented limitation: a later named-style update re-resolves
            // only where links exist — mitigated by the conservative marking
            // in the named-style arms below).
            Diff::SetCellStyle {
                sheet, row, column, ..
            }
            | Diff::ApplyNamedStyle {
                sheet, row, column, ..
            }
            | Diff::CellClearFormatting {
                sheet, row, column, ..
            } => self.touch_cell_style(*sheet, *row, *column),
            Diff::SetRowStyle { sheet, row, .. } | Diff::DeleteRowStyle { sheet, row, .. } => {
                self.touch_axis_props(*sheet, Axis::Rows, *row)
            }
            Diff::SetColumnStyle { sheet, column, .. }
            | Diff::DeleteColumnStyle { sheet, column, .. } => {
                self.touch_axis_props(*sheet, Axis::Columns, *column)
            }
            Diff::CreateNamedStyle { .. } => {
                self.touched.named_styles = true;
                Ok(())
            }
            Diff::DeleteNamedStyle { .. } | Diff::UpdateNamedStyle { .. } => {
                self.touched.named_styles = true;
                // Updating/deleting a named style re-resolves every cell that
                // links to it in this model. Which cells those are is not
                // visible from outside, so conservatively re-read every
                // styled location.
                self.touch_all_styled_locations();
                Ok(())
            }

            // Not replicated in v1: purely visual state.
            Diff::AddConditionalFormatting { .. }
            | Diff::DeleteConditionalFormatting { .. }
            | Diff::UpdateConditionalFormatting { .. }
            | Diff::SwapConditionalFormattingPriority { .. }
            | Diff::SetTheme { .. } => Ok(()),

            // Moves: pure position rewrites. The diff already carries the
            // hidden-rows-adjusted delta (resolved by `move_rows_action`
            // before the diff is recorded), so the translation never depends
            // on replica-local hidden state. Undo replays the inverse move at
            // current indices — exactly how the model applies it.
            Diff::MoveRows {
                sheet,
                row,
                row_count,
                delta,
            } => {
                if invert {
                    self.move_axis(*sheet, Axis::Rows, *row + *delta, *row_count, -*delta)
                } else {
                    self.move_axis(*sheet, Axis::Rows, *row, *row_count, *delta)
                }
            }
            Diff::MoveColumns {
                sheet,
                column,
                column_count,
                delta,
            } => {
                if invert {
                    self.move_axis(
                        *sheet,
                        Axis::Columns,
                        *column + *delta,
                        *column_count,
                        -*delta,
                    )
                } else {
                    self.move_axis(*sheet, Axis::Columns, *column, *column_count, *delta)
                }
            }
            // Defined names: pass 2 re-syncs the whole (small) name map from
            // the post-batch model, which handles create/update/delete/rename
            // and their undos uniformly.
            Diff::CreateDefinedName { .. } | Diff::DeleteDefinedName { .. } => {
                self.touched.names = true;
                Ok(())
            }
            Diff::UpdateDefinedName {
                name, new_name, ..
            } => {
                self.touched.names = true;
                // A rename rewrites every dependent cell formula in the model
                // (both directions, for undo); push those cells too.
                if !name.eq_ignore_ascii_case(new_name) {
                    self.touch_cells_mentioning(name);
                    self.touch_cells_mentioning(new_name);
                }
                Ok(())
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
        // Content operations can change the style too: an undo restores the
        // whole old cell (or removes it), quote prefixes and date inputs
        // restyle. Pass 2 skips the style write when it is unchanged, so the
        // style register stays independent for plain content edits.
        self.touched.cell_styles.insert((sheet_id, col_id, row_id));
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
            self.touched.cell_styles.insert((sheet_id, col_id, row_id));
            self.touched.keep_rows.insert((sheet_id, row_id));
            self.touched.keep_cols.insert((sheet_id, col_id));
        }
        Ok(())
    }

    fn touch_cell_style(&mut self, sheet: u32, row: i32, column: i32) -> Result<(), String> {
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
        self.touched.cell_styles.insert((sheet_id, col_id, row_id));
        self.touched.keep_rows.insert((sheet_id, row_id));
        self.touched.keep_cols.insert((sheet_id, col_id));
        Ok(())
    }

    /// Marks existing styled cells inside a rectangle (a clear can only
    /// change cells that have a style; batch-earlier writes are marked
    /// already).
    fn touch_rect_styles(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        width: i32,
        height: i32,
    ) -> Result<(), String> {
        let sheet_id = self.ctx.sheet_at(sheet)?;
        let Some(sp) = self.shadow.sheets.get(&sheet_id) else {
            return Ok(());
        };
        let rows = self.ctx.order(sheet_id, Axis::Rows)?;
        let cols = self.ctx.order(sheet_id, Axis::Columns)?;
        let mut hits: Vec<(EntityId, EntityId)> = Vec::new();
        for (col_id, row_id) in sp.cell_styles.keys() {
            let (Some(r), Some(c)) = (rows.index_of(*row_id), cols.index_of(*col_id)) else {
                continue;
            };
            let (r, c) = (r as i32, c as i32);
            if r >= row && r < row + height && c >= column && c < column + width {
                hits.push((*col_id, *row_id));
            }
        }
        for (col_id, row_id) in hits {
            self.touched.cell_styles.insert((sheet_id, col_id, row_id));
        }
        Ok(())
    }

    /// Conservatively marks every styled location known to the document
    /// (used when a named-style change re-resolves an unknown set of cells).
    fn touch_all_styled_locations(&mut self) {
        let mut cell_marks: Vec<(EntityId, EntityId, EntityId)> = Vec::new();
        let mut row_marks: Vec<(EntityId, EntityId)> = Vec::new();
        let mut col_marks: Vec<(EntityId, EntityId)> = Vec::new();
        for (sheet_id, sp) in &self.shadow.sheets {
            for (col_id, row_id) in sp.cell_styles.keys() {
                cell_marks.push((*sheet_id, *col_id, *row_id));
            }
            for (id, entry) in &sp.rows {
                if entry.style.is_some() {
                    row_marks.push((*sheet_id, *id));
                }
            }
            for (id, entry) in &sp.cols {
                if entry.style.is_some() {
                    col_marks.push((*sheet_id, *id));
                }
            }
        }
        self.touched.cell_styles.extend(cell_marks);
        self.touched.row_props.extend(row_marks);
        self.touched.col_props.extend(col_marks);
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
        self.touch_at_risk_formulas();
        Ok(())
    }

    /// Marks the formulas a structural edit puts at risk, so pass 2 pushes
    /// their post-displacement model text:
    ///
    /// * **plain-text fallbacks** (formulas the codec could not encode): their
    ///   references are positional, so the local displacement must be fanned
    ///   out — the pre-id-form mechanism, kept only for these;
    /// * **id-form formulas with an overflowed reference** (shifted past the
    ///   end of the grid): the engine renders those as out-of-grid
    ///   *identifiers* (`=A1048577`) which freeze — identifiers are never
    ///   displaced back — so the doc entry is demoted to the frozen plain
    ///   text by re-encoding the model's rendering.
    ///
    /// Id-form formulas otherwise need nothing here: their stored form is
    /// displacement-invariant and receivers re-render.
    fn touch_at_risk_formulas(&mut self) {
        // Structural edits also displace defined-name formulas in the model;
        // the name map is re-synced in pass 2 (id-form entries are no-ops).
        self.touched.names = true;
        let mut marks: Vec<(EntityId, EntityId, EntityId)> = Vec::new();
        let resolver = CtxResolver { ctx: self.ctx };
        for (sheet_id, sp) in &self.shadow.sheets {
            for ((col_id, row_id), text) in &sp.cells {
                let at_risk = if is_id_form(text) {
                    needs_reencode(text, *sheet_id, &resolver)
                } else {
                    text.starts_with('=')
                };
                if at_risk {
                    marks.push((*sheet_id, *col_id, *row_id));
                }
            }
        }
        for mark in marks {
            self.touched.cells.insert(mark);
        }
    }

    /// Marks every formula cell whose text mentions `name` (case-insensitive,
    /// conservative): a defined-name rename rewrites those formulas in the
    /// model, so their doc entries must be re-encoded.
    fn touch_cells_mentioning(&mut self, name: &str) {
        let needle = name.to_uppercase();
        let mut marks: Vec<(EntityId, EntityId, EntityId)> = Vec::new();
        for (sheet_id, sp) in &self.shadow.sheets {
            for ((col_id, row_id), text) in &sp.cells {
                if text.starts_with('=') && text.to_uppercase().contains(&needle) {
                    marks.push((*sheet_id, *col_id, *row_id));
                }
            }
        }
        for mark in marks {
            self.touched.cells.insert(mark);
        }
    }

    /// Moves `count` rows/columns from display index `from` so the block ends
    /// up starting at `from + delta` (the engine's block semantics): identity
    /// is a map key, so a move is a last-write-wins overwrite of the position
    /// registers — it cannot duplicate a line, and concurrent moves of the
    /// same line resolve to the latest one. Cells, properties and keep-sets
    /// travel with the ids untouched; id-form formula references follow
    /// automatically.
    fn move_axis(
        &mut self,
        sheet: u32,
        axis: Axis,
        from: i32,
        count: i32,
        delta: i32,
    ) -> Result<(), String> {
        if count <= 0 || delta == 0 {
            return Ok(());
        }
        let sheet_id = self.ctx.sheet_at(sheet)?;
        let order = self.ctx.order(sheet_id, axis)?;
        let mut ids = Vec::with_capacity(count as usize);
        for i in 0..count {
            let index = (from + i) as u32;
            ids.push(
                order
                    .id_at(index)
                    .ok_or_else(|| format!("collab: index {index} out of range"))?,
            );
        }
        for id in &ids {
            self.ctx.order_mut(sheet_id, axis).remove(*id);
        }
        // In the block-less order, the destination is right before the
        // element currently at `from + delta` (holds for both directions).
        let axis_map = self.maps.axis(axis).0.clone();
        for (slot, id) in ((from + delta) as u32..).zip(ids.iter()) {
            let (lo, hi) = self.ctx.order_mut(sheet_id, axis).insert_bounds(slot);
            *self.counter += 1;
            let pos = unique_position(lo.as_deref(), hi.as_deref(), self.client_id, *self.counter);
            axis_map.insert(&mut *self.txn, axis_key(sheet_id, *id, "p"), pos.as_str());
            self.ctx.order_mut(sheet_id, axis).insert(*id, pos);
            // A move is a positive op: it preempts a concurrent deletion
            // (update-wins), matching the AegisSheet semantics.
            match axis {
                Axis::Rows => self.touched.keep_rows.insert((sheet_id, *id)),
                Axis::Columns => self.touched.keep_cols.insert((sheet_id, *id)),
            };
        }
        self.touch_at_risk_formulas();
        Ok(())
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
            self.journal.push(JournalEntry::DeletedAxis {
                sheet: sheet_id,
                axis,
                ids,
            });
        }
        self.touch_at_risk_formulas();
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
            Some(JournalEntry::DeletedAxis { sheet: s, axis: a, ids })
                if *s == sheet_id && *a == axis && ids.len() == count as usize
        );
        let ids: Vec<(EntityId, String)> = if matches {
            match self.journal.pop() {
                Some(JournalEntry::DeletedAxis { ids, .. }) => ids,
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
            for (id, pos) in &ids {
                // An id may already be visible again: a concurrent positive
                // op resurrected it (update-wins); the undo is then a no-op
                // for it — never insert it into the order twice.
                let visible = self.ctx.order(sheet_id, axis)?.index_of(*id).is_some();
                axis_map.remove(&mut *self.txn, &axis_key(sheet_id, *id, "d"));
                if !visible {
                    self.ctx.order_mut(sheet_id, axis).insert(*id, pos.clone());
                }
                match axis {
                    Axis::Rows => self.touched.keep_rows.insert((sheet_id, *id)),
                    Axis::Columns => self.touched.keep_cols.insert((sheet_id, *id)),
                };
            }
            // The resurrect is doc-authoritative: the model's own index-based
            // undo may have re-inserted the line at a stale slot (remote
            // structural drift), duplicated an update-wins-resurrected line,
            // and it loses `#REF!` references the document can heal (id
            // tokens pointing at the resurrected line render again). Rebuild
            // the model sheet from the document after this batch.
            self.repair.insert(sheet_id);
            // Fallback formulas still carry the model's re-displaced text;
            // push them before the repair pulls the merged state back.
            self.touch_at_risk_formulas();
            return Ok(());
        }
        // Fallback path (e.g. session attached mid-history): fresh ids were
        // inserted; the restored content only exists in the model.
        for (offset, (id, _)) in ids.iter().enumerate() {
            self.mark_restored_line(sheet_id, axis, *id, cross_indices.get(offset))?;
        }
        self.touch_at_risk_formulas();
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
        // Clear the keep entries this replica has seen; concurrent positive
        // ops elsewhere survive and keep the sheet alive (update-wins).
        let prefix = sheet_keep_prefix(id);
        let seen: Vec<String> = self
            .maps
            .keep_sheets
            .iter(&*self.txn)
            .filter(|(key, _)| key.starts_with(prefix.as_str()))
            .map(|(key, _)| key.to_string())
            .collect();
        for key in seen {
            self.maps.keep_sheets.remove(&mut *self.txn, &key);
        }
        if push_journal {
            self.journal
                .push(JournalEntry::DeletedSheet { sheet: id, pos });
        }
        Ok(())
    }

    fn undo_delete_sheet(&mut self, index: u32) -> Result<(), String> {
        match self.journal.pop() {
            Some(JournalEntry::DeletedSheet { sheet, pos }) => {
                // The undo resurrects the same sheet id only when that is
                // coherent with the document: the id must still be invisible
                // (a concurrent positive op may have update-wins-resurrected
                // it — the model's undo then re-created a duplicate) and its
                // positional slot must equal the index the model re-inserted
                // at. Otherwise the model's new sheet is registered as a
                // fresh sheet.
                let already_visible = self.ctx.sheets.iter().any(|(id, _)| *id == sheet);
                let doc_slot = self
                    .ctx
                    .sheets
                    .iter()
                    .filter(|(id, p)| (p.as_str(), *id) < (pos.as_str(), sheet))
                    .count() as u32;
                if already_visible || doc_slot != index {
                    return self.new_sheet_at(index, true);
                }
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
    shadow: &Projection,
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

    let default_style = um.model.workbook.styles.get_style(0)?;

    for sheet_id in &touched.full_sheets {
        let Some(sheet) = ctx.sheet_index(*sheet_id) else {
            continue;
        };
        let ws = um.model.workbook.worksheet(sheet)?;
        let rows_order = ctx.order(*sheet_id, Axis::Rows)?;
        let cols_order = ctx.order(*sheet_id, Axis::Columns)?;
        let mut style_writes: Vec<(String, Style)> = Vec::new();
        for (row, row_cells) in &ws.sheet_data {
            for (column, cell) in row_cells {
                if matches!(cell, Cell::SpillCell { .. }) {
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
                if !matches!(cell, Cell::EmptyCell { .. }) {
                    let content =
                        read_cell_for_doc(um, sheet, *row, *column, *sheet_id, &resolver)?;
                    if !content.is_empty() {
                        maps.cells.insert(
                            txn,
                            cell_key(*sheet_id, col_id, row_id),
                            content.as_str(),
                        );
                    }
                }
                let style = um.model.get_style_for_cell(sheet, *row, *column)?;
                if style != default_style {
                    style_writes.push((cell_key(*sheet_id, col_id, row_id), style));
                }
            }
        }
        for (key, style) in style_writes {
            let hash = ensure_style_in_pool(txn, maps, &style);
            maps.cell_styles.insert(txn, key, hash.as_str());
        }
        let row_styles: Vec<(EntityId, Style)> = ws
            .rows
            .iter()
            .filter(|row| row.r >= 1 && row.custom_format)
            .map(|row| {
                Ok((
                    EntityId::Original(row.r as u32),
                    um.model.workbook.styles.get_style(row.s)?,
                ))
            })
            .collect::<Result<_, String>>()?;
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
        for (id, style) in row_styles {
            let hash = ensure_style_in_pool(txn, maps, &style);
            maps.rows
                .insert(txn, axis_key(*sheet_id, id, "sty"), hash.as_str());
        }
        let mut col_writes: Vec<(EntityId, Option<f64>, bool, Option<Style>)> = Vec::new();
        for col in &ws.cols {
            for c in col.min..=col.max {
                if c < 1 {
                    continue;
                }
                let style = match col.style {
                    Some(index) => Some(um.model.workbook.styles.get_style(index)?),
                    None => None,
                };
                col_writes.push((
                    EntityId::Original(c as u32),
                    col.custom_width.then_some(col.width * COLUMN_WIDTH_FACTOR),
                    col.hidden,
                    style,
                ));
            }
        }
        for (id, width, hidden, style) in col_writes {
            if let Some(width) = width {
                maps.cols.insert(txn, axis_key(*sheet_id, id, "h"), width);
            }
            if hidden {
                maps.cols.insert(txn, axis_key(*sheet_id, id, "x"), true);
            }
            if let Some(style) = style {
                let hash = ensure_style_in_pool(txn, maps, &style);
                maps.cols
                    .insert(txn, axis_key(*sheet_id, id, "sty"), hash.as_str());
            }
        }
    }

    for (sheet_id, col_id, row_id) in &touched.cell_styles {
        let Some(sheet) = ctx.sheet_index(*sheet_id) else {
            continue;
        };
        let (Some(row), Some(column)) = (
            ctx.order(*sheet_id, Axis::Rows)?.index_of(*row_id),
            ctx.order(*sheet_id, Axis::Columns)?.index_of(*col_id),
        ) else {
            continue;
        };
        let style = um.model.get_style_for_cell(sheet, row as i32, column as i32)?;
        let key = cell_key(*sheet_id, *col_id, *row_id);
        // Only write on change (vs the pre-batch shadow): content edits mark
        // styles conservatively, and an unconditional rewrite would stomp a
        // concurrent style edit, breaking the registers' independence.
        let previous = shadow
            .sheets
            .get(sheet_id)
            .and_then(|sp| sp.cell_styles.get(&(*col_id, *row_id)));
        if style == default_style {
            if previous.is_some() {
                maps.cell_styles.remove(txn, &key);
            }
        } else {
            let hash = ensure_style_in_pool(txn, maps, &style);
            if previous.map(String::as_str) != Some(hash.as_str()) {
                maps.cell_styles.insert(txn, key, hash.as_str());
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
        let style_key = axis_key(*sheet_id, *row_id, "sty");
        match entry {
            Some(r) if r.custom_format => {
                let style = um.model.workbook.styles.get_style(r.s)?;
                let hash = ensure_style_in_pool(txn, maps, &style);
                maps.rows.insert(txn, style_key, hash.as_str());
            }
            _ => {
                maps.rows.remove(txn, &style_key);
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
        let style_key = axis_key(*sheet_id, *col_id, "sty");
        match entry.and_then(|c| c.style) {
            Some(index) => {
                let style = um.model.workbook.styles.get_style(index)?;
                let hash = ensure_style_in_pool(txn, maps, &style);
                maps.cols.insert(txn, style_key, hash.as_str());
            }
            None => {
                maps.cols.remove(txn, &style_key);
            }
        }
    }

    if touched.names {
        sync_names(txn, maps, um, &resolver)?;
    }

    if touched.named_styles {
        sync_named_styles(txn, maps, um)?;
    }

    if touched.workbook {
        let settings = &um.model.workbook.settings;
        maps.meta
            .insert(txn, "wb.locale", settings.locale.as_str());
        maps.meta.insert(txn, "wb.tz", settings.tz.as_str());
    }

    // Sheet keep-sets: every positive op on a sheet keeps it alive against a
    // concurrent deletion (update-wins at sheet granularity).
    let mut positive_sheets: BTreeSet<EntityId> = BTreeSet::new();
    positive_sheets.extend(touched.cells.iter().map(|(s, _, _)| *s));
    positive_sheets.extend(touched.row_props.iter().map(|(s, _)| *s));
    positive_sheets.extend(touched.col_props.iter().map(|(s, _)| *s));
    positive_sheets.extend(touched.keep_rows.iter().map(|(s, _)| *s));
    positive_sheets.extend(touched.keep_cols.iter().map(|(s, _)| *s));
    positive_sheets.extend(touched.sheet_meta.iter().copied());
    positive_sheets.extend(touched.full_sheets.iter().copied());
    positive_sheets.extend(touched.cell_styles.iter().map(|(s, _, _)| *s));
    for sheet_id in positive_sheets {
        if ctx.sheet_index(sheet_id).is_none() {
            continue; // deleted later in the same batch
        }
        maps.keep_sheets.insert(
            txn,
            sheet_keep_key(sheet_id, client_id),
            op_counter as i64,
        );
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
        let color_key = sheet_meta_key(*sheet_id, "color");
        match color_to_doc(&ws.color) {
            Some(color) => {
                maps.meta.insert(txn, color_key, color.as_str());
            }
            None => {
                maps.meta.remove(txn, &color_key);
            }
        }
        let state_key = sheet_meta_key(*sheet_id, "state");
        match state_to_doc(&ws.state) {
            Some(state) => {
                maps.meta.insert(txn, state_key, state.as_str());
            }
            None => {
                maps.meta.remove(txn, &state_key);
            }
        }
        let grid_key = sheet_meta_key(*sheet_id, "grid");
        if ws.show_grid_lines {
            maps.meta.remove(txn, &grid_key);
        } else {
            maps.meta.insert(txn, grid_key, false);
        }
    }

    Ok(())
}

/// Re-syncs the document's defined-name map from the post-batch model state:
/// writes changed/new entries, removes vanished ones. Formulas are encoded to
/// id-form where possible (sheet-qualified references), so structural edits
/// never rewrite them in the document.
fn sync_names(
    txn: &mut TransactionMut,
    maps: &SchemaMaps,
    um: &UserModel,
    resolver: &DocResolver,
) -> Result<(), String> {
    let mut desired: BTreeMap<String, String> = BTreeMap::new();
    for (name, scope_index, formula) in um.model.get_defined_name_list() {
        let scope = match scope_index {
            None => None,
            // Index-aligned with the resolver's sheet list by invariant.
            Some(index) => match resolver.sheets.get(index as usize) {
                Some((id, _)) => Some(*id),
                None => continue,
            },
        };
        let value = encode_formula(&formula, NAME_SCOPE_SENTINEL, resolver)
            .unwrap_or_else(|_| formula.clone());
        desired.insert(name_key(scope, &name), value);
    }
    let current: Vec<(String, Option<String>)> = maps
        .names
        .iter(&*txn)
        .map(|(key, value)| {
            let text = match value {
                yrs::Out::Any(yrs::Any::String(s)) => Some(s.to_string()),
                _ => None,
            };
            (key.to_string(), text)
        })
        .collect();
    for (key, value) in &current {
        match desired.get(key) {
            Some(wanted) if Some(wanted) == value.as_ref() => {}
            Some(wanted) => {
                maps.names.insert(txn, key.as_str(), wanted.as_str());
            }
            None => {
                maps.names.remove(txn, key);
            }
        }
    }
    for (key, wanted) in &desired {
        if !current.iter().any(|(k, _)| k == key) {
            maps.names.insert(txn, key.as_str(), wanted.as_str());
        }
    }
    Ok(())
}

/// Re-syncs the named-style definitions from the post-batch model.
fn sync_named_styles(
    txn: &mut TransactionMut,
    maps: &SchemaMaps,
    um: &UserModel,
) -> Result<(), String> {
    let mut desired: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for name in um.get_named_style_list() {
        let style = um.get_named_style(&name)?;
        let includes = um.get_named_style_includes(&name)?;
        desired.insert(name, bitcode::encode(&(style, includes)));
    }
    let current: Vec<(String, Option<Vec<u8>>)> = maps
        .named_styles
        .iter(&*txn)
        .map(|(key, value)| {
            let bytes = match value {
                yrs::Out::Any(yrs::Any::Buffer(b)) => Some(b.to_vec()),
                _ => None,
            };
            (key.to_string(), bytes)
        })
        .collect();
    for (key, value) in &current {
        match desired.get(key) {
            Some(wanted) if Some(wanted) == value.as_ref() => {}
            Some(wanted) => {
                maps.named_styles
                    .insert(txn, key.as_str(), yrs::Any::from(wanted.clone()));
            }
            None => {
                maps.named_styles.remove(txn, key);
            }
        }
    }
    for (key, wanted) in &desired {
        if !current.iter().any(|(k, _)| k == key) {
            maps.named_styles
                .insert(txn, key.as_str(), yrs::Any::from(wanted.clone()));
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

/// Applies the document's defined-name map to the model: renders id-form
/// formulas against the current orders, then diffs against the model's list
/// (delete + recreate on change — dependent cell rewrites arrive as ordinary
/// cell updates from the renaming replica, so no in-model rename is needed).
fn reconcile_names(
    um: &mut UserModel,
    proj: &Projection,
    resolver: &DocResolver,
) -> Result<(), String> {
    let mut desired: Vec<(String, Option<u32>, String)> = Vec::new();
    for (key, value) in &proj.names {
        let Some((scope_entity, name)) = parse_name_key(key) else {
            continue;
        };
        let scope_index = match scope_entity {
            None => None,
            Some(id) => {
                match resolver.sheets.iter().position(|(sid, _)| *sid == id) {
                    Some(index) => Some(index as u32),
                    // The scope sheet is deleted; skip the name for now.
                    None => continue,
                }
            }
        };
        let text = if is_id_form(value) {
            render_formula(value, NAME_SCOPE_SENTINEL, resolver)?
        } else {
            value.clone()
        };
        desired.push((name.to_string(), scope_index, text));
    }
    let current = um.model.get_defined_name_list();
    // Drop names that vanished or changed…
    for (name, scope, formula) in &current {
        let keep = desired
            .iter()
            .any(|(n, s, f)| n.eq_ignore_ascii_case(name) && s == scope && f == formula);
        if !keep {
            um.model.delete_defined_name(name, *scope)?;
        }
    }
    // …then create what is missing. Errors (e.g. a case-variant duplicate
    // from a concurrent create) are skipped deterministically: both replicas
    // process the same key order over the same converged state.
    for (name, scope, formula) in &desired {
        let exists = um
            .model
            .get_defined_name_list()
            .iter()
            .any(|(n, s, f)| n.eq_ignore_ascii_case(name) && s == scope && f == formula);
        if !exists {
            let _ = um.model.new_defined_name(name, *scope, formula);
        }
    }
    Ok(())
}

/// Applies the document's named-style definitions to the model.
///
/// Known limitation (documented in the design doc): updating a definition
/// re-resolves the cells *linked* to it in this model; links are local (the
/// replicated per-cell styles are flattened), so a replica that applied a
/// named style locally re-resolves those cells while the flattened doc values
/// only catch up when the originating replica pushes them.
fn reconcile_named_styles(um: &mut UserModel, proj: &Projection) -> Result<(), String> {
    let mut desired: BTreeMap<String, (Style, StyleIncludes)> = BTreeMap::new();
    for (name, bytes) in &proj.named_styles {
        let decoded: (Style, StyleIncludes) = bitcode::decode(bytes)
            .map_err(|e| format!("collab: corrupt named style body: {e}"))?;
        desired.insert(name.clone(), decoded);
    }
    for name in um.get_named_style_list() {
        if !desired.contains_key(&name) {
            um.model.workbook.styles.delete_named_style_entry(&name)?;
        }
    }
    for (name, (style, includes)) in &desired {
        if um.get_named_style_list().contains(name) {
            let current_style = um.get_named_style(name)?;
            let current_includes = um.get_named_style_includes(name)?;
            if current_style != *style || current_includes != *includes {
                um.model.update_named_style(name, name, style, *includes)?;
            }
        } else {
            um.model
                .workbook
                .styles
                .create_named_style(name, style, *includes)?;
        }
    }
    Ok(())
}

fn dedupe_names(sheets: &[(EntityId, &SheetProj)]) -> Vec<String> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut names = Vec::with_capacity(sheets.len());
    for (_, sp) in sheets {
        let base = if sp.name.is_empty() { "Sheet" } else { &sp.name };
        let mut candidate = base.to_string();
        let mut n = 1;
        while seen.contains(&candidate.to_lowercase()) {
            n += 1;
            // Sheet names are capped at 31 chars; make room for the suffix.
            let suffix = format!(" ({n})");
            let max_base = 31usize.saturating_sub(suffix.chars().count());
            let truncated: String = base.chars().take(max_base).collect();
            candidate = format!("{truncated}{suffix}");
        }
        seen.insert(candidate.to_lowercase());
        names.push(candidate);
    }
    names
}

/// Renames the model's sheets to the deterministic display names derived
/// from the document (two-phase: direct renames can collide transiently on
/// name swaps, concurrent renames of different sheets to the same name, or
/// placeholder names of freshly inserted sheets). Must run after both remote
/// applies *and* local translation: a local sheet deletion can dissolve a
/// name collision and change another sheet's display name.
fn align_sheet_display_names(um: &mut UserModel, display_names: &[String]) -> Result<(), String> {
    let mut current_names: Vec<String> = Vec::with_capacity(display_names.len());
    for index in 0..display_names.len() {
        current_names.push(um.model.workbook.worksheet(index as u32)?.get_name());
    }
    if current_names == display_names {
        return Ok(());
    }
    let mut salt = 0usize;
    loop {
        let prefix = format!("collab-tmp{salt}-");
        if !current_names.iter().any(|n| n.starts_with(&prefix)) {
            break;
        }
        salt += 1;
    }
    for (index, current) in current_names.iter().enumerate() {
        if *current != display_names[index] {
            um.model
                .rename_sheet_by_index(index as u32, &format!("collab-tmp{salt}-{index}"))?;
        }
    }
    for (index, current) in current_names.iter().enumerate() {
        if *current != display_names[index] {
            um.model
                .rename_sheet_by_index(index as u32, &display_names[index])?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn reconcile_sheet(
    um: &mut UserModel,
    sheet: u32,
    sheet_id: EntityId,
    sp_old: &SheetProj,
    sp_new: &SheetProj,
    resolver: &DocResolver,
    proj: &Projection,
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
        // Cell style deltas (an independent register per cell).
        let style_keys: BTreeSet<(EntityId, EntityId)> = sp_old
            .cell_styles
            .keys()
            .chain(sp_new.cell_styles.keys())
            .copied()
            .collect();
        for key in style_keys {
            let old_hash = sp_old.cell_styles.get(&key);
            let new_hash = sp_new.cell_styles.get(&key);
            if old_hash == new_hash {
                continue;
            }
            set_projected_cell_style(um, sheet, sheet_id, resolver, proj, &key, new_hash)?;
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
            let old_props = old_entry
                .map(|e| (e.size, e.hidden, e.style.clone()))
                .unwrap_or((None, false, None));
            let new_props = new_entry
                .map(|e| (e.size, e.hidden, e.style.clone()))
                .unwrap_or((None, false, None));
            if old_props == new_props {
                continue;
            }
            let Some(row) = rows_new.index_of(id) else {
                continue;
            };
            let style = pool_style_opt(proj, new_props.2.as_deref())?;
            apply_row_props(um, sheet, row as i32, new_props.0, new_props.1, style.as_ref())?;
        }
        let col_ids: BTreeSet<EntityId> = sp_old.cols.keys().chain(sp_new.cols.keys()).copied().collect();
        for id in col_ids {
            let old_entry = sp_old.cols.get(&id);
            let new_entry = sp_new.cols.get(&id);
            let old_props = old_entry
                .map(|e| (e.size, e.hidden, e.style.clone()))
                .unwrap_or((None, false, None));
            let new_props = new_entry
                .map(|e| (e.size, e.hidden, e.style.clone()))
                .unwrap_or((None, false, None));
            if old_props == new_props {
                continue;
            }
            let Some(column) = cols_new.index_of(id) else {
                continue;
            };
            let style = pool_style_opt(proj, new_props.2.as_deref())?;
            apply_column_props(
                um,
                sheet,
                column as i32,
                new_props.0,
                new_props.1,
                style.as_ref(),
            )?;
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
    // Clear old cell styles at their old positions.
    let default_style = um.model.workbook.styles.get_style(0)?;
    for key in sp_old.cell_styles.keys() {
        let (col_id, row_id) = key;
        let (Some(row), Some(column)) = (rows_old.index_of(*row_id), cols_old.index_of(*col_id))
        else {
            continue;
        };
        um.model
            .set_cell_style(sheet, row as i32, column as i32, &default_style)?;
    }
    // Reset old row/column properties.
    let mut prop_rows: Vec<i32> = Vec::new();
    for (id, e) in &sp_old.rows {
        if e.size.is_some() || e.hidden || e.style.is_some() {
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
        if e.size.is_some() || e.hidden || e.style.is_some() {
            if let Some(column) = cols_old.index_of(*id) {
                apply_column_props(um, sheet, column as i32, None, false, None)?;
            }
        }
    }
    // Write the new state.
    apply_sheet_content(um, sheet, sheet_id, sp_new, resolver, proj)
}

fn apply_full_sheet(
    um: &mut UserModel,
    sheet: u32,
    sheet_id: EntityId,
    sp: &SheetProj,
    resolver: &DocResolver,
    proj: &Projection,
) -> Result<(), String> {
    apply_sheet_content(um, sheet, sheet_id, sp, resolver, proj)
}

fn apply_sheet_content(
    um: &mut UserModel,
    sheet: u32,
    sheet_id: EntityId,
    sp: &SheetProj,
    resolver: &DocResolver,
    proj: &Projection,
) -> Result<(), String> {
    for (key, value) in &sp.cells {
        set_projected_cell(um, sheet, sheet_id, resolver, key, value)?;
    }
    for (key, hash) in &sp.cell_styles {
        set_projected_cell_style(um, sheet, sheet_id, resolver, proj, key, Some(hash))?;
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
        if e.size.is_none() && !e.hidden && e.style.is_none() {
            continue;
        }
        let Some(row) = rows.index_of(*id) else {
            continue;
        };
        let style = pool_style_opt(proj, e.style.as_deref())?;
        apply_row_props(um, sheet, row as i32, e.size, e.hidden, style.as_ref())?;
    }
    for (id, e) in &sp.cols {
        if e.size.is_none() && !e.hidden && e.style.is_none() {
            continue;
        }
        let Some(column) = cols.index_of(*id) else {
            continue;
        };
        let style = pool_style_opt(proj, e.style.as_deref())?;
        apply_column_props(um, sheet, column as i32, e.size, e.hidden, style.as_ref())?;
    }
    Ok(())
}

fn pool_style_opt(proj: &Projection, hash: Option<&str>) -> Result<Option<Style>, String> {
    match hash {
        Some(hash) => Ok(Some(style_from_pool(proj, hash)?)),
        None => Ok(None),
    }
}

fn set_projected_cell_style(
    um: &mut UserModel,
    sheet: u32,
    sheet_id: EntityId,
    resolver: &DocResolver,
    proj: &Projection,
    key: &(EntityId, EntityId),
    hash: Option<&String>,
) -> Result<(), String> {
    let (col_id, row_id) = key;
    let (Some(rows), Some(cols)) = (resolver.rows.get(&sheet_id), resolver.cols.get(&sheet_id))
    else {
        return Err("collab: unknown sheet in resolver".to_string());
    };
    let (Some(row), Some(column)) = (rows.index_of(*row_id), cols.index_of(*col_id)) else {
        return Ok(()); // masked
    };
    let style = match hash {
        Some(hash) => style_from_pool(proj, hash)?,
        None => um.model.workbook.styles.get_style(0)?,
    };
    um.model
        .set_cell_style(sheet, row as i32, column as i32, &style)
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
    style: Option<&Style>,
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
    match style {
        Some(style) => um.model.set_row_style(sheet, row, style)?,
        None => {
            let has_style = um
                .model
                .workbook
                .worksheet(sheet)?
                .rows
                .iter()
                .any(|r| r.r == row && r.custom_format);
            if has_style {
                um.model.delete_row_style(sheet, row)?;
            }
        }
    }
    if height.is_none() && !hidden && style.is_none() {
        // The setters above may have materialized a default row record.
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
    style: Option<&Style>,
) -> Result<(), String> {
    um.model
        .set_column_width(sheet, column, width.unwrap_or(DEFAULT_COLUMN_WIDTH))?;
    um.model.set_column_hidden(sheet, column, hidden)?;
    match style {
        Some(style) => um.model.set_column_style(sheet, column, style)?,
        None => {
            let has_style = um
                .model
                .workbook
                .worksheet(sheet)?
                .cols
                .iter()
                .any(|c| c.min <= column && column <= c.max && c.style.is_some());
            if has_style {
                um.model.delete_column_style(sheet, column)?;
            }
        }
    }
    Ok(())
}

// Silence an unused-constant warning until row-height defaults are needed.
const _: f64 = DEFAULT_ROW_HEIGHT;
