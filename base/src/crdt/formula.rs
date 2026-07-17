//! Formula reference extraction and the id-form codec (phase 3.1/3.2).
//!
//! The document must not store formulas with positional references — a
//! concurrent structural edit would displace them on one replica only.
//! Instead, replicated formulas are **canonical text with every reference
//! replaced by a stable-id token**:
//!
//! ```text
//!   =SUM(A1:B5)*2   ⇒   =SUM(␟r1;r1:r2;r5␟)*2
//! ```
//!
//! (`␟` is [`REF_DELIM`], ASCII unit separator `\u{1F}`; `r`/`a` prefixes are
//! the relative/absolute flags, the rest are [`EntityId`] encodings.)
//!
//! * **Extraction** ([`extract_reference_spans`]): the engine's lexer already
//!   produces positioned tokens (`expressions::lexer::util::get_tokens`), so
//!   references and ranges are located as char spans in the canonical text.
//!   Anything the codec cannot represent (structured/table references,
//!   illegal tokens) makes the whole formula *unsupported*: the caller falls
//!   back to storing plain text.
//! * **Encoding** ([`encode_formula`]): reference spans are spliced out and
//!   replaced by id tokens; everything else is kept verbatim (with the
//!   delimiter escaped by doubling, in case a string literal contains it).
//! * **Rendering** ([`render_formula`]): id tokens are resolved against the
//!   *current* orders and substituted with A1 references. A tombstoned id in
//!   a single reference renders as `#REF!`; a tombstoned **range endpoint**
//!   clamps inward to the nearest visible row/column (spreadsheet intuition:
//!   deleting an edge row shrinks the range); a range whose endpoints end up
//!   crossed renders as `#REF!`. A reference to a deleted or unknown sheet
//!   renders as `#REF!`.
//!
//! The codec is deliberately independent of the session: resolution goes
//! through the [`RefResolver`] trait, so it can be unit-tested against
//! hand-built [`AxisOrder`]s and wired to the live projection in phase 3.3.

use crate::expressions::lexer::util::get_tokens;
use crate::expressions::token::TokenType;
use crate::expressions::utils::{number_to_column, quote_name};

use super::ids::{EntityId, MAX_COLUMN, MAX_ROW};
use super::order::ResolvedIndex;

/// Delimiter around id tokens inside a replicated formula. ASCII "unit
/// separator": never produced by typing; escaped by doubling if it appears
/// in the source text.
pub(crate) const REF_DELIM: char = '\u{1f}';

/// A formula that cannot be represented in id-form; the caller stores the
/// plain text instead (with the structural-op fan-out as the fallback
/// consistency mechanism).
#[derive(Debug, PartialEq)]
pub(crate) struct Unsupported(pub &'static str);

/// One endpoint of a reference: engine coordinates plus absolute flags.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct CellEndpoint {
    pub column: i32,
    pub row: i32,
    pub absolute_column: bool,
    pub absolute_row: bool,
}

/// A reference occurrence in a formula: a char span (`start..end`) plus the
/// parsed components. `right` is `Some` for ranges.
#[derive(Debug, PartialEq)]
pub(crate) struct ReferenceSpan {
    pub start: usize,
    pub end: usize,
    pub sheet: Option<String>,
    pub left: CellEndpoint,
    pub right: Option<CellEndpoint>,
}

/// Locates every cell reference and range in a canonical (English, A1)
/// formula text as char spans.
pub(crate) fn extract_reference_spans(
    formula: &str,
) -> Result<Vec<ReferenceSpan>, Unsupported> {
    let chars: Vec<char> = formula.chars().collect();
    let mut spans = Vec::new();
    for marked in get_tokens(formula) {
        let mut start = marked.start.max(0) as usize;
        let end = marked.end.max(0) as usize;
        if end > chars.len() || start > end {
            return Err(Unsupported("token span out of bounds"));
        }
        // Token spans may include leading whitespace (the lexer marks a token
        // from the end of the previous one); trim it so splices are exact.
        while start < end && chars[start].is_whitespace() {
            start += 1;
        }
        match marked.token {
            TokenType::Reference {
                sheet,
                row,
                column,
                absolute_column,
                absolute_row,
            } => spans.push(ReferenceSpan {
                start,
                end,
                sheet,
                left: CellEndpoint {
                    column,
                    row,
                    absolute_column,
                    absolute_row,
                },
                right: None,
            }),
            TokenType::Range { sheet, left, right } => spans.push(ReferenceSpan {
                start,
                end,
                sheet,
                left: CellEndpoint {
                    column: left.column,
                    row: left.row,
                    absolute_column: left.absolute_column,
                    absolute_row: left.absolute_row,
                },
                right: Some(CellEndpoint {
                    column: right.column,
                    row: right.row,
                    absolute_column: right.absolute_column,
                    absolute_row: right.absolute_row,
                }),
            }),
            TokenType::StructuredReference { .. } => {
                return Err(Unsupported("structured references"));
            }
            TokenType::Illegal(_) => return Err(Unsupported("illegal token")),
            _ => {}
        }
    }
    Ok(spans)
}

/// Resolution interface between the codec and the replicated orders.
pub(crate) trait RefResolver {
    fn sheet_id_by_name(&self, name: &str) -> Option<EntityId>;
    fn sheet_name_by_id(&self, id: EntityId) -> Option<String>;
    fn row_id_at(&self, sheet: EntityId, index: u32) -> Option<EntityId>;
    fn column_id_at(&self, sheet: EntityId, index: u32) -> Option<EntityId>;
    fn resolve_row(&self, sheet: EntityId, id: EntityId) -> ResolvedIndex;
    fn resolve_column(&self, sheet: EntityId, id: EntityId) -> ResolvedIndex;
}

/// Encodes a canonical formula into id-form. `own_sheet` is the sheet the
/// formula lives on (used for sheet-less references, which stay sheet-less:
/// they mean "my sheet" wherever the cell is).
pub(crate) fn encode_formula(
    canonical: &str,
    own_sheet: EntityId,
    resolver: &impl RefResolver,
) -> Result<String, Unsupported> {
    let spans = extract_reference_spans(canonical)?;
    let chars: Vec<char> = canonical.chars().collect();
    let mut out = String::with_capacity(canonical.len() + spans.len() * 8);
    let mut cursor = 0usize;
    for span in &spans {
        push_escaped(&mut out, &chars[cursor..span.start]);
        out.push(REF_DELIM);
        encode_span(&mut out, span, own_sheet, resolver)?;
        out.push(REF_DELIM);
        cursor = span.end;
    }
    push_escaped(&mut out, &chars[cursor..]);
    Ok(out)
}

fn push_escaped(out: &mut String, chars: &[char]) {
    for &c in chars {
        out.push(c);
        if c == REF_DELIM {
            out.push(REF_DELIM);
        }
    }
}

fn encode_span(
    out: &mut String,
    span: &ReferenceSpan,
    own_sheet: EntityId,
    resolver: &impl RefResolver,
) -> Result<(), Unsupported> {
    let sheet = match &span.sheet {
        None => own_sheet,
        Some(name) => {
            let id = resolver
                .sheet_id_by_name(name)
                .ok_or(Unsupported("unknown sheet"))?;
            out.push('s');
            out.push_str(&id.encode());
            out.push(';');
            id
        }
    };
    // Full column (D:D) and full row (5:5) ranges are *pinned*: the engine's
    // displacement skips their spanning axis entirely (see `full_row` /
    // `full_column` in stringify.rs), so those endpoints must not track ids.
    let (pin_rows, pin_columns) = match &span.right {
        Some(right) => (
            span.left.absolute_row
                && right.absolute_row
                && span.left.row == 1
                && right.row == MAX_ROW as i32,
            span.left.absolute_column
                && right.absolute_column
                && span.left.column == 1
                && right.column == MAX_COLUMN as i32,
        ),
        None => (false, false),
    };
    encode_endpoint(out, &span.left, sheet, resolver, pin_rows, pin_columns)?;
    if let Some(right) = &span.right {
        out.push(':');
        encode_endpoint(out, right, sheet, resolver, pin_rows, pin_columns)?;
    }
    Ok(())
}

fn encode_endpoint(
    out: &mut String,
    endpoint: &CellEndpoint,
    sheet: EntityId,
    resolver: &impl RefResolver,
    pin_row: bool,
    pin_column: bool,
) -> Result<(), Unsupported> {
    if endpoint.column < 1 || endpoint.row < 1 {
        return Err(Unsupported("reference outside the grid"));
    }
    if pin_column {
        out.push('p');
        out.push_str(&EntityId::Original(endpoint.column as u32).encode());
    } else {
        let column_id = resolver
            .column_id_at(sheet, endpoint.column as u32)
            .ok_or(Unsupported("column outside the grid"))?;
        out.push(if endpoint.absolute_column { 'a' } else { 'r' });
        out.push_str(&column_id.encode());
    }
    out.push(';');
    if pin_row {
        out.push('p');
        out.push_str(&EntityId::Original(endpoint.row as u32).encode());
    } else {
        let row_id = resolver
            .row_id_at(sheet, endpoint.row as u32)
            .ok_or(Unsupported("row outside the grid"))?;
        out.push(if endpoint.absolute_row { 'a' } else { 'r' });
        out.push_str(&row_id.encode());
    }
    Ok(())
}

/// Renders an id-form formula back to canonical A1 text against the current
/// orders. Unresolvable references become `#REF!`; malformed id tokens are an
/// error (they indicate a bug or corruption, not a merge outcome).
pub(crate) fn render_formula(
    id_form: &str,
    own_sheet: EntityId,
    resolver: &impl RefResolver,
) -> Result<String, String> {
    let chars: Vec<char> = id_form.chars().collect();
    let mut out = String::with_capacity(id_form.len());
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if c != REF_DELIM {
            out.push(c);
            i += 1;
            continue;
        }
        if chars.get(i + 1) == Some(&REF_DELIM) {
            // Escaped literal delimiter.
            out.push(REF_DELIM);
            i += 2;
            continue;
        }
        let close = chars[i + 1..]
            .iter()
            .position(|&c| c == REF_DELIM)
            .ok_or("unterminated reference token")?;
        let payload: String = chars[i + 1..i + 1 + close].iter().collect();
        out.push_str(&render_payload(&payload, own_sheet, resolver)?);
        i += close + 2;
    }
    Ok(out)
}

/// Is the formula stored in id-form (as opposed to a plain-text fallback)?
pub(crate) fn is_id_form(stored: &str) -> bool {
    stored.contains(REF_DELIM)
}

fn render_payload(
    payload: &str,
    own_sheet: EntityId,
    resolver: &impl RefResolver,
) -> Result<String, String> {
    // Optional sheet field.
    let (sheet, prefix, rest) = match payload.strip_prefix('s') {
        Some(rest) => {
            let (sheet_enc, endpoints) = rest
                .split_once(';')
                .ok_or("malformed reference token: missing sheet separator")?;
            let sheet_id = EntityId::decode(sheet_enc)
                .ok_or("malformed reference token: bad sheet id")?;
            match resolver.sheet_name_by_id(sheet_id) {
                Some(name) => (sheet_id, format!("{}!", quote_name(&name)), endpoints),
                // The referenced sheet is gone.
                None => return Ok("#REF!".to_string()),
            }
        }
        None => (own_sheet, String::new(), payload),
    };

    match rest.split_once(':') {
        None => {
            let endpoint = parse_endpoint(rest)?;
            match (
                part_index(sheet, Axis2::Columns, &endpoint.column, resolver),
                part_index(sheet, Axis2::Rows, &endpoint.row, resolver),
            ) {
                (Some(column), Some(row)) => Ok(format!(
                    "{}{}",
                    prefix,
                    cell_text(column.0, row.0, column.1, row.1)?
                )),
                _ => Ok(format!("{prefix}#REF!")),
            }
        }
        Some((left, right)) => {
            let l = parse_endpoint(left)?;
            let r = parse_endpoint(right)?;
            // Pinned axes render in the engine's short form: `D:D` for full
            // columns (row parts omitted), `5:9` for full rows.
            let rows_pinned =
                matches!(l.row, Part::Pinned(_)) && matches!(r.row, Part::Pinned(_));
            let columns_pinned =
                matches!(l.column, Part::Pinned(_)) && matches!(r.column, Part::Pinned(_));
            if rows_pinned && !columns_pinned {
                let left = part_index(sheet, Axis2::Columns, &l.column, resolver);
                let right = part_index(sheet, Axis2::Columns, &r.column, resolver);
                let (left, right) = normalize_pair(left, right);
                let left_text = axis_text(Axis2::Columns, left)?;
                let right_text = axis_text(Axis2::Columns, right)?;
                return Ok(format!("{prefix}{left_text}:{right_text}"));
            }
            if columns_pinned && !rows_pinned {
                let left = part_index(sheet, Axis2::Rows, &l.row, resolver);
                let right = part_index(sheet, Axis2::Rows, &r.row, resolver);
                let (left, right) = normalize_pair(left, right);
                let left_text = axis_text(Axis2::Rows, left)?;
                let right_text = axis_text(Axis2::Rows, right)?;
                return Ok(format!("{prefix}{left_text}:{right_text}"));
            }
            // Resolve all four parts; when both endpoints are alive the
            // engine normalizes crossed ranges per axis, moving the absolute
            // flag together with its coordinate ($D$5:B9 → B$5:$D9). A dead
            // endpoint renders as #REF! with no normalization.
            let lc = part_index(sheet, Axis2::Columns, &l.column, resolver);
            let lr = part_index(sheet, Axis2::Rows, &l.row, resolver);
            let rc = part_index(sheet, Axis2::Columns, &r.column, resolver);
            let rr = part_index(sheet, Axis2::Rows, &r.row, resolver);
            match (lc, lr, rc, rr) {
                (Some(c1), Some(r1), Some(c2), Some(r2)) => {
                    let (c1, c2) = if c1.0 > c2.0 { (c2, c1) } else { (c1, c2) };
                    let (r1, r2) = if r1.0 > r2.0 { (r2, r1) } else { (r1, r2) };
                    Ok(format!(
                        "{}{}:{}",
                        prefix,
                        cell_text(c1.0, r1.0, c1.1, r1.1)?,
                        cell_text(c2.0, r2.0, c2.1, r2.1)?
                    ))
                }
                _ => Ok(format!(
                    "{}{}:{}",
                    prefix,
                    endpoint_fragment(lc, lr)?,
                    endpoint_fragment(rc, rr)?
                )),
            }
        }
    }
}

/// A dead row or column makes an endpoint `#REF!` — matching the engine's
/// own displacement semantics (`displace_cells`): deleting an endpoint
/// row/column turns *that endpoint* into `#REF!` (`=SUM(A1:#REF!)`), it does
/// not clamp the range as Excel does. Interior deletions shrink the range
/// automatically here, because endpoints keep their ids and only their
/// rendered indices change.
fn endpoint_fragment(
    column: Option<(u32, bool)>,
    row: Option<(u32, bool)>,
) -> Result<String, String> {
    match (column, row) {
        (Some(c), Some(r)) => cell_text(c.0, r.0, c.1, r.1),
        _ => Ok("#REF!".to_string()),
    }
}

/// Engine range normalization: a live crossed pair swaps (the absolute flag
/// travels with its coordinate); dead endpoints stay put.
fn normalize_pair(
    left: Option<IndexedPart>,
    right: Option<IndexedPart>,
) -> (Option<IndexedPart>, Option<IndexedPart>) {
    match (left, right) {
        (Some(l), Some(r)) if l.0 > r.0 => (Some(r), Some(l)),
        other => other,
    }
}

/// Renders one side of a pinned-axis (full) range: a column name or a row
/// number, `#REF!` when dead.
fn axis_text(axis: Axis2, part: Option<(u32, bool)>) -> Result<String, String> {
    match part {
        None => Ok("#REF!".to_string()),
        Some((index, absolute)) => {
            let dollar = if absolute { "$" } else { "" };
            match axis {
                Axis2::Rows => Ok(format!("{dollar}{index}")),
                Axis2::Columns => {
                    let name = number_to_column(index as i32)
                        .ok_or_else(|| "column out of range".to_string())?;
                    Ok(format!("{dollar}{name}"))
                }
            }
        }
    }
}

/// Internal axis selector (the projection's `Axis` lives a module up).
#[derive(Clone, Copy)]
enum Axis2 {
    Rows,
    Columns,
}

/// A resolved part: `(display index, absolute flag)`.
type IndexedPart = (u32, bool);

/// One side (row or column) of an endpoint.
#[derive(Debug, PartialEq)]
enum Part {
    Id { id: EntityId, absolute: bool },
    /// A literal index that never tracks structural edits (full ranges).
    Pinned(u32),
}

#[derive(Debug, PartialEq)]
struct Endpoint {
    column: Part,
    row: Part,
}

/// The current display index of a part, or `None` if it is dead. Overflowed
/// parts (shifted past the grid) keep their literal rank — the engine renders
/// them that way too (`=A1048577`).
fn part_index(
    sheet: EntityId,
    axis: Axis2,
    part: &Part,
    resolver: &impl RefResolver,
) -> Option<(u32, bool)> {
    match part {
        Part::Pinned(index) => Some((*index, false)),
        Part::Id { id, absolute } => {
            let resolved = match axis {
                Axis2::Rows => resolver.resolve_row(sheet, *id),
                Axis2::Columns => resolver.resolve_column(sheet, *id),
            };
            match resolved {
                ResolvedIndex::Visible(index) | ResolvedIndex::Overflow(index) => {
                    Some((index, *absolute))
                }
                ResolvedIndex::Gone { .. } | ResolvedIndex::Unknown => None,
            }
        }
    }
}

fn parse_endpoint(text: &str) -> Result<Endpoint, String> {
    let (column, row) = text
        .split_once(';')
        .ok_or("malformed reference token: missing endpoint separator")?;
    Ok(Endpoint {
        column: parse_part(column)?,
        row: parse_part(row)?,
    })
}

fn parse_part(part: &str) -> Result<Part, String> {
    let mut chars = part.chars();
    let flag = chars.next().ok_or("malformed reference token: empty part")?;
    let body = chars.as_str();
    match flag {
        'a' | 'r' => Ok(Part::Id {
            id: EntityId::decode(body).ok_or("malformed reference token: bad entity id")?,
            absolute: flag == 'a',
        }),
        'p' => match EntityId::decode(body) {
            Some(EntityId::Original(index)) => Ok(Part::Pinned(index)),
            _ => Err("malformed reference token: bad pinned index".to_string()),
        },
        _ => Err("malformed reference token: bad flag".to_string()),
    }
}

/// Does any reference in this id-form formula currently resolve past the end
/// of the grid? Such formulas are demoted to plain text by the caller: the
/// engine renders them as out-of-grid identifiers (`=A1048577`) which freeze
/// (identifiers are never displaced), so an id-token would wrongly keep
/// tracking structural changes.
pub(crate) fn has_overflow_refs(
    id_form: &str,
    own_sheet: EntityId,
    resolver: &impl RefResolver,
) -> bool {
    let chars: Vec<char> = id_form.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] != REF_DELIM {
            i += 1;
            continue;
        }
        if chars.get(i + 1) == Some(&REF_DELIM) {
            i += 2;
            continue;
        }
        let Some(close) = chars[i + 1..].iter().position(|&c| c == REF_DELIM) else {
            return false;
        };
        let payload: String = chars[i + 1..i + 1 + close].iter().collect();
        i += close + 2;

        let (sheet, rest) = match payload.strip_prefix('s') {
            Some(rest) => match rest.split_once(';') {
                Some((enc, endpoints)) => match EntityId::decode(enc) {
                    Some(id) => (id, endpoints),
                    None => continue,
                },
                None => continue,
            },
            None => (own_sheet, payload.as_str()),
        };
        for endpoint_text in rest.split(':') {
            let Ok(endpoint) = parse_endpoint(endpoint_text) else {
                continue;
            };
            for (axis, part) in [(Axis2::Columns, &endpoint.column), (Axis2::Rows, &endpoint.row)]
            {
                if let Part::Id { id, .. } = part {
                    let resolved = match axis {
                        Axis2::Rows => resolver.resolve_row(sheet, *id),
                        Axis2::Columns => resolver.resolve_column(sheet, *id),
                    };
                    if matches!(resolved, ResolvedIndex::Overflow(_)) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn cell_text(column: u32, row: u32, abs_column: bool, abs_row: bool) -> Result<String, String> {
    let column_name =
        number_to_column(column as i32).ok_or_else(|| "column out of range".to_string())?;
    Ok(format!(
        "{}{}{}{}",
        if abs_column { "$" } else { "" },
        column_name,
        if abs_row { "$" } else { "" },
        row
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt::order::AxisOrder;
    use std::collections::BTreeMap;

    const S0: EntityId = EntityId::Original(0);
    const S1: EntityId = EntityId::Original(1);

    struct TestResolver {
        sheets: Vec<(String, EntityId)>,
        rows: BTreeMap<EntityId, AxisOrder>,
        cols: BTreeMap<EntityId, AxisOrder>,
    }

    impl TestResolver {
        fn pristine() -> TestResolver {
            let mut resolver = TestResolver {
                sheets: vec![
                    ("Sheet1".to_string(), S0),
                    ("My Sheet".to_string(), S1),
                ],
                rows: BTreeMap::new(),
                cols: BTreeMap::new(),
            };
            for id in [S0, S1] {
                resolver.rows.insert(id, AxisOrder::new(1_048_576, Vec::new()));
                resolver.cols.insert(id, AxisOrder::new(16_384, Vec::new()));
            }
            resolver
        }
    }

    impl RefResolver for TestResolver {
        fn sheet_id_by_name(&self, name: &str) -> Option<EntityId> {
            self.sheets
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, id)| *id)
        }
        fn sheet_name_by_id(&self, id: EntityId) -> Option<String> {
            self.sheets
                .iter()
                .find(|(_, i)| *i == id)
                .map(|(n, _)| n.clone())
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

    fn round_trip(resolver: &TestResolver, formula: &str) -> String {
        let encoded = encode_formula(formula, S0, resolver).unwrap();
        render_formula(&encoded, S0, resolver).unwrap()
    }

    #[test]
    fn extraction_finds_references_and_ranges() {
        let spans = extract_reference_spans("=A1+SUM($B$2:C3)").unwrap();
        assert_eq!(spans.len(), 2);
        assert_eq!((spans[0].start, spans[0].end), (1, 3));
        assert_eq!(spans[0].left.column, 1);
        assert_eq!(spans[0].left.row, 1);
        assert!(spans[0].right.is_none());
        let range = &spans[1];
        assert_eq!(range.left.column, 2);
        assert!(range.left.absolute_column && range.left.absolute_row);
        assert_eq!(range.right.unwrap().column, 3);
    }

    #[test]
    fn extraction_handles_sheet_prefixes_and_strings() {
        let spans =
            extract_reference_spans("=IF('My Sheet'!A1>2,\"A1 is not a ref\",Sheet1!B2)").unwrap();
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].sheet.as_deref(), Some("My Sheet"));
        assert_eq!(spans[1].sheet.as_deref(), Some("Sheet1"));
    }

    #[test]
    fn extraction_ignores_names_errors_and_literals() {
        // Defined names, error literals and array literals carry no spans.
        let spans = extract_reference_spans("=MyName+{1,2;3}+#REF!+\"txt\"").unwrap();
        assert!(spans.is_empty());
    }

    #[test]
    fn extraction_rejects_structured_references() {
        assert!(extract_reference_spans("=Table1[Column]").is_err());
    }

    #[test]
    fn encode_render_round_trip_is_identity_on_pristine_orders() {
        let resolver = TestResolver::pristine();
        for formula in [
            "=A1+B2",
            "=SUM($A$1:B$5)*2",
            "='My Sheet'!C3",
            "='My Sheet'!A1:B2",
            "=IF(A1>2,\"x\",B2)",
            "=MyName+#REF!+1",
        ] {
            assert_eq!(round_trip(&resolver, formula), formula, "{formula}");
        }
    }

    #[test]
    fn encoded_form_is_stable_under_inserts_render_shifts() {
        let mut resolver = TestResolver::pristine();
        let encoded = encode_formula("=A5+$B$2", S0, &resolver).unwrap();

        // Insert a row before row 3 and a column before column 1.
        let rows = resolver.rows.get_mut(&S0).unwrap();
        let (lo, hi) = rows.insert_bounds(3);
        rows.insert(
            EntityId::Inserted { client: 9, counter: 1 },
            crate::crdt::order::between(lo.as_deref(), hi.as_deref()),
        );
        let cols = resolver.cols.get_mut(&S0).unwrap();
        let (lo, hi) = cols.insert_bounds(1);
        cols.insert(
            EntityId::Inserted { client: 9, counter: 2 },
            crate::crdt::order::between(lo.as_deref(), hi.as_deref()),
        );

        // The stored form did not change; only the rendering does.
        assert_eq!(
            render_formula(&encoded, S0, &resolver).unwrap(),
            "=B6+$C$2"
        );
    }

    #[test]
    fn deleted_single_reference_renders_ref_error() {
        let mut resolver = TestResolver::pristine();
        let encoded = encode_formula("=A5*2", S0, &resolver).unwrap();
        resolver
            .rows
            .get_mut(&S0)
            .unwrap()
            .remove(EntityId::Original(5));
        assert_eq!(render_formula(&encoded, S0, &resolver).unwrap(), "=#REF!*2");
    }

    #[test]
    fn range_endpoint_deletion_matches_engine_semantics() {
        // The engine's displace_cells turns a deleted *endpoint* into #REF!
        // (=SUM(A1:#REF!)) and shrinks ranges only on interior deletions; the
        // codec must render identically so both replicas agree.
        let mut resolver = TestResolver::pristine();
        let encoded = encode_formula("=SUM(A2:A6)", S0, &resolver).unwrap();

        // Deleting the top endpoint kills that endpoint only.
        resolver
            .rows
            .get_mut(&S0)
            .unwrap()
            .remove(EntityId::Original(2));
        assert_eq!(
            render_formula(&encoded, S0, &resolver).unwrap(),
            "=SUM(#REF!:A5)"
        );

        // Deleting the bottom endpoint too kills the whole range.
        resolver
            .rows
            .get_mut(&S0)
            .unwrap()
            .remove(EntityId::Original(6));
        assert_eq!(
            render_formula(&encoded, S0, &resolver).unwrap(),
            "=SUM(#REF!:#REF!)"
        );

        // An interior deletion shrinks the span but keeps the endpoints.
        let mut resolver2 = TestResolver::pristine();
        let encoded2 = encode_formula("=SUM(A2:A6)", S0, &resolver2).unwrap();
        resolver2
            .rows
            .get_mut(&S0)
            .unwrap()
            .remove(EntityId::Original(4));
        assert_eq!(
            render_formula(&encoded2, S0, &resolver2).unwrap(),
            "=SUM(A2:A5)"
        );
    }

    #[test]
    fn single_cell_range_deletion_renders_dead_range() {
        let mut resolver = TestResolver::pristine();
        let encoded = encode_formula("=SUM(B3:B3)", S0, &resolver).unwrap();
        resolver
            .rows
            .get_mut(&S0)
            .unwrap()
            .remove(EntityId::Original(3));
        assert_eq!(
            render_formula(&encoded, S0, &resolver).unwrap(),
            "=SUM(#REF!:#REF!)"
        );
    }

    #[test]
    fn deleted_sheet_renders_ref_error() {
        let mut resolver = TestResolver::pristine();
        let encoded = encode_formula("='My Sheet'!B2+1", S0, &resolver).unwrap();
        resolver.sheets.retain(|(_, id)| *id != S1);
        assert_eq!(
            render_formula(&encoded, S0, &resolver).unwrap(),
            "=#REF!+1"
        );
    }

    #[test]
    fn renamed_sheet_renders_new_name() {
        let mut resolver = TestResolver::pristine();
        let encoded = encode_formula("='My Sheet'!B2", S0, &resolver).unwrap();
        resolver.sheets[1].0 = "Data".to_string();
        assert_eq!(render_formula(&encoded, S0, &resolver).unwrap(), "=Data!B2");
        // And a rename that needs quoting again.
        resolver.sheets[1].0 = "Data 2026".to_string();
        assert_eq!(
            render_formula(&encoded, S0, &resolver).unwrap(),
            "='Data 2026'!B2"
        );
    }

    #[test]
    fn full_ranges_round_trip_and_are_pinned() {
        let resolver = TestResolver::pristine();
        assert_eq!(round_trip(&resolver, "=SUM(D:D)"), "=SUM(D:D)");
        assert_eq!(round_trip(&resolver, "=SUM($D:$F)"), "=SUM($D:$F)");
        assert_eq!(round_trip(&resolver, "=SUM(5:9)"), "=SUM(5:9)");
    }

    #[test]
    fn full_column_range_ignores_row_edits_but_tracks_column_edits() {
        // The engine's displacement skips the spanning axis of a full range
        // (`full_row` in stringify.rs): D:D is unaffected by row inserts and
        // deletes, but shifts with column inserts.
        let mut resolver = TestResolver::pristine();
        let encoded = encode_formula("=SUM(D:D)", S0, &resolver).unwrap();

        let rows = resolver.rows.get_mut(&S0).unwrap();
        let (lo, hi) = rows.insert_bounds(1);
        rows.insert(
            EntityId::Inserted { client: 9, counter: 1 },
            crate::crdt::order::between(lo.as_deref(), hi.as_deref()),
        );
        rows.remove(EntityId::Original(5));
        assert_eq!(render_formula(&encoded, S0, &resolver).unwrap(), "=SUM(D:D)");

        let cols = resolver.cols.get_mut(&S0).unwrap();
        let (lo, hi) = cols.insert_bounds(1);
        cols.insert(
            EntityId::Inserted { client: 9, counter: 2 },
            crate::crdt::order::between(lo.as_deref(), hi.as_deref()),
        );
        assert_eq!(render_formula(&encoded, S0, &resolver).unwrap(), "=SUM(E:E)");
    }

    #[test]
    fn overflowed_reference_renders_out_of_grid_index() {
        // The engine displaces near-edge references past the grid and renders
        // the literal index (`=A1048577`, an identifier that then freezes).
        let mut resolver = TestResolver::pristine();
        let encoded = encode_formula("=A1048576", S0, &resolver).unwrap();
        let rows = resolver.rows.get_mut(&S0).unwrap();
        let (lo, hi) = rows.insert_bounds(3);
        rows.insert(
            EntityId::Inserted { client: 9, counter: 1 },
            crate::crdt::order::between(lo.as_deref(), hi.as_deref()),
        );
        assert_eq!(render_formula(&encoded, S0, &resolver).unwrap(), "=A1048577");
        // The overflow scan flags the formula for demotion to plain text.
        assert!(has_overflow_refs(&encoded, S0, &resolver));
        let healthy = encode_formula("=A5", S0, &resolver).unwrap();
        assert!(!has_overflow_refs(&healthy, S0, &resolver));
    }

    #[test]
    fn delimiter_in_string_literal_survives() {
        let resolver = TestResolver::pristine();
        let tricky = format!("=A1&\"x{}y\"", REF_DELIM);
        assert_eq!(round_trip(&resolver, &tricky), tricky);
    }

    #[test]
    fn unknown_sheet_reference_is_unsupported() {
        let resolver = TestResolver::pristine();
        assert!(encode_formula("=Missing!A1", S0, &resolver).is_err());
    }

    #[test]
    fn is_id_form_detects_encoded_formulas() {
        let resolver = TestResolver::pristine();
        assert!(is_id_form(&encode_formula("=A1", S0, &resolver).unwrap()));
        assert!(!is_id_form("=A1"));
        assert!(!is_id_form("plain value"));
    }

    #[test]
    fn resolve_reports_gone_rank() {
        let mut order = AxisOrder::new(100, Vec::new());
        order.remove(EntityId::Original(5));
        assert_eq!(
            order.resolve(EntityId::Original(5)),
            ResolvedIndex::Gone { rank: 5 }
        );
        assert_eq!(
            order.resolve(EntityId::Original(6)),
            ResolvedIndex::Visible(5)
        );
        assert_eq!(
            order.resolve(EntityId::Inserted { client: 1, counter: 1 }),
            ResolvedIndex::Unknown
        );
    }
}
