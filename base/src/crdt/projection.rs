//! The replicated document schema and its projection.
//!
//! The yrs document is a set of **flat root maps with composite string keys**.
//! Deliberately no nested shared types: concurrent creation of the same nested
//! map resolves whole-subtree LWW in Yjs/yrs and silently drops one side's
//! writes, whereas flat keys make every conflict an independent key-level LWW.
//!
//! Root maps and key formats (`<sid>`/`<rid>`/`<cid>` are [`EntityId`]
//! encodings; the id charset is disjoint from the separators `. ! : /`):
//!
//! | map         | key                        | value                          |
//! |-------------|----------------------------|--------------------------------|
//! | `meta`      | `wb.name` / `.locale` / `.tz` | workbook LWW registers      |
//! |             | `wb.theme`                 | bitcode of `Theme`             |
//! |             | `s.<sid>.name`             | sheet name (string)            |
//! |             | `s.<sid>.pos`              | fractional position (string)   |
//! |             | `s.<sid>.del`              | `true` (tombstone)             |
//! |             | `s.<sid>.fr` / `.fc`       | frozen rows / columns (int)    |
//! | `cells`     | `<sid>!<cid>:<rid>`        | user input (string)            |
//! | `rows`      | `<sid>!<rid>.p`            | fractional position (string)   |
//! |             | `<sid>!<rid>.h`            | row height (number)            |
//! |             | `<sid>!<rid>.x`            | hidden (bool)                  |
//! |             | `<sid>!<rid>.d`            | `true` (tombstone)             |
//! | `cols`      | same fields as `rows`      | (`.h` is the column width)     |
//! | `keep_rows` | `<sid>!<rid>/<client36>`   | op counter (int) — keep-set    |
//! | `keep_cols` | `<sid>!<cid>/<client36>`   | op counter (int)               |
//! | `cf`        | `<sid>!<ruleId>.p`         | fractional position (string)   |
//! |             | `<sid>!<ruleId>.v`         | bitcode `(range, rule, dxf)`   |
//! | `edges`     | `<sid>!v.<cid>:<rid>`      | border item (line left of col) |
//! |             | `<sid>!h.<cid>:<rid>`      | border item (line top of row)  |
//!
//! Update-wins deletion: a row/column is visible iff it has no `.d` tombstone
//! OR its keep-set is non-empty. Deleting clears the keep entries the deleter
//! has *seen*; a concurrent positive op adds an unseen entry that survives the
//! clear, so the row stays visible with all its (masked, never erased) cells.
//!
//! [`Projection`] is a plain-Rust snapshot of the document used to (a) diff
//! remote changes against the last applied state and (b) derive the
//! [`AxisOrder`]s that map stable ids to display indices.

use std::collections::{BTreeMap, HashSet};

use yrs::{Any, Doc, Map, MapRef, Out, Transact};

use super::ids::{EntityId, MAX_COLUMN, MAX_ROW};
use super::order::AxisOrder;

pub(crate) const MAP_META: &str = "meta";
pub(crate) const MAP_CELLS: &str = "cells";
pub(crate) const MAP_ROWS: &str = "rows";
pub(crate) const MAP_COLS: &str = "cols";
pub(crate) const MAP_KEEP_ROWS: &str = "keep_rows";
pub(crate) const MAP_KEEP_COLS: &str = "keep_cols";
pub(crate) const MAP_KEEP_SHEETS: &str = "keep_sheets";
pub(crate) const MAP_NAMES: &str = "names";
pub(crate) const MAP_STYLES: &str = "styles";
pub(crate) const MAP_CELL_STYLES: &str = "cell_styles";
pub(crate) const MAP_NAMED_STYLES: &str = "named_styles";
pub(crate) const MAP_CF: &str = "cf";
pub(crate) const MAP_EDGES: &str = "edges";
pub(crate) const MAP_LINKS: &str = "links";
pub(crate) const MAP_MERGES: &str = "merges";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Axis {
    Rows,
    Columns,
}

/// Handles to the root maps of the document.
#[derive(Clone)]
pub(crate) struct SchemaMaps {
    pub meta: MapRef,
    pub cells: MapRef,
    pub rows: MapRef,
    pub cols: MapRef,
    pub keep_rows: MapRef,
    pub keep_cols: MapRef,
    pub keep_sheets: MapRef,
    pub names: MapRef,
    /// Content-addressed style pool: fnv1a-128 hash → bitcode bytes.
    pub styles: MapRef,
    /// Per-cell style references: same key as `cells` → pool hash.
    pub cell_styles: MapRef,
    /// Named-style definitions: name → bitcode of `(Style, StyleIncludes)`.
    pub named_styles: MapRef,
    /// Conditional-formatting rules: `<sid>!<ruleId>.p` → fractional position
    /// (priority order), `<sid>!<ruleId>.v` → bitcode of
    /// `(range, CfRule, Option<Dxf>)` with id-form range/formulas and the
    /// dxf content inlined (dxf ids are replica-local).
    pub cf: MapRef,
    /// Border-edge registers, one per grid line: `<sid>!v.<cid>:<rid>` is the
    /// line left of column `cid` at row `rid`; `<sid>!h.<cid>:<rid>` is the
    /// line on top of row `rid` at column `cid`. Value: session-encoded
    /// `BorderItem`. One identity per line dissolves the shared-edge
    /// conflict between adjacent cells' styles by construction.
    pub edges: MapRef,
    /// Cell hyperlinks: same key as `cells` → bitcode of `Link`. An
    /// independent LWW register per cell (like `cell_styles`), so a link
    /// survives a concurrent content edit of its cell; the engine removes
    /// the link when the content is cleared and that clear replicates as a
    /// register removal.
    pub links: MapRef,
    /// Merged cells, keyed by their anchor (top-left) cell — same key as
    /// `cells` — with the bottom-right corner ids as value (see
    /// [`merge_value`]). Anchor ids are stable under structural ops, so a
    /// merge follows its cells for free; two concurrent merges with the same
    /// anchor are plain LWW on the key, overlapping ones with different
    /// anchors converge in the document and are resolved by a deterministic
    /// render-time fixup (the loser is masked, not lost).
    pub merges: MapRef,
}

impl SchemaMaps {
    pub(crate) fn attach(doc: &Doc) -> SchemaMaps {
        SchemaMaps {
            meta: doc.get_or_insert_map(MAP_META),
            cells: doc.get_or_insert_map(MAP_CELLS),
            rows: doc.get_or_insert_map(MAP_ROWS),
            cols: doc.get_or_insert_map(MAP_COLS),
            keep_rows: doc.get_or_insert_map(MAP_KEEP_ROWS),
            keep_cols: doc.get_or_insert_map(MAP_KEEP_COLS),
            keep_sheets: doc.get_or_insert_map(MAP_KEEP_SHEETS),
            names: doc.get_or_insert_map(MAP_NAMES),
            styles: doc.get_or_insert_map(MAP_STYLES),
            cell_styles: doc.get_or_insert_map(MAP_CELL_STYLES),
            named_styles: doc.get_or_insert_map(MAP_NAMED_STYLES),
            cf: doc.get_or_insert_map(MAP_CF),
            edges: doc.get_or_insert_map(MAP_EDGES),
            links: doc.get_or_insert_map(MAP_LINKS),
            merges: doc.get_or_insert_map(MAP_MERGES),
        }
    }

    pub(crate) fn axis(&self, axis: Axis) -> (&MapRef, &MapRef) {
        match axis {
            Axis::Rows => (&self.rows, &self.keep_rows),
            Axis::Columns => (&self.cols, &self.keep_cols),
        }
    }
}

// Key builders.

pub(crate) fn sheet_meta_key(sheet: EntityId, field: &str) -> String {
    format!("s.{}.{}", sheet.encode(), field)
}

pub(crate) fn cell_key(sheet: EntityId, column: EntityId, row: EntityId) -> String {
    format!("{}!{}:{}", sheet.encode(), column.encode(), row.encode())
}

/// Value of a merge register: the bottom-right corner as `<colId>:<rowId>`.
pub(crate) fn merge_value(column: EntityId, row: EntityId) -> String {
    format!("{}:{}", column.encode(), row.encode())
}

pub(crate) fn parse_merge_value(value: &str) -> Option<(EntityId, EntityId)> {
    let (cid, rid) = value.split_once(':')?;
    Some((EntityId::decode(cid)?, EntityId::decode(rid)?))
}

pub(crate) fn axis_key(sheet: EntityId, id: EntityId, field: &str) -> String {
    format!("{}!{}.{}", sheet.encode(), id.encode(), field)
}

pub(crate) fn keep_prefix(sheet: EntityId, id: EntityId) -> String {
    format!("{}!{}/", sheet.encode(), id.encode())
}

pub(crate) fn keep_key(sheet: EntityId, id: EntityId, client: u64) -> String {
    format!("{}{:x}", keep_prefix(sheet, id), client)
}

pub(crate) fn sheet_keep_prefix(sheet: EntityId) -> String {
    format!("{}/", sheet.encode())
}

pub(crate) fn sheet_keep_key(sheet: EntityId, client: u64) -> String {
    format!("{}{:x}", sheet_keep_prefix(sheet), client)
}

/// Key of a border edge register: `axis` is `'v'` (line left of the column)
/// or `'h'` (line on top of the row).
pub(crate) fn edge_key(sheet: EntityId, axis: char, column: EntityId, row: EntityId) -> String {
    format!(
        "{}!{}.{}:{}",
        sheet.encode(),
        axis,
        column.encode(),
        row.encode()
    )
}

/// Key of a defined name: `<scope>|<name>` where scope is `g` (global) or a
/// sheet id (`|` cannot appear in a valid defined name).
pub(crate) fn name_key(scope: Option<EntityId>, name: &str) -> String {
    match scope {
        None => format!("g|{name}"),
        Some(sheet) => format!("{}|{}", sheet.encode(), name),
    }
}

/// Inverse of [`name_key`]; `Ok(None)` scope means global.
pub(crate) fn parse_name_key(key: &str) -> Option<(Option<EntityId>, &str)> {
    let (scope, name) = key.split_once('|')?;
    if scope == "g" {
        Some((None, name))
    } else {
        Some((Some(EntityId::decode(scope)?), name))
    }
}

// Value readers.

fn as_string(value: &Out) -> Option<String> {
    match value {
        Out::Any(Any::String(s)) => Some(s.to_string()),
        _ => None,
    }
}

fn as_f64(value: &Out) -> Option<f64> {
    match value {
        Out::Any(Any::Number(n)) => Some(*n),
        Out::Any(Any::BigInt(n)) => Some(*n as f64),
        _ => None,
    }
}

fn as_bool(value: &Out) -> Option<bool> {
    match value {
        Out::Any(Any::Bool(b)) => Some(*b),
        _ => None,
    }
}

fn as_i32(value: &Out) -> Option<i32> {
    match value {
        Out::Any(Any::BigInt(n)) => i32::try_from(*n).ok(),
        Out::Any(Any::Number(n)) => Some(*n as i32),
        _ => None,
    }
}

/// Materialized state of one conditional-formatting rule. The rule exists
/// while its body (`value`) is present; a pos-only remnant (concurrent
/// delete vs. reorder) is treated as deleted.
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct CfRuleProj {
    /// Fractional position: ascending position = ascending priority number
    /// (the highest position wins the evaluation, like the engine's highest
    /// priority number). Missing pos (concurrent delete vs. body update)
    /// sorts first, tie-broken by rule id.
    pub pos: Option<String>,
    /// bitcode of `(range, CfRule, Option<Dxf>)`.
    pub value: Option<Vec<u8>>,
}

/// Materialized state of one row or column.
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct AxisEntryProj {
    pub pos: Option<String>,
    /// Row height or column width.
    pub size: Option<f64>,
    pub hidden: bool,
    /// Row/column style: pool hash.
    pub style: Option<String>,
    /// Tombstone; the entity stays visible while its keep-set is non-empty.
    pub del: bool,
}

/// Snapshot of one sheet as described by the document.
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct SheetProj {
    pub name: String,
    pub pos: String,
    pub del: bool,
    pub frozen_rows: i32,
    pub frozen_columns: i32,
    /// Tab color, session-encoded (`r#RRGGBB` / `t<idx>;<tint>`); absent = none.
    pub color: Option<String>,
    /// Sheet state (`hidden` / `veryHidden`); absent = visible.
    pub state: Option<String>,
    /// Grid lines flag; absent = shown (the default).
    pub grid_lines: Option<bool>,
    pub rows: BTreeMap<EntityId, AxisEntryProj>,
    pub cols: BTreeMap<EntityId, AxisEntryProj>,
    /// Ids with at least one keep-set entry.
    pub keep_rows: HashSet<EntityId>,
    pub keep_cols: HashSet<EntityId>,
    /// `(column, row) → user input`. Includes masked cells of deleted
    /// rows/columns; visibility is decided by the axis orders.
    pub cells: BTreeMap<(EntityId, EntityId), String>,
    /// `(column, row) → style pool hash` (independent LWW register per cell,
    /// so concurrent content and style edits of the same cell both survive).
    pub cell_styles: BTreeMap<(EntityId, EntityId), String>,
    /// Conditional-formatting rules by stable rule id.
    pub cf: BTreeMap<EntityId, CfRuleProj>,
    /// Vertical border edges: `(col_id, row_id) → encoded BorderItem`, the
    /// line **left of** `col_id` at `row_id`.
    pub v_edges: BTreeMap<(EntityId, EntityId), String>,
    /// Horizontal border edges: `(col_id, row_id) → encoded BorderItem`, the
    /// line **on top of** `row_id` at `col_id`.
    pub h_edges: BTreeMap<(EntityId, EntityId), String>,
    /// Cell hyperlinks: `(column, row) → bitcode of Link`.
    pub links: BTreeMap<(EntityId, EntityId), Vec<u8>>,
    /// Merged cells: anchor `(column, row)` → bottom-right `(column, row)`.
    pub merges: BTreeMap<(EntityId, EntityId), (EntityId, EntityId)>,
}

impl SheetProj {
    /// Live CF rules in canonical order: sorted by `(pos, id)`; entries
    /// without a body are remnants of a delete and are skipped.
    pub(crate) fn cf_canonical(&self) -> Vec<(EntityId, &CfRuleProj)> {
        let mut rules: Vec<(EntityId, &CfRuleProj)> = self
            .cf
            .iter()
            .filter(|(_, e)| e.value.is_some())
            .map(|(id, e)| (*id, e))
            .collect();
        rules.sort_by(|a, b| {
            (a.1.pos.as_deref().unwrap_or(""), a.0).cmp(&(b.1.pos.as_deref().unwrap_or(""), b.0))
        });
        rules
    }

    pub(crate) fn axis_order(&self, axis: Axis) -> AxisOrder {
        let (entries, keeps, max) = match axis {
            Axis::Rows => (&self.rows, &self.keep_rows, MAX_ROW),
            Axis::Columns => (&self.cols, &self.keep_cols, MAX_COLUMN),
        };
        AxisOrder::new(
            max,
            entries.iter().map(|(id, e)| {
                let visible = !e.del || keeps.contains(id);
                (*id, e.pos.clone(), visible)
            }),
        )
    }
}

/// A plain snapshot of the whole document.
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct Projection {
    pub sheets: BTreeMap<EntityId, SheetProj>,
    /// Sheet ids with at least one keep-set entry (update-wins for sheets).
    pub keep_sheets: HashSet<EntityId>,
    /// Defined names: [`name_key`] → formula (id-form or plain text).
    pub names: BTreeMap<String, String>,
    /// Workbook-level LWW registers.
    pub name: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    /// Bitcode of `Theme`.
    pub theme: Option<Vec<u8>>,
    /// Content-addressed style pool: hash → bitcode of `Style`.
    pub styles: BTreeMap<String, Vec<u8>>,
    /// Named styles: name → bitcode of `(Style, StyleIncludes)`.
    pub named_styles: BTreeMap<String, Vec<u8>>,
}

impl Projection {
    /// Visible sheets in display order: `(id, proj)` sorted by `(pos, id)`.
    /// A sheet is visible iff it has no tombstone OR its keep-set is
    /// non-empty (update-wins). Deterministic fixup: a workbook cannot have
    /// zero sheets, so if concurrent deletions tombstoned everything, the
    /// sheet with the smallest `(pos, id)` stays visible on every replica.
    pub(crate) fn visible_sheets(&self) -> Vec<(EntityId, &SheetProj)> {
        let mut sheets: Vec<(EntityId, &SheetProj)> = self
            .sheets
            .iter()
            .filter(|(id, s)| !s.del || self.keep_sheets.contains(id))
            .map(|(id, s)| (*id, s))
            .collect();
        sheets.sort_by(|a, b| (a.1.pos.as_str(), a.0).cmp(&(b.1.pos.as_str(), b.0)));
        if sheets.is_empty() {
            let mut all: Vec<(EntityId, &SheetProj)> =
                self.sheets.iter().map(|(id, s)| (*id, s)).collect();
            all.sort_by(|a, b| (a.1.pos.as_str(), a.0).cmp(&(b.1.pos.as_str(), b.0)));
            sheets.extend(all.into_iter().take(1));
        }
        sheets
    }

    /// Builds the projection by replaying every entry of every root map
    /// through the same per-entry patch functions the incremental path uses
    /// ([`Projection::patch`]), so the two can never disagree.
    pub(crate) fn from_doc(doc: &Doc, maps: &SchemaMaps) -> Projection {
        let txn = doc.transact();
        let mut proj = Projection::default();
        for (kind, map) in maps.all() {
            for (key, value) in map.iter(&txn) {
                proj.patch(kind, key, Some(value), None);
            }
        }
        proj
    }

    /// Applies the current document value of one root-map entry (`None` =
    /// the key is absent) to the projection. When `delta` is given, the old
    /// value is recorded there whenever the projection actually changed.
    ///
    /// Invariant: after patching every changed key of a transaction, the
    /// projection equals [`Projection::from_doc`] on the resulting document.
    /// (`refresh_shadow` checks this in tests.)
    pub(crate) fn patch(
        &mut self,
        kind: MapKind,
        key: &str,
        value: Option<Out>,
        delta: Option<&mut Delta>,
    ) {
        match kind {
            MapKind::Meta => self.patch_meta(key, value, delta),
            MapKind::KeepSheets => {
                let Some((sid, _client)) = key.split_once('/') else {
                    return;
                };
                let Some(sheet_id) = EntityId::decode(sid) else {
                    return;
                };
                let present = value.is_some();
                let changed = if present {
                    self.keep_sheets.insert(sheet_id)
                } else {
                    self.keep_sheets.remove(&sheet_id)
                };
                if changed {
                    if let Some(delta) = delta {
                        delta.keep_sheets = true;
                    }
                }
            }
            MapKind::Cells => {
                let Some((sheet_id, cell)) = parse_cell_key(key) else {
                    return;
                };
                let new = value.as_ref().and_then(as_string);
                let old = match &new {
                    Some(input) => self
                        .sheets
                        .entry(sheet_id)
                        .or_default()
                        .cells
                        .insert(cell, input.clone()),
                    None => self
                        .sheets
                        .get_mut(&sheet_id)
                        .and_then(|s| s.cells.remove(&cell)),
                };
                if old != new {
                    if let Some(delta) = delta {
                        delta.sheet(sheet_id).cells.push((cell, old));
                    }
                }
                if new.is_none() {
                    self.prune_sheet(sheet_id);
                }
            }
            MapKind::Rows | MapKind::Cols => {
                let is_rows = kind == MapKind::Rows;
                let Some((sid, rest)) = key.split_once('!') else {
                    return;
                };
                let Some((id, field)) = rest.rsplit_once('.') else {
                    return;
                };
                let (Some(sheet_id), Some(entity_id)) =
                    (EntityId::decode(sid), EntityId::decode(id))
                else {
                    return;
                };
                if !matches!(field, "p" | "h" | "x" | "d" | "sty") {
                    return;
                }
                let sheet = self.sheets.entry(sheet_id).or_default();
                let entries = if is_rows {
                    &mut sheet.rows
                } else {
                    &mut sheet.cols
                };
                let old = entries.get(&entity_id).cloned();
                let entry = entries.entry(entity_id).or_default();
                match field {
                    "p" => entry.pos = value.as_ref().and_then(as_string),
                    "h" => entry.size = value.as_ref().and_then(as_f64),
                    "x" => entry.hidden = value.as_ref().and_then(as_bool).unwrap_or(false),
                    "d" => entry.del = value.as_ref().and_then(as_bool).unwrap_or(false),
                    _ => entry.style = value.as_ref().and_then(as_string),
                }
                if *entry == AxisEntryProj::default() {
                    entries.remove(&entity_id);
                }
                let new = entries.get(&entity_id).cloned();
                if old != new {
                    if let Some(delta) = delta {
                        let sd = delta.sheet(sheet_id);
                        sd.axis = true;
                        let axis_delta = if is_rows { &mut sd.rows } else { &mut sd.cols };
                        axis_delta.entry(entity_id).or_insert(old);
                    }
                }
                if new.is_none() {
                    self.prune_sheet(sheet_id);
                }
            }
            MapKind::Names => {
                let new = value.as_ref().and_then(as_string);
                let old = match &new {
                    Some(formula) => self.names.insert(key.to_string(), formula.clone()),
                    None => self.names.remove(key),
                };
                if old != new {
                    if let Some(delta) = delta {
                        delta.names = true;
                    }
                }
            }
            MapKind::CellStyles => {
                let Some((sheet_id, cell)) = parse_cell_key(key) else {
                    return;
                };
                let new = value.as_ref().and_then(as_string);
                let old = match &new {
                    Some(hash) => self
                        .sheets
                        .entry(sheet_id)
                        .or_default()
                        .cell_styles
                        .insert(cell, hash.clone()),
                    None => self
                        .sheets
                        .get_mut(&sheet_id)
                        .and_then(|s| s.cell_styles.remove(&cell)),
                };
                if old != new {
                    if let Some(delta) = delta {
                        delta.sheet(sheet_id).cell_styles.entry(cell).or_insert(old);
                    }
                }
                if new.is_none() {
                    self.prune_sheet(sheet_id);
                }
            }
            MapKind::Styles => {
                let new = value.as_ref().and_then(as_buffer);
                let old = match &new {
                    Some(bytes) => self.styles.insert(key.to_string(), bytes.clone()),
                    None => self.styles.remove(key),
                };
                if old != new {
                    if let Some(delta) = delta {
                        delta.styles = true;
                    }
                }
            }
            MapKind::Merges => {
                let Some((sheet_id, cell)) = parse_cell_key(key) else {
                    return;
                };
                let new = value
                    .as_ref()
                    .and_then(as_string)
                    .as_deref()
                    .and_then(parse_merge_value);
                let old = match new {
                    Some(corner) => self
                        .sheets
                        .entry(sheet_id)
                        .or_default()
                        .merges
                        .insert(cell, corner),
                    None => self
                        .sheets
                        .get_mut(&sheet_id)
                        .and_then(|s| s.merges.remove(&cell)),
                };
                if old != new {
                    if let Some(delta) = delta {
                        delta.sheet(sheet_id).merges = true;
                    }
                }
                if new.is_none() {
                    self.prune_sheet(sheet_id);
                }
            }
            MapKind::Links => {
                let Some((sheet_id, cell)) = parse_cell_key(key) else {
                    return;
                };
                let new = value.as_ref().and_then(as_buffer);
                let old = match &new {
                    Some(bytes) => self
                        .sheets
                        .entry(sheet_id)
                        .or_default()
                        .links
                        .insert(cell, bytes.clone()),
                    None => self
                        .sheets
                        .get_mut(&sheet_id)
                        .and_then(|s| s.links.remove(&cell)),
                };
                if old != new {
                    if let Some(delta) = delta {
                        delta.sheet(sheet_id).links.entry(cell).or_insert(old);
                    }
                }
                if new.is_none() {
                    self.prune_sheet(sheet_id);
                }
            }
            MapKind::NamedStyles => {
                let new = value.as_ref().and_then(as_buffer);
                let old = match &new {
                    Some(bytes) => self.named_styles.insert(key.to_string(), bytes.clone()),
                    None => self.named_styles.remove(key),
                };
                if old != new {
                    if let Some(delta) = delta {
                        delta.named_styles = true;
                    }
                }
            }
            MapKind::Edges => {
                let Some((sid, rest)) = key.split_once('!') else {
                    return;
                };
                let Some((axis, ids)) = rest.split_once('.') else {
                    return;
                };
                let Some((cid, rid)) = ids.split_once(':') else {
                    return;
                };
                let (Some(sheet_id), Some(col_id), Some(row_id)) = (
                    EntityId::decode(sid),
                    EntityId::decode(cid),
                    EntityId::decode(rid),
                ) else {
                    return;
                };
                let vertical = match axis {
                    "v" => true,
                    "h" => false,
                    _ => return,
                };
                let cell = (col_id, row_id);
                let new = value.as_ref().and_then(as_string);
                let old = match &new {
                    Some(item) => {
                        let sheet = self.sheets.entry(sheet_id).or_default();
                        let edges = if vertical {
                            &mut sheet.v_edges
                        } else {
                            &mut sheet.h_edges
                        };
                        edges.insert(cell, item.clone())
                    }
                    None => self.sheets.get_mut(&sheet_id).and_then(|sheet| {
                        if vertical {
                            sheet.v_edges.remove(&cell)
                        } else {
                            sheet.h_edges.remove(&cell)
                        }
                    }),
                };
                if old != new {
                    if let Some(delta) = delta {
                        let sd = delta.sheet(sheet_id);
                        let edges = if vertical {
                            &mut sd.v_edges
                        } else {
                            &mut sd.h_edges
                        };
                        edges.entry(cell).or_insert(old);
                    }
                }
                if new.is_none() {
                    self.prune_sheet(sheet_id);
                }
            }
            MapKind::Cf => {
                let Some((sid, rest)) = key.split_once('!') else {
                    return;
                };
                let Some((rid, field)) = rest.rsplit_once('.') else {
                    return;
                };
                let (Some(sheet_id), Some(rule_id)) =
                    (EntityId::decode(sid), EntityId::decode(rid))
                else {
                    return;
                };
                if !matches!(field, "p" | "v") {
                    return;
                }
                let sheet = self.sheets.entry(sheet_id).or_default();
                let old = sheet.cf.get(&rule_id).cloned();
                let entry = sheet.cf.entry(rule_id).or_default();
                match field {
                    "p" => entry.pos = value.as_ref().and_then(as_string),
                    _ => entry.value = value.as_ref().and_then(as_buffer),
                }
                if *entry == CfRuleProj::default() {
                    sheet.cf.remove(&rule_id);
                }
                let new = sheet.cf.get(&rule_id).cloned();
                if old != new {
                    if let Some(delta) = delta {
                        delta.sheet(sheet_id).cf = true;
                    }
                }
                if new.is_none() {
                    self.prune_sheet(sheet_id);
                }
            }
            MapKind::KeepRows | MapKind::KeepCols => {
                // Membership is "some client's key exists": a removal only
                // drops the id when no other key with the same prefix is
                // left, which the caller checks (`KeepMembership`).
                let is_rows = kind == MapKind::KeepRows;
                let Some((sid, rest)) = key.split_once('!') else {
                    return;
                };
                let Some((id, _client)) = rest.split_once('/') else {
                    return;
                };
                let (Some(sheet_id), Some(entity_id)) =
                    (EntityId::decode(sid), EntityId::decode(id))
                else {
                    return;
                };
                let present = value.is_some();
                let changed = {
                    let sheet = self.sheets.entry(sheet_id).or_default();
                    let keeps = if is_rows {
                        &mut sheet.keep_rows
                    } else {
                        &mut sheet.keep_cols
                    };
                    if present {
                        keeps.insert(entity_id)
                    } else {
                        keeps.remove(&entity_id)
                    }
                };
                if changed {
                    if let Some(delta) = delta {
                        delta.sheet(sheet_id).axis = true;
                    }
                }
                if !present {
                    self.prune_sheet(sheet_id);
                }
            }
        }
    }

    /// Drops a sheet entry that no map entry references any more, so the
    /// incremental projection never holds a sheet `from_doc` would not.
    fn prune_sheet(&mut self, sheet_id: EntityId) {
        if self
            .sheets
            .get(&sheet_id)
            .is_some_and(|sheet| *sheet == SheetProj::default())
        {
            self.sheets.remove(&sheet_id);
        }
    }

    fn patch_meta(&mut self, key: &str, value: Option<Out>, delta: Option<&mut Delta>) {
        if let Some(field) = key.strip_prefix("wb.") {
            let changed = match field {
                "name" => {
                    let new = value.as_ref().and_then(as_string);
                    let changed = self.name != new;
                    self.name = new;
                    changed
                }
                "locale" => {
                    let new = value.as_ref().and_then(as_string);
                    let changed = self.locale != new;
                    self.locale = new;
                    changed
                }
                "tz" => {
                    let new = value.as_ref().and_then(as_string);
                    let changed = self.timezone != new;
                    self.timezone = new;
                    changed
                }
                "theme" => {
                    let new = value.as_ref().and_then(as_buffer);
                    let changed = self.theme != new;
                    self.theme = new;
                    changed
                }
                _ => false,
            };
            if changed {
                if let Some(delta) = delta {
                    delta.workbook = true;
                }
            }
            return;
        }
        let Some(rest) = key.strip_prefix("s.") else {
            return;
        };
        let Some((sid, field)) = rest.split_once('.') else {
            return;
        };
        let Some(sheet_id) = EntityId::decode(sid) else {
            return;
        };
        if !matches!(
            field,
            "name" | "pos" | "del" | "fr" | "fc" | "color" | "state" | "grid"
        ) {
            return;
        }
        if value.is_none() && !self.sheets.contains_key(&sheet_id) {
            return;
        }
        let sheet = self.sheets.entry(sheet_id).or_default();
        let changed = match field {
            "name" => {
                let new = value.as_ref().and_then(as_string).unwrap_or_default();
                let changed = sheet.name != new;
                sheet.name = new;
                changed
            }
            "pos" => {
                let new = value.as_ref().and_then(as_string).unwrap_or_default();
                let changed = sheet.pos != new;
                sheet.pos = new;
                changed
            }
            "del" => {
                let new = value.as_ref().and_then(as_bool).unwrap_or(false);
                let changed = sheet.del != new;
                sheet.del = new;
                changed
            }
            "fr" => {
                let new = value.as_ref().and_then(as_i32).unwrap_or(0);
                let changed = sheet.frozen_rows != new;
                sheet.frozen_rows = new;
                changed
            }
            "fc" => {
                let new = value.as_ref().and_then(as_i32).unwrap_or(0);
                let changed = sheet.frozen_columns != new;
                sheet.frozen_columns = new;
                changed
            }
            "color" => {
                let new = value.as_ref().and_then(as_string);
                let changed = sheet.color != new;
                sheet.color = new;
                changed
            }
            "state" => {
                let new = value.as_ref().and_then(as_string);
                let changed = sheet.state != new;
                sheet.state = new;
                changed
            }
            _ => {
                let new = value.as_ref().and_then(as_bool);
                let changed = sheet.grid_lines != new;
                sheet.grid_lines = new;
                changed
            }
        };
        if changed {
            if let Some(delta) = delta {
                delta.sheet(sheet_id).meta = true;
            }
        }
        if value.is_none() {
            self.prune_sheet(sheet_id);
        }
    }
}

/// `<sid>!<cid>:<rid>` → `(sheet, (column, row))`.
fn parse_cell_key(key: &str) -> Option<(EntityId, (EntityId, EntityId))> {
    let (sid, rest) = key.split_once('!')?;
    let (cid, rid) = rest.split_once(':')?;
    Some((
        EntityId::decode(sid)?,
        (EntityId::decode(cid)?, EntityId::decode(rid)?),
    ))
}

fn as_buffer(value: &Out) -> Option<Vec<u8>> {
    match value {
        Out::Any(Any::Buffer(bytes)) => Some(bytes.to_vec()),
        _ => None,
    }
}

/// Identifies one of the root maps (see [`SchemaMaps`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum MapKind {
    Meta,
    Cells,
    Rows,
    Cols,
    KeepRows,
    KeepCols,
    KeepSheets,
    Names,
    Styles,
    CellStyles,
    NamedStyles,
    Cf,
    Edges,
    Links,
    Merges,
}

impl SchemaMaps {
    /// Every root map with its kind.
    pub(crate) fn all(&self) -> [(MapKind, &MapRef); 15] {
        [
            (MapKind::Meta, &self.meta),
            (MapKind::Cells, &self.cells),
            (MapKind::Rows, &self.rows),
            (MapKind::Cols, &self.cols),
            (MapKind::KeepRows, &self.keep_rows),
            (MapKind::KeepCols, &self.keep_cols),
            (MapKind::KeepSheets, &self.keep_sheets),
            (MapKind::Names, &self.names),
            (MapKind::Styles, &self.styles),
            (MapKind::CellStyles, &self.cell_styles),
            (MapKind::NamedStyles, &self.named_styles),
            (MapKind::Cf, &self.cf),
            (MapKind::Edges, &self.edges),
            (MapKind::Links, &self.links),
            (MapKind::Merges, &self.merges),
        ]
    }

    pub(crate) fn map(&self, kind: MapKind) -> &MapRef {
        match kind {
            MapKind::Meta => &self.meta,
            MapKind::Cells => &self.cells,
            MapKind::Rows => &self.rows,
            MapKind::Cols => &self.cols,
            MapKind::KeepRows => &self.keep_rows,
            MapKind::KeepCols => &self.keep_cols,
            MapKind::KeepSheets => &self.keep_sheets,
            MapKind::Names => &self.names,
            MapKind::Styles => &self.styles,
            MapKind::CellStyles => &self.cell_styles,
            MapKind::NamedStyles => &self.named_styles,
            MapKind::Cf => &self.cf,
            MapKind::Edges => &self.edges,
            MapKind::Links => &self.links,
            MapKind::Merges => &self.merges,
        }
    }
}

/// What changed in one sheet between two projections: the keys whose value
/// differs, each with its **old** value (the new one is in the projection).
#[derive(Debug, Default)]
pub(crate) struct SheetDelta {
    /// Changed cells with their old value, in patch order. A plain list: on
    /// a join this holds every cell and a sorted map cost as much as the
    /// document apply itself; consumers only iterate it.
    pub cells: Vec<((EntityId, EntityId), Option<String>)>,
    pub cell_styles: BTreeMap<(EntityId, EntityId), Option<String>>,
    pub links: BTreeMap<(EntityId, EntityId), Option<Vec<u8>>>,
    pub v_edges: BTreeMap<(EntityId, EntityId), Option<String>>,
    pub h_edges: BTreeMap<(EntityId, EntityId), Option<String>>,
    pub rows: BTreeMap<EntityId, Option<AxisEntryProj>>,
    pub cols: BTreeMap<EntityId, Option<AxisEntryProj>>,
    /// Some CF register changed.
    pub cf: bool,
    /// Some merge register changed.
    pub merges: bool,
    /// Some sheet-level meta register changed (name, pos, del, …).
    pub meta: bool,
    /// Some row/column entry or keep-set membership changed, so the axis
    /// orders may differ.
    pub axis: bool,
    /// The row or column order changed (set by the session after patching).
    pub structural: bool,
    /// The whole old sheet, captured only when `structural` (the rebuild
    /// clears every old location).
    pub old: Option<Box<SheetProj>>,
}

/// What changed in the projection since the last refresh.
#[derive(Debug, Default)]
pub(crate) struct Delta {
    pub sheets: BTreeMap<EntityId, SheetDelta>,
    pub names: bool,
    pub named_styles: bool,
    /// Workbook-level registers (name, locale, timezone, theme).
    pub workbook: bool,
    pub keep_sheets: bool,
    pub styles: bool,
    /// Visible sheet ids, in order, before the refresh.
    pub old_visible: Vec<EntityId>,
}

impl Delta {
    pub(crate) fn sheet(&mut self, id: EntityId) -> &mut SheetDelta {
        self.sheets.entry(id).or_default()
    }

    /// True when no value differs (the style pool is content-addressed and
    /// only ever grows, so it never changes what the model shows on its own).
    pub(crate) fn is_empty(&self) -> bool {
        !self.names
            && !self.named_styles
            && !self.workbook
            && !self.keep_sheets
            && self.sheets.values().all(|sd| {
                sd.cells.is_empty()
                    && sd.cell_styles.is_empty()
                    && sd.links.is_empty()
                    && sd.v_edges.is_empty()
                    && sd.h_edges.is_empty()
                    && sd.rows.is_empty()
                    && sd.cols.is_empty()
                    && !sd.cf
                    && !sd.merges
                    && !sd.meta
                    && !sd.axis
            })
    }
}
