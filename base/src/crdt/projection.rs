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

    pub(crate) fn from_doc(doc: &Doc, maps: &SchemaMaps) -> Projection {
        let txn = doc.transact();
        let mut proj = Projection::default();

        for (key, value) in maps.meta.iter(&txn) {
            if let Some(field) = key.strip_prefix("wb.") {
                match field {
                    "name" => proj.name = as_string(&value),
                    "locale" => proj.locale = as_string(&value),
                    "tz" => proj.timezone = as_string(&value),
                    "theme" => {
                        if let Out::Any(Any::Buffer(bytes)) = &value {
                            proj.theme = Some(bytes.to_vec());
                        }
                    }
                    _ => {}
                }
                continue;
            }
            let Some(rest) = key.strip_prefix("s.") else {
                continue;
            };
            let Some((sid, field)) = rest.split_once('.') else {
                continue;
            };
            let Some(sheet_id) = EntityId::decode(sid) else {
                continue;
            };
            let sheet = proj.sheets.entry(sheet_id).or_default();
            match field {
                "name" => {
                    if let Some(name) = as_string(&value) {
                        sheet.name = name;
                    }
                }
                "pos" => {
                    if let Some(pos) = as_string(&value) {
                        sheet.pos = pos;
                    }
                }
                "del" => sheet.del = as_bool(&value).unwrap_or(false),
                "fr" => sheet.frozen_rows = as_i32(&value).unwrap_or(0),
                "fc" => sheet.frozen_columns = as_i32(&value).unwrap_or(0),
                "color" => sheet.color = as_string(&value),
                "state" => sheet.state = as_string(&value),
                "grid" => sheet.grid_lines = as_bool(&value),
                _ => {}
            }
        }

        for (key, _) in maps.keep_sheets.iter(&txn) {
            let Some((sid, _client)) = key.split_once('/') else {
                continue;
            };
            if let Some(sheet_id) = EntityId::decode(sid) {
                proj.keep_sheets.insert(sheet_id);
            }
        }

        for (key, value) in maps.cells.iter(&txn) {
            let Some((sid, rest)) = key.split_once('!') else {
                continue;
            };
            let Some((cid, rid)) = rest.split_once(':') else {
                continue;
            };
            let (Some(sheet_id), Some(col_id), Some(row_id)) = (
                EntityId::decode(sid),
                EntityId::decode(cid),
                EntityId::decode(rid),
            ) else {
                continue;
            };
            if let Some(input) = as_string(&value) {
                proj.sheets
                    .entry(sheet_id)
                    .or_default()
                    .cells
                    .insert((col_id, row_id), input);
            }
        }

        for (axis_map, is_rows) in [(&maps.rows, true), (&maps.cols, false)] {
            for (key, value) in axis_map.iter(&txn) {
                let Some((sid, rest)) = key.split_once('!') else {
                    continue;
                };
                let Some((id, field)) = rest.rsplit_once('.') else {
                    continue;
                };
                let (Some(sheet_id), Some(entity_id)) =
                    (EntityId::decode(sid), EntityId::decode(id))
                else {
                    continue;
                };
                let sheet = proj.sheets.entry(sheet_id).or_default();
                let entries = if is_rows {
                    &mut sheet.rows
                } else {
                    &mut sheet.cols
                };
                let entry = entries.entry(entity_id).or_default();
                match field {
                    "p" => entry.pos = as_string(&value),
                    "h" => entry.size = as_f64(&value),
                    "x" => entry.hidden = as_bool(&value).unwrap_or(false),
                    "d" => entry.del = as_bool(&value).unwrap_or(false),
                    "sty" => entry.style = as_string(&value),
                    _ => {}
                }
            }
        }

        for (key, value) in maps.names.iter(&txn) {
            if let Some(formula) = as_string(&value) {
                proj.names.insert(key.to_string(), formula);
            }
        }

        for (key, value) in maps.cell_styles.iter(&txn) {
            let Some((sid, rest)) = key.split_once('!') else {
                continue;
            };
            let Some((cid, rid)) = rest.split_once(':') else {
                continue;
            };
            let (Some(sheet_id), Some(col_id), Some(row_id)) = (
                EntityId::decode(sid),
                EntityId::decode(cid),
                EntityId::decode(rid),
            ) else {
                continue;
            };
            if let Some(hash) = as_string(&value) {
                proj.sheets
                    .entry(sheet_id)
                    .or_default()
                    .cell_styles
                    .insert((col_id, row_id), hash);
            }
        }

        for (key, value) in maps.styles.iter(&txn) {
            if let Out::Any(Any::Buffer(bytes)) = value {
                proj.styles.insert(key.to_string(), bytes.to_vec());
            }
        }

        for (key, value) in maps.named_styles.iter(&txn) {
            if let Out::Any(Any::Buffer(bytes)) = value {
                proj.named_styles.insert(key.to_string(), bytes.to_vec());
            }
        }

        for (key, value) in maps.edges.iter(&txn) {
            let Some((sid, rest)) = key.split_once('!') else {
                continue;
            };
            let Some((axis, ids)) = rest.split_once('.') else {
                continue;
            };
            let Some((cid, rid)) = ids.split_once(':') else {
                continue;
            };
            let (Some(sheet_id), Some(col_id), Some(row_id)) = (
                EntityId::decode(sid),
                EntityId::decode(cid),
                EntityId::decode(rid),
            ) else {
                continue;
            };
            let Some(item) = as_string(&value) else {
                continue;
            };
            let sheet = proj.sheets.entry(sheet_id).or_default();
            match axis {
                "v" => {
                    sheet.v_edges.insert((col_id, row_id), item);
                }
                "h" => {
                    sheet.h_edges.insert((col_id, row_id), item);
                }
                _ => {}
            }
        }

        for (key, value) in maps.cf.iter(&txn) {
            let Some((sid, rest)) = key.split_once('!') else {
                continue;
            };
            let Some((rid, field)) = rest.rsplit_once('.') else {
                continue;
            };
            let (Some(sheet_id), Some(rule_id)) =
                (EntityId::decode(sid), EntityId::decode(rid))
            else {
                continue;
            };
            let entry = proj
                .sheets
                .entry(sheet_id)
                .or_default()
                .cf
                .entry(rule_id)
                .or_default();
            match field {
                "p" => entry.pos = as_string(&value),
                "v" => {
                    if let Out::Any(Any::Buffer(bytes)) = value {
                        entry.value = Some(bytes.to_vec());
                    }
                }
                _ => {}
            }
        }

        for (keep_map, is_rows) in [(&maps.keep_rows, true), (&maps.keep_cols, false)] {
            for (key, _) in keep_map.iter(&txn) {
                let Some((sid, rest)) = key.split_once('!') else {
                    continue;
                };
                let Some((id, _client)) = rest.split_once('/') else {
                    continue;
                };
                let (Some(sheet_id), Some(entity_id)) =
                    (EntityId::decode(sid), EntityId::decode(id))
                else {
                    continue;
                };
                let sheet = proj.sheets.entry(sheet_id).or_default();
                if is_rows {
                    sheet.keep_rows.insert(entity_id);
                } else {
                    sheet.keep_cols.insert(entity_id);
                }
            }
        }

        proj
    }
}
