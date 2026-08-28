//! Replicated operations.
//!
//! A [`Patch`] is an assignment `register ← value`. The register is addressed by a scope (workbook,
//! sheet, row, column, cell, defined name, ...) together with a *property discriminant*, and removal
//! is expressed as `None` rather than a dedicated operation.
//!
//! The property discriminant being part of the register address is what makes concurrent edits to
//! different properties of the same object survive: renaming a sheet and recolouring it are writes
//! to two different registers, so neither loses. Collapsing a property sub-enum into a single
//! register would silently change that.
//!
//! Column spans are the one register address that is not a single object. A sequential edit shatters
//! any wide span it partly overlaps eagerly — the wide register is removed and the narrower ones
//! written in the same commit — so locally spans never overlap. Only concurrency can make them, and
//! overlapping spans are then resolved per position by register write timestamp: LWW, newest
//! covering span wins. v1 emitters never shatter: a column property write is a point write to the
//! single-column span `(k, k)`, and the read-time resolution handles narrow-over-wide. Open-ended
//! spans are spelled two
//! ways on purpose: `Option` axes in the generic [`RangeRef`](crate::types::RangeRef) world, where
//! [`Ordinal`](crate::types::Ordinal) has no sentinel to spare, and [`FractionalKey::NULL`]
//! brackets in this FractionalKey-native patch and storage layer.
//!
//! Patches must be **index-free**: they may not carry values whose meaning depends on a replica's
//! local tables. Style indices into `Styles.cell_xfs`, shared-string indices and shared-formula
//! indices are all assigned by local insertion order, so two replicas can mint the same index for
//! different content. Patches therefore carry resolved values ([`Style`], the formula text, ...) and
//! each replica interns them locally on apply. Rows, columns and sheets are addressed by
//! [`FractionalKey`] instead, which every replica agrees on by construction.
//!
//! Every `prev` field is undo data. It is `#[bitcode(skip)]`, so it is populated only on locally
//! generated patches and comes back defaulted on anything received from a peer.
//!
//! # Wire format
//!
//! [`encode_patches`] writes a [`PATCH_FORMAT_VERSION`] byte, then the bitcode payload. bitcode
//! encodes an enum variant as its *index*, so [`Patch`] variants and the property enums below may
//! only ever be **appended** — reordering or removing one silently reinterprets older payloads.
//!
//! Caveat: a malformed [`FractionalKey`] or
//! [`FractionalIndex`](super::fractional_index::FractionalIndex) payload panics inside the bitcode
//! decoder instead of erroring, so untrusted bytes must not reach [`decode_patches`] yet.

use crate::cf_types::CfRule;
use crate::collab::fractional_index::FractionalKey;
use crate::collab::log::Timestamp;
use crate::collab::model::{Stable, StableCellAddress, StableRange};
use crate::collab::DynError;
use crate::constants::{DEFAULT_COLUMN_WIDTH, DEFAULT_ROW_HEIGHT};
use crate::expressions::token::Error;
use crate::types::{ArrayKind, Color, Comment, SheetState, Style, Theme};
use crate::{COLUMN_WIDTH_FACTOR, ROW_HEIGHT_FACTOR};
use bitcode::{Decode, Encode};

/// Identifies a sheet. Minted by hashing the creating replica's session, so two peers adding a
/// sheet concurrently do not collide.
pub type SheetId = u32;

/// Identifies a named style. Hashed from the name it was created under, so two peers creating the
/// same style concurrently write one register rather than two.
pub type NamedStyleId = u64;

/// Identifies a defined name. Hashed from the scope and case-folded name it was created under, so
/// two peers creating the same name concurrently write one register rather than two.
pub type DefinedNameId = u64;

/// Version byte prefixing every [`encode_patches`] payload.
pub const PATCH_FORMAT_VERSION: u8 = 1;

/// Encodes a commit's worth of patches: the version byte, then the bitcode payload.
pub fn encode_patches(patches: &[Patch]) -> Result<Vec<u8>, DynError> {
    let mut out = Vec::new();
    out.push(PATCH_FORMAT_VERSION);
    out.extend_from_slice(&bitcode::encode(patches));
    Ok(out)
}

/// Reads back what [`encode_patches`] wrote, rejecting a payload this build cannot interpret.
pub fn decode_patches(bytes: &[u8]) -> Result<Vec<Patch>, DynError> {
    let (&version, payload) = bytes.split_first().ok_or("empty patch payload")?;
    if version != PATCH_FORMAT_VERSION {
        return Err(format!("unsupported patch format version: {version}").into());
    }
    Ok(bitcode::decode(payload)?)
}

/// The patches that undo `patches`, in reverse order. Undo is a new commit — see
/// [`log`](crate::collab::log) — so an inverse wins by being later, never by rewriting history.
///
/// It is built from the `prev` fields, which are local-only, so this is meaningful only on patches
/// this replica authored. What inverts: every `Set*` whose `prev` is populated, an insert into the
/// matching delete, and a delete into the matching insert followed by restores of the snapshot's
/// state and cells. What does not, and is dropped:
///
/// - `SetArrayValue`, whose `prev` covers a rectangle rather than the anchor it would write to,
/// - `DeleteSheet` with no [`SheetRestore`] captured — a user-initiated delete still takes no
///   snapshot, so only the one an `AddSheet` inverted into puts its sheet back,
/// - `MoveConditionalFormats`, which is a no-op to begin with,
/// - anything whose `prev` came off the wire, where it decodes as the default.
///
/// Content restores replay at the stamp they were captured with, via the `ts` field, so a
/// concurrent newer edit to a restored cell survives the undo.
pub fn invert_patches(patches: &[Patch]) -> Vec<Patch> {
    let mut out = Vec::new();
    for patch in patches.iter().rev() {
        match patch {
            Patch::SetCellValue {
                sheet,
                at,
                value,
                prev,
                ..
            } => out.push(Patch::SetCellValue {
                sheet: *sheet,
                at: at.clone(),
                value: (**prev).clone(),
                ts: None,
                prev: Box::new(value.clone()),
            }),
            Patch::SetCellStyle {
                sheet,
                at,
                style,
                prev,
                ..
            } => out.push(Patch::SetCellStyle {
                sheet: *sheet,
                at: at.clone(),
                style: (**prev).clone().map(Box::new),
                ts: None,
                prev: Box::new(style.as_ref().map(|s| (**s).clone())),
            }),
            Patch::InsertRows { sheet, keys } => out.push(Patch::DeleteRows {
                sheet: *sheet,
                keys: keys.clone(),
                prev: keys
                    .iter()
                    .map(|k| RowSnapshot {
                        key: k.clone(),
                        state: RowState::default(),
                        cell_values: Vec::new(),
                        cell_styles: Vec::new(),
                        prop_ts: Vec::new(),
                    })
                    .collect(),
            }),
            Patch::InsertColumns { sheet, keys } => out.push(Patch::DeleteColumns {
                sheet: *sheet,
                keys: keys.clone(),
                prev: keys
                    .iter()
                    .map(|k| ColumnSnapshot {
                        key: k.clone(),
                        state: ColState::default(),
                        cell_values: Vec::new(),
                        cell_styles: Vec::new(),
                        prop_ts: Vec::new(),
                    })
                    .collect(),
            }),
            Patch::SetRowProperty {
                sheet, row, prev, ..
            } => {
                if let Some(property) = prev {
                    out.push(Patch::SetRowProperty {
                        sheet: *sheet,
                        row: row.clone(),
                        property: property.clone(),
                        ts: None,
                        prev: None,
                    });
                }
            }
            Patch::SetColumnSpan {
                sheet, span, prev, ..
            } => {
                if let Some(property) = prev {
                    out.push(Patch::SetColumnSpan {
                        sheet: *sheet,
                        span: span.clone(),
                        property: property.clone(),
                        ts: None,
                        prev: None,
                    });
                }
            }
            Patch::SetSheetProperty { sheet, prev, .. } => {
                if let Some(property) = prev {
                    out.push(Patch::SetSheetProperty {
                        sheet: *sheet,
                        property: property.clone(),
                        prev: None,
                    });
                }
            }
            Patch::SetWorkbookProperty { prev, .. } => {
                if let Some(property) = prev {
                    out.push(Patch::SetWorkbookProperty {
                        property: property.clone(),
                        prev: None,
                    });
                }
            }
            Patch::SetDefinedName { id, property, prev } => {
                if let Some(previous) = prev {
                    out.push(Patch::SetDefinedName {
                        id: *id,
                        property: previous.clone(),
                        prev: Some(property.clone()),
                    });
                }
            }
            Patch::SetNamedStyle { id, property, prev } => {
                if let Some(previous) = prev {
                    out.push(Patch::SetNamedStyle {
                        id: *id,
                        property: previous.clone(),
                        prev: Some(property.clone()),
                    });
                }
            }
            Patch::AddConditionalFormat { sheet, key, .. } => {
                out.push(Patch::DeleteConditionalFormat {
                    sheet: *sheet,
                    key: key.clone(),
                    prev: None,
                })
            }
            Patch::DeleteConditionalFormat { sheet, key, prev } => {
                if let Some(state) = prev {
                    out.push(Patch::AddConditionalFormat {
                        sheet: *sheet,
                        key: key.clone(),
                        rule: Box::new(state.rule.clone()),
                        ranges: state.ranges.clone(),
                    });
                }
            }
            Patch::SetConditionalFormat {
                sheet, key, prev, ..
            } => {
                if let Some(property) = prev {
                    out.push(Patch::SetConditionalFormat {
                        sheet: *sheet,
                        key: key.clone(),
                        property: property.clone(),
                        prev: None,
                    });
                }
            }
            Patch::SetMergedRange {
                sheet,
                range,
                merged,
                prev,
            } => out.push(Patch::SetMergedRange {
                sheet: *sheet,
                range: range.clone(),
                merged: *prev,
                prev: *merged,
            }),
            Patch::SetComment {
                sheet,
                at,
                comment,
                prev,
            } => out.push(Patch::SetComment {
                sheet: *sheet,
                at: at.clone(),
                comment: prev.clone(),
                prev: comment.clone(),
            }),
            Patch::DeleteRows { sheet, keys, prev } => {
                // Snapshots came off the wire, or none were taken: there is nothing to restore.
                if prev.len() != keys.len() {
                    continue;
                }
                out.push(Patch::InsertRows {
                    sheet: *sheet,
                    keys: keys.clone(),
                });
                for snap in prev {
                    let state = &snap.state;
                    let ts = |kind| {
                        Some(
                            snap.prop_ts
                                .iter()
                                .find_map(|(k, ts)| if *k == kind { Some(*ts) } else { None })
                                .unwrap_or_default(), // zero timestamp so it loses over any more recent
                        )
                    };
                    if let Some(style) = &state.style {
                        out.push(Patch::SetRowProperty {
                            sheet: *sheet,
                            row: snap.key.clone(),
                            property: RowProperty::Style(Some(style.clone())),
                            ts: ts(RowPropKind::Style),
                            prev: None,
                        });
                    }
                    if state.custom_height {
                        out.push(Patch::SetRowProperty {
                            sheet: *sheet,
                            row: snap.key.clone(),
                            property: RowProperty::Height(state.height),
                            ts: ts(RowPropKind::Height),
                            prev: None,
                        });
                    }
                    if state.hidden {
                        out.push(Patch::SetRowProperty {
                            sheet: *sheet,
                            row: snap.key.clone(),
                            property: RowProperty::Hidden(true),
                            ts: ts(RowPropKind::Hidden),
                            prev: None,
                        });
                    }
                }
                for snap in prev {
                    for (col, input, ts) in &snap.cell_values {
                        out.push(Patch::SetCellValue {
                            sheet: *sheet,
                            at: (snap.key.clone(), col.clone()),
                            value: Some(input.clone()),
                            ts: Some(*ts),
                            prev: Box::new(None),
                        });
                    }
                }
                for snap in prev {
                    for (col, style, ts) in &snap.cell_styles {
                        out.push(Patch::SetCellStyle {
                            sheet: *sheet,
                            at: (snap.key.clone(), col.clone()),
                            style: Some(Box::new(style.clone())),
                            ts: Some(*ts),
                            prev: Box::new(None),
                        });
                    }
                }
            }
            Patch::DeleteColumns { sheet, keys, prev } => {
                if prev.len() != keys.len() {
                    continue;
                }
                out.push(Patch::InsertColumns {
                    sheet: *sheet,
                    keys: keys.clone(),
                });
                for snap in prev {
                    let state = &snap.state;
                    let span = (snap.key.clone(), snap.key.clone());
                    let ts = |kind| {
                        Some(
                            snap.prop_ts
                                .iter()
                                .find_map(|(k, ts)| if *k == kind { Some(*ts) } else { None })
                                .unwrap_or_default(), // zero timestamp so it loses over any more recent
                        )
                    };
                    if let Some(style) = &state.style {
                        out.push(Patch::SetColumnSpan {
                            sheet: *sheet,
                            span: span.clone(),
                            property: ColProperty::Style(Some(style.clone())),
                            ts: ts(ColPropKind::Style),
                            prev: None,
                        });
                    }
                    if state.custom_width {
                        out.push(Patch::SetColumnSpan {
                            sheet: *sheet,
                            span: span.clone(),
                            property: ColProperty::Width(state.width),
                            ts: ts(ColPropKind::Width),
                            prev: None,
                        });
                    }
                    if state.hidden {
                        out.push(Patch::SetColumnSpan {
                            sheet: *sheet,
                            span,
                            property: ColProperty::Hidden(true),
                            ts: ts(ColPropKind::Hidden),
                            prev: None,
                        });
                    }
                }
                for snap in prev {
                    for (row, input, ts) in &snap.cell_values {
                        out.push(Patch::SetCellValue {
                            sheet: *sheet,
                            at: (row.clone(), snap.key.clone()),
                            value: Some(input.clone()),
                            ts: Some(*ts),
                            prev: Box::new(None),
                        });
                    }
                }
                for snap in prev {
                    for (row, style, ts) in &snap.cell_styles {
                        out.push(Patch::SetCellStyle {
                            sheet: *sheet,
                            at: (row.clone(), snap.key.clone()),
                            style: Some(Box::new(style.clone())),
                            ts: Some(*ts),
                            prev: Box::new(None),
                        });
                    }
                }
            }
            Patch::MoveRows { sheet, moves, prev } => {
                if prev.len() != moves.len() {
                    continue; // came off the wire: no undo data
                }
                out.push(Patch::MoveRows {
                    sheet: *sheet,
                    moves: moves
                        .iter()
                        .zip(prev)
                        .map(|((id, _), held)| (id.clone(), held.clone()))
                        .collect(),
                    // The forward destinations, so inverting the inverse moves them forward again.
                    prev: moves.iter().map(|(_, dest)| dest.clone()).collect(),
                });
            }
            Patch::MoveColumns { sheet, moves, prev } => {
                if prev.len() != moves.len() {
                    continue;
                }
                out.push(Patch::MoveColumns {
                    sheet: *sheet,
                    moves: moves
                        .iter()
                        .zip(prev)
                        .map(|((id, _), held)| (id.clone(), held.clone()))
                        .collect(),
                    prev: moves.iter().map(|(_, dest)| dest.clone()).collect(),
                });
            }
            Patch::AddSheet {
                id,
                name,
                position,
                content,
            } => out.push(Patch::DeleteSheet {
                sheet: *id,
                // The redo reuses the id, so references to the sheet resolve again.
                prev: Some(Box::new(SheetRestore {
                    name: name.clone(),
                    position: position.clone(),
                    content: content.clone(),
                })),
            }),
            Patch::DeleteSheet {
                sheet,
                prev: Some(restore),
            } => out.push(Patch::AddSheet {
                id: *sheet,
                name: restore.name.clone(),
                position: restore.position.clone(),
                content: restore.content.clone(),
            }),
            Patch::SetArrayValue { .. }
            | Patch::DeleteSheet { prev: None, .. }
            | Patch::MoveConditionalFormats { .. } => {}
        }
    }
    out
}

#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub enum Patch {
    // ---- Cells ----
    /// `value: None` clears the cell's contents. A range clear fans out to one patch per populated
    /// cell, so that every write stays a single-register assignment.
    SetCellValue {
        sheet: SheetId,
        at: StableCellAddress,
        value: Option<CellInput>,

        /// `None` applies at the commit's stamp; `Some` replays at the given stamp — undo restores
        /// use it so they lose to concurrent newer edits.
        ts: Option<Timestamp>,

        #[bitcode(skip)]
        prev: Box<Option<CellInput>>,
    },
    /// `value` is a [`CellInput::Array`], or `None` to clear the array. `prev` covers the whole
    /// range the array occupied, which is why this is not folded into [`Patch::SetCellValue`].
    SetArrayValue {
        sheet: SheetId,
        anchor: StableCellAddress,
        value: Option<CellInput>,

        #[bitcode(skip)]
        prev: Vec<Vec<Option<CellInput>>>,
    },
    /// `style: None` clears the cell's formatting.
    SetCellStyle {
        sheet: SheetId,
        at: StableCellAddress,
        style: Option<Box<Style>>,

        /// See [`Patch::SetCellValue`]'s `ts`.
        ts: Option<Timestamp>,

        #[bitcode(skip)]
        prev: Box<Option<Style>>,
    },

    // ---- Rows ----
    InsertRows {
        sheet: SheetId,
        keys: Vec<FractionalKey>,
    },
    DeleteRows {
        sheet: SheetId,
        keys: Vec<FractionalKey>,

        #[bitcode(skip)]
        prev: Vec<RowSnapshot>,
    },
    /// `(source identity, destination key)` pairs, both minted by the author. Minting is
    /// session-dependent, so destinations travel in the patch: every replica applies the keys it is
    /// given rather than minting its own.
    MoveRows {
        sheet: SheetId,
        moves: Vec<(FractionalKey, FractionalKey)>,

        /// The key each element held before the move, aligned with `moves` — what the inverse files
        /// it back to.
        #[bitcode(skip)]
        prev: Vec<FractionalKey>,
    },
    SetRowProperty {
        sheet: SheetId,
        row: FractionalKey,
        property: RowProperty,

        /// See [`Patch::SetCellValue`]'s `ts`.
        ts: Option<Timestamp>,

        /// Same discriminant as `property`, holding the value it replaced.
        #[bitcode(skip)]
        prev: Option<RowProperty>,
    },

    // ---- Columns ----
    InsertColumns {
        sheet: SheetId,
        keys: Vec<FractionalKey>,
    },
    DeleteColumns {
        sheet: SheetId,
        keys: Vec<FractionalKey>,

        #[bitcode(skip)]
        prev: Vec<ColumnSnapshot>,
    },
    /// `(source identity, destination key)` pairs — see [`Patch::MoveRows`].
    MoveColumns {
        sheet: SheetId,
        moves: Vec<(FractionalKey, FractionalKey)>,

        /// The key each element held before the move, aligned with `moves` — what the inverse files
        /// it back to.
        #[bitcode(skip)]
        prev: Vec<FractionalKey>,
    },
    /// A property write over a column *span*, addressed by its corner keys — see
    /// [`Col`](crate::types::Col) for what a span covers. A [`FractionalKey::NULL`] corner is an
    /// open end: `(NULL, k)` and `(k, NULL)` are half-open, and `(NULL, NULL)` is every column,
    /// including ones no replica has materialized yet — the register whole-sheet styling writes to,
    /// which ordinal code spells as a `(1, LAST_COLUMN)` record and stable-land cannot, having no
    /// "last" key.
    SetColumnSpan {
        sheet: SheetId,
        span: (FractionalKey, FractionalKey),
        property: ColProperty,

        /// See [`Patch::SetCellValue`]'s `ts`.
        ts: Option<Timestamp>,

        /// Same discriminant as `property`, holding the value it replaced.
        #[bitcode(skip)]
        prev: Option<ColProperty>,
    },

    // ---- Sheets ----
    /// `content: None` creates a blank sheet. `Some(..)` covers both duplicating an existing sheet
    /// and undoing a [`Patch::DeleteSheet`] — in the latter case `id` is the deleted sheet's own
    /// id, so existing [`SheetId`] references resolve again.
    ///
    /// The content is captured on the authoring replica at commit time, never resolved at apply
    /// time. Resolving on apply would make the result depend on which concurrent edits to the source
    /// sheet a replica had already seen, and replicas would diverge.
    ///
    /// `position` is the sheet's place in the tab order.
    AddSheet {
        id: u32,
        name: String,
        position: FractionalKey,
        content: Option<Box<SheetContent>>,
    },
    DeleteSheet {
        sheet: SheetId,

        #[bitcode(skip)]
        prev: Option<Box<SheetRestore>>,
    },
    SetSheetProperty {
        sheet: SheetId,
        property: SheetProperty,

        /// Same discriminant as `property`, holding the value it replaced.
        #[bitcode(skip)]
        prev: Option<SheetProperty>,
    },

    // ---- Workbook ----
    SetWorkbookProperty {
        property: WorkbookProperty,

        /// Same discriminant as `property`, holding the value it replaced.
        #[bitcode(skip)]
        prev: Option<WorkbookProperty>,
    },

    SetDefinedName {
        id: DefinedNameId,
        property: DefinedNameProperty,

        #[bitcode(skip)]
        prev: Option<DefinedNameProperty>,
    },

    SetNamedStyle {
        id: NamedStyleId,
        property: NamedStyleProperty,

        #[bitcode(skip)]
        prev: Option<NamedStyleProperty>,
    },

    // ---- Conditional formatting ----
    // A rule is identified by the `FractionalKey` minted when it was created, and its priority is
    // its position in the worksheet's conditional formatting index. Reordering is therefore a move,
    // not a property write, and the `u32` priority together with the swap operation it required both
    // disappear.
    AddConditionalFormat {
        sheet: SheetId,
        key: FractionalKey,
        rule: Box<CfRule>,
        ranges: Vec<StableRange>,
    },
    DeleteConditionalFormat {
        sheet: SheetId,
        key: FractionalKey,

        #[bitcode(skip)]
        prev: Option<Box<ConditionalFormatState>>,
    },
    /// Dead: superseded by [`CfProperty::Priority`], which writes a position register instead of
    /// reordering an index. Kept because variants are append-only; applying it does nothing.
    MoveConditionalFormats {
        sheet: SheetId,
        keys: Vec<FractionalKey>,
        dest: FractionalKey,
    },
    SetConditionalFormat {
        sheet: SheetId,
        key: FractionalKey,
        property: CfProperty,

        /// Same discriminant as `property`, holding the value it replaced.
        #[bitcode(skip)]
        prev: Option<CfProperty>,
    },

    /// Whether the cells `range` covers are merged.
    SetMergedRange {
        sheet: SheetId,
        range: StableRange,
        merged: bool,

        #[bitcode(skip)]
        prev: bool,
    },
    /// `comment: None` removes the comment on `at`.
    SetComment {
        sheet: SheetId,
        at: StableCellAddress,
        comment: Option<Comment<Stable>>,

        #[bitcode(skip)]
        prev: Option<Comment<Stable>>,
    },
}

impl Patch {
    /// The sheet this writes *into*, if any. Workbook-scoped patches answer `None`, and so do
    /// [`Patch::AddSheet`]/[`Patch::DeleteSheet`], which manage sheet existence rather than obey it.
    pub(crate) fn target_sheet(&self) -> Option<SheetId> {
        match self {
            Patch::SetCellValue { sheet, .. }
            | Patch::SetArrayValue { sheet, .. }
            | Patch::SetCellStyle { sheet, .. }
            | Patch::InsertRows { sheet, .. }
            | Patch::DeleteRows { sheet, .. }
            | Patch::MoveRows { sheet, .. }
            | Patch::SetRowProperty { sheet, .. }
            | Patch::InsertColumns { sheet, .. }
            | Patch::DeleteColumns { sheet, .. }
            | Patch::MoveColumns { sheet, .. }
            | Patch::SetColumnSpan { sheet, .. }
            | Patch::SetSheetProperty { sheet, .. }
            | Patch::AddConditionalFormat { sheet, .. }
            | Patch::DeleteConditionalFormat { sheet, .. }
            | Patch::MoveConditionalFormats { sheet, .. }
            | Patch::SetConditionalFormat { sheet, .. }
            | Patch::SetMergedRange { sheet, .. }
            | Patch::SetComment { sheet, .. } => Some(*sheet),
            // `SetDefinedName`'s scope is a name scope, not a place a write lands.
            Patch::AddSheet { .. }
            | Patch::DeleteSheet { .. }
            | Patch::SetWorkbookProperty { .. }
            | Patch::SetDefinedName { .. }
            | Patch::SetNamedStyle { .. } => None,
        }
    }
}

/// A property of a single row. Each variant is a distinct register.
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub enum RowProperty {
    /// `None` deletes the row style.
    Style(Option<Box<Style>>),
    Height(f64),
    Hidden(bool),
}

/// The register a [`RowProperty`] writes to, without its value.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Encode, Decode)]
pub enum RowPropKind {
    Style,
    Height,
    Hidden,
}

impl RowProperty {
    pub fn kind(&self) -> RowPropKind {
        match self {
            RowProperty::Style(_) => RowPropKind::Style,
            RowProperty::Height(_) => RowPropKind::Height,
            RowProperty::Hidden(_) => RowPropKind::Hidden,
        }
    }
}

/// A property of a single column. Each variant is a distinct register.
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub enum ColProperty {
    /// `None` deletes the column style.
    Style(Option<Box<Style>>),
    Width(f64),
    Hidden(bool),
}

/// The register a [`ColProperty`] writes to, without its value.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Encode, Decode)]
pub enum ColPropKind {
    Style,
    Width,
    Hidden,
}

impl ColProperty {
    pub fn kind(&self) -> ColPropKind {
        match self {
            ColProperty::Style(_) => ColPropKind::Style,
            ColProperty::Width(_) => ColPropKind::Width,
            ColProperty::Hidden(_) => ColPropKind::Hidden,
        }
    }
}

/// A property of a single worksheet. Each variant is a distinct register.
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub enum SheetProperty {
    Name(String),
    Color(Color),
    State(SheetState),
    ShowGridLines(bool),
    FrozenRows(i32),
    FrozenColumns(i32),
    /// Where the sheet sits in the tab order. Moving a sheet is a write to this register.
    Position(FractionalKey),
}

/// The register a [`SheetProperty`] writes to, without its value.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Encode, Decode)]
pub enum SheetPropKind {
    Name,
    Color,
    State,
    ShowGridLines,
    FrozenRows,
    FrozenColumns,
    Position,
}

impl SheetProperty {
    pub fn kind(&self) -> SheetPropKind {
        match self {
            SheetProperty::Name(_) => SheetPropKind::Name,
            SheetProperty::Color(_) => SheetPropKind::Color,
            SheetProperty::State(_) => SheetPropKind::State,
            SheetProperty::ShowGridLines(_) => SheetPropKind::ShowGridLines,
            SheetProperty::FrozenRows(_) => SheetPropKind::FrozenRows,
            SheetProperty::FrozenColumns(_) => SheetPropKind::FrozenColumns,
            SheetProperty::Position(_) => SheetPropKind::Position,
        }
    }
}

/// A workbook-global property. Each variant is a distinct register.
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub enum WorkbookProperty {
    Theme(Box<Theme>),
    Locale(String),
    Timezone(String),
}

/// The register a [`WorkbookProperty`] writes to, without its value.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Encode, Decode)]
pub enum WorkbookPropKind {
    Theme,
    Locale,
    Timezone,
}

impl WorkbookProperty {
    pub fn kind(&self) -> WorkbookPropKind {
        match self {
            WorkbookProperty::Theme(_) => WorkbookPropKind::Theme,
            WorkbookProperty::Locale(_) => WorkbookPropKind::Locale,
            WorkbookProperty::Timezone(_) => WorkbookPropKind::Timezone,
        }
    }
}

/// A property of a single conditional formatting rule. Each variant is a distinct register.
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub enum CfProperty {
    Rule(Box<CfRule>),
    Ranges(Vec<StableRange>),
    /// Where the rule sits among the sheet's rules, which is what its priority *is*. A rule with
    /// no position written sorts by its own identity key.
    Priority(FractionalKey),
}

/// The register a [`CfProperty`] writes to, without its value.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Encode, Decode)]
pub enum CfPropKind {
    Rule,
    Ranges,
    Priority,
}

impl CfProperty {
    pub fn kind(&self) -> CfPropKind {
        match self {
            CfProperty::Rule(_) => CfPropKind::Rule,
            CfProperty::Ranges(_) => CfPropKind::Ranges,
            CfProperty::Priority(_) => CfPropKind::Priority,
        }
    }
}

/// A named cell style, carried by value rather than as an `xf_id` index into
/// `Styles.cell_style_xfs`, which is assigned per replica. `builtin_id` is an OOXML constant and is
/// safe to replicate as-is.
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct NamedStyle {
    pub style: Style,
    pub builtin_id: i32,
}

/// A property of a named style. Each variant is a distinct register.
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub enum NamedStyleProperty {
    /// What the author called it. The name shown in the style table is derived from this and
    /// repaired for collisions, exactly as a sheet's is.
    Name(String),
    /// `None` deletes the style. Its name register survives, so an undo can revive it.
    Definition(Option<Box<NamedStyle>>),
}

/// The register a [`NamedStyleProperty`] writes to, without its value.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Encode, Decode)]
pub enum NamedStylePropKind {
    Name,
    Definition,
}

impl NamedStyleProperty {
    pub fn kind(&self) -> NamedStylePropKind {
        match self {
            NamedStyleProperty::Name(_) => NamedStylePropKind::Name,
            NamedStyleProperty::Definition(_) => NamedStylePropKind::Definition,
        }
    }
}

/// A property of a defined name. Each variant is a distinct register, so a concurrent rename and
/// redefinition of the same name both survive.
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub enum DefinedNameProperty {
    /// Where the name lives and what the author called it. A scope move is address-shaped like a
    /// rename, so both ride one register. The name shown is derived from this and repaired for
    /// collisions, exactly as a sheet's is.
    Name((Option<SheetId>, String)),
    /// `None` deletes the name. Its address register survives, so an undo can revive it.
    Definition(Option<String>),
}

/// The register a [`DefinedNameProperty`] writes to, without its value.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Encode, Decode)]
pub enum DefinedNamePropKind {
    Name,
    Definition,
}

impl DefinedNameProperty {
    pub fn kind(&self) -> DefinedNamePropKind {
        match self {
            DefinedNameProperty::Name(_) => DefinedNamePropKind::Name,
            DefinedNameProperty::Definition(_) => DefinedNamePropKind::Definition,
        }
    }
}

/// A conditional formatting rule, without its priority — priority is the rule's position in the
/// worksheet's conditional formatting index.
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct ConditionalFormatState {
    pub rule: CfRule,
    pub ranges: Vec<StableRange>,
}

/// Everything a [`Patch::DeleteSheet`] removed, as the [`Patch::AddSheet`] that puts it back.
/// Local-only undo data.
#[derive(Clone, Debug, PartialEq)]
pub struct SheetRestore {
    pub name: String,
    pub position: FractionalKey,
    pub content: Option<Box<SheetContent>>,
}

/// A complete worksheet payload, used to seed [`Patch::AddSheet`] and to restore a
/// [`Patch::DeleteSheet`]. The sheet's name is carried by `AddSheet` itself and so is absent here.
///
/// Every field is index-free — see the module documentation.
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct SheetContent {
    pub state: SheetState,
    pub color: Color,
    pub show_grid_lines: bool,
    pub frozen_rows: i32,
    pub frozen_columns: i32,
    /// Ordered by [`FractionalKey`].
    pub rows: Vec<(FractionalKey, RowState)>,
    /// Column spans, addressed by corner keys; a [`FractionalKey::NULL`] corner is an open end, so
    /// `(NULL, NULL)` is the whole-sheet span.
    pub columns: Vec<((FractionalKey, FractionalKey), ColState)>,
    /// Cell contents and cell styles are kept apart because most cells carry no style of their own,
    /// and they are separate registers in any case.
    pub cell_values: Vec<(StableCellAddress, CellInput)>,
    pub cell_styles: Vec<(StableCellAddress, Style)>,
    pub merge_cells: Vec<StableRange>,
    pub comments: Vec<Comment<Stable>>,
    /// Ordered by [`FractionalKey`], which is both each rule's identity and its priority.
    pub conditional_formatting: Vec<(FractionalKey, ConditionalFormatState)>,
}

#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct RowState {
    pub height: f64,
    pub hidden: bool,
    pub style: Option<Box<Style>>,
    pub custom_height: bool,
    pub custom_format: bool,
}

impl Default for RowState {
    fn default() -> Self {
        RowState {
            height: DEFAULT_ROW_HEIGHT / ROW_HEIGHT_FACTOR,
            hidden: false,
            style: None,
            custom_height: false,
            custom_format: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct ColState {
    pub width: f64,
    pub hidden: bool,
    pub style: Option<Box<Style>>,
    pub custom_width: bool,
}

impl Default for ColState {
    fn default() -> Self {
        ColState {
            width: DEFAULT_COLUMN_WIDTH / COLUMN_WIDTH_FACTOR,
            hidden: false,
            style: None,
            custom_width: false,
        }
    }
}

/// Everything a [`Patch::DeleteRows`] removed, so that undo can put it back. Local-only undo data.
/// Each entry carries the stamp of the write it captured, so the restore can replay at that stamp.
#[derive(Clone, Debug, PartialEq)]
pub struct RowSnapshot {
    pub key: FractionalKey,
    pub state: RowState,
    /// Keyed by column.
    pub cell_values: Vec<(FractionalKey, CellInput, Timestamp)>,
    /// Keyed by column.
    pub cell_styles: Vec<(FractionalKey, Style, Timestamp)>,
    /// Only the property kinds that had a register entry.
    pub prop_ts: Vec<(RowPropKind, Timestamp)>,
}

/// Everything a [`Patch::DeleteColumns`] removed, so that undo can put it back. Local-only undo
/// data.
#[derive(Clone, Debug, PartialEq)]
pub struct ColumnSnapshot {
    pub key: FractionalKey,
    pub state: ColState,
    /// Keyed by row.
    pub cell_values: Vec<(FractionalKey, CellInput, Timestamp)>,
    /// Keyed by row.
    pub cell_styles: Vec<(FractionalKey, Style, Timestamp)>,
    /// Only the property kinds that had a register entry.
    pub prop_ts: Vec<(ColPropKind, Timestamp)>,
}

/// The contents of a cell as authored, never as evaluated.
///
/// Literals are carried already parsed rather than as the text the user typed: parsing depends on
/// the workbook locale, which is itself a replicated register, so re-parsing on each replica could
/// resolve differently. Formulas are carried in the internal (English) form for the same reason.
///
/// Evaluated results — [`FormulaValue`](crate::types::FormulaValue), spill values and spill cells —
/// are derived state. They are recomputed locally and never replicated.
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub enum CellInput {
    Number(f64),
    Boolean(bool),
    Text(String),
    Error(Error),
    Formula(String),
    /// The anchor of an array formula. The cells it spills into are derived, not stored.
    Array {
        formula: String,
        range: StableRange,
        kind: ArrayKind,
    },
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::collab::hlc::Hlc;
    use crate::collab::log::CommitId;

    fn key(byte: u8) -> FractionalKey {
        FractionalKey::from([byte, 0, 0, 0, 0].as_slice())
    }

    fn range() -> StableRange {
        StableRange {
            rows: Some((key(1), key(2))),
            cols: Some((key(3), FractionalKey::NULL)),
        }
    }

    fn comment() -> Comment<Stable> {
        Comment {
            text: "note".to_string(),
            author_name: "me".to_string(),
            author_id: None,
            cell_ref: (key(1), key(3)),
        }
    }

    fn row_state() -> RowState {
        RowState {
            height: 20.0,
            hidden: false,
            style: Some(Box::default()),
            custom_height: true,
            custom_format: false,
        }
    }

    fn col_state() -> ColState {
        ColState {
            width: 40.0,
            hidden: true,
            style: None,
            custom_width: true,
        }
    }

    fn cf_rule() -> CfRule {
        CfRule::Formula {
            formula: "A1>0".to_string(),
            dxf_id: 0,
            stop_if_true: false,
        }
    }

    fn content() -> SheetContent {
        SheetContent {
            state: SheetState::Visible,
            color: Color::Rgb("#112233".to_string()),
            show_grid_lines: false,
            frozen_rows: 1,
            frozen_columns: 2,
            rows: vec![(key(1), row_state())],
            columns: vec![((key(3), key(4)), col_state())],
            cell_values: vec![((key(1), key(3)), CellInput::Number(3.5))],
            cell_styles: vec![((key(1), key(3)), Style::default())],
            merge_cells: vec![range()],
            comments: vec![comment()],
            conditional_formatting: vec![(
                key(9),
                ConditionalFormatState {
                    rule: cf_rule(),
                    ranges: vec![range()],
                },
            )],
        }
    }

    /// One of every variant, with every `prev` left at its default so the round trip is an equality.
    fn every_variant() -> Vec<Patch> {
        vec![
            Patch::SetCellValue {
                sheet: 7,
                at: (key(1), key(3)),
                value: Some(CellInput::Text("hi".to_string())),
                ts: None,
                prev: Box::default(),
            },
            Patch::SetArrayValue {
                sheet: 7,
                anchor: (key(1), key(3)),
                value: Some(CellInput::Array {
                    formula: "SEQUENCE(2)".to_string(),
                    range: range(),
                    kind: ArrayKind::Dynamic,
                }),
                prev: Vec::new(),
            },
            Patch::SetCellStyle {
                sheet: 7,
                at: (key(1), key(3)),
                style: Some(Box::default()),
                ts: None,
                prev: Box::default(),
            },
            Patch::InsertRows {
                sheet: 7,
                keys: vec![key(1), key(2)],
            },
            Patch::DeleteRows {
                sheet: 7,
                keys: vec![key(2)],
                prev: Vec::new(),
            },
            Patch::MoveRows {
                sheet: 7,
                moves: vec![(key(1), key(5))],
                prev: vec![],
            },
            Patch::SetRowProperty {
                sheet: 7,
                row: key(1),
                property: RowProperty::Height(33.0),
                // A restore's stamp travels: it is not `#[bitcode(skip)]` undo data.
                ts: Some(Timestamp::new(Hlc::new(42), 3)),
                prev: None,
            },
            Patch::InsertColumns {
                sheet: 7,
                keys: vec![key(3)],
            },
            Patch::DeleteColumns {
                sheet: 7,
                keys: vec![key(4)],
                prev: Vec::new(),
            },
            Patch::MoveColumns {
                sheet: 7,
                moves: vec![(key(3), key(6))],
                prev: vec![],
            },
            Patch::SetColumnSpan {
                sheet: 7,
                span: (FractionalKey::NULL, FractionalKey::NULL),
                property: ColProperty::Style(Some(Box::default())),
                ts: None,
                prev: None,
            },
            Patch::AddSheet {
                id: 7,
                name: "Sheet1".to_string(),
                position: key(1),
                content: Some(Box::new(content())),
            },
            Patch::DeleteSheet {
                sheet: 7,
                prev: None,
            },
            Patch::SetSheetProperty {
                sheet: 7,
                property: SheetProperty::Position(key(2)),
                prev: None,
            },
            Patch::SetWorkbookProperty {
                property: WorkbookProperty::Theme(Box::default()),
                prev: None,
            },
            Patch::SetDefinedName {
                id: 11,
                property: DefinedNameProperty::Name((Some(7), "total".to_string())),
                prev: None,
            },
            Patch::SetDefinedName {
                id: 11,
                property: DefinedNameProperty::Definition(Some("Sheet1!$A$1".to_string())),
                prev: None,
            },
            Patch::SetNamedStyle {
                id: 42,
                property: NamedStyleProperty::Definition(Some(Box::new(NamedStyle {
                    style: Style::default(),
                    builtin_id: 26,
                }))),
                prev: None,
            },
            Patch::SetNamedStyle {
                id: 42,
                property: NamedStyleProperty::Name("Good".to_string()),
                prev: None,
            },
            Patch::AddConditionalFormat {
                sheet: 7,
                key: key(9),
                rule: Box::new(cf_rule()),
                ranges: vec![range()],
            },
            Patch::DeleteConditionalFormat {
                sheet: 7,
                key: key(9),
                prev: None,
            },
            Patch::MoveConditionalFormats {
                sheet: 7,
                keys: vec![key(9)],
                dest: key(10),
            },
            Patch::SetConditionalFormat {
                sheet: 7,
                key: key(9),
                property: CfProperty::Ranges(vec![range()]),
                prev: None,
            },
            Patch::SetMergedRange {
                sheet: 7,
                range: range(),
                merged: true,
                prev: false,
            },
            Patch::SetComment {
                sheet: 7,
                at: (key(1), key(3)),
                comment: Some(comment()),
                prev: None,
            },
        ]
    }

    #[test]
    fn patch_wire_round_trip() {
        let patches = every_variant();
        let bytes = encode_patches(&patches).unwrap();
        assert_eq!(bytes[0], PATCH_FORMAT_VERSION);
        assert_eq!(decode_patches(&bytes).unwrap(), patches);

        // Undo data never leaves the replica that produced it: it decodes back as the default.
        let local = vec![Patch::SetRowProperty {
            sheet: 7,
            row: key(1),
            property: RowProperty::Hidden(true),
            ts: None,
            prev: Some(RowProperty::Hidden(false)),
        }];
        let decoded = decode_patches(&encode_patches(&local).unwrap()).unwrap();
        assert_ne!(decoded, local);
        match &decoded[0] {
            Patch::SetRowProperty { property, prev, .. } => {
                assert_eq!(property, &RowProperty::Hidden(true));
                assert_eq!(prev, &None);
            }
            other => panic!("wrong variant: {other:?}"),
        }

        // A payload this build cannot interpret is rejected rather than misread.
        let mut wrong = bytes.clone();
        wrong[0] = PATCH_FORMAT_VERSION.wrapping_add(1);
        assert!(decode_patches(&wrong).is_err());
        assert!(decode_patches(&[]).is_err());

        // Every property enum reports the register it writes to.
        assert_eq!(RowProperty::Height(1.0).kind(), RowPropKind::Height);
        assert_eq!(ColProperty::Hidden(true).kind(), ColPropKind::Hidden);
        assert_eq!(
            SheetProperty::Position(key(1)).kind(),
            SheetPropKind::Position
        );
        assert_eq!(
            WorkbookProperty::Locale("en".to_string()).kind(),
            WorkbookPropKind::Locale
        );
        assert_eq!(CfProperty::Ranges(vec![]).kind(), CfPropKind::Ranges);
        assert_eq!(
            NamedStyleProperty::Name(String::new()).kind(),
            NamedStylePropKind::Name
        );
        assert_eq!(
            DefinedNameProperty::Definition(None).kind(),
            DefinedNamePropKind::Definition
        );

        // Commit ids ride the same wire.
        let id = CommitId::from([1u8, 2, 3, 4, 5, 6, 7, 8, 9].as_slice());
        let decoded: CommitId = bitcode::decode(&bitcode::encode(&id)).unwrap();
        assert_eq!(decoded, id);
        assert_eq!(&*decoded, &[1u8, 2, 3, 4, 5, 6, 7, 8, 9]);
        let empty: CommitId = bitcode::decode(&bitcode::encode(&CommitId::default())).unwrap();
        assert_eq!(empty, CommitId::default());
    }
}
