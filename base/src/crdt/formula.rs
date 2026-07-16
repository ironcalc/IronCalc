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

use super::ids::EntityId;
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
    encode_endpoint(out, &span.left, sheet, resolver)?;
    if let Some(right) = &span.right {
        out.push(':');
        encode_endpoint(out, right, sheet, resolver)?;
    }
    Ok(())
}

fn encode_endpoint(
    out: &mut String,
    endpoint: &CellEndpoint,
    sheet: EntityId,
    resolver: &impl RefResolver,
) -> Result<(), Unsupported> {
    if endpoint.column < 1 || endpoint.row < 1 {
        return Err(Unsupported("reference outside the grid"));
    }
    let column_id = resolver
        .column_id_at(sheet, endpoint.column as u32)
        .ok_or(Unsupported("column outside the grid"))?;
    let row_id = resolver
        .row_id_at(sheet, endpoint.row as u32)
        .ok_or(Unsupported("row outside the grid"))?;
    out.push(if endpoint.absolute_column { 'a' } else { 'r' });
    out.push_str(&column_id.encode());
    out.push(';');
    out.push(if endpoint.absolute_row { 'a' } else { 'r' });
    out.push_str(&row_id.encode());
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
                resolver.resolve_column(sheet, endpoint.0),
                resolver.resolve_row(sheet, endpoint.2),
            ) {
                (ResolvedIndex::Visible(column), ResolvedIndex::Visible(row)) => Ok(format!(
                    "{}{}",
                    prefix,
                    cell_text(column, row, endpoint.1, endpoint.3)?
                )),
                _ => Ok(format!("{prefix}#REF!")),
            }
        }
        Some((left, right)) => {
            let l = parse_endpoint(left)?;
            let r = parse_endpoint(right)?;
            let columns = clamp_axis(
                resolver.resolve_column(sheet, l.0),
                resolver.resolve_column(sheet, r.0),
            );
            let rows = clamp_axis(
                resolver.resolve_row(sheet, l.2),
                resolver.resolve_row(sheet, r.2),
            );
            match (columns, rows) {
                (Some((c1, c2)), Some((r1, r2))) => Ok(format!(
                    "{}{}:{}",
                    prefix,
                    cell_text(c1, r1, l.1, l.3)?,
                    cell_text(c2, r2, r.1, r.3)?
                )),
                _ => Ok(format!("{prefix}#REF!")),
            }
        }
    }
}

/// `(column_id, abs_column, row_id, abs_row)`
fn parse_endpoint(text: &str) -> Result<(EntityId, bool, EntityId, bool), String> {
    let (column, row) = text
        .split_once(';')
        .ok_or("malformed reference token: missing endpoint separator")?;
    let parse_part = |part: &str| -> Result<(EntityId, bool), String> {
        let mut chars = part.chars();
        let absolute = match chars.next() {
            Some('a') => true,
            Some('r') => false,
            _ => return Err("malformed reference token: bad flag".to_string()),
        };
        let id = EntityId::decode(chars.as_str())
            .ok_or("malformed reference token: bad entity id")?;
        Ok((id, absolute))
    };
    let (column_id, abs_column) = parse_part(column)?;
    let (row_id, abs_row) = parse_part(row)?;
    Ok((column_id, abs_column, row_id, abs_row))
}

/// Clamps a range's two endpoints on one axis. A tombstoned start endpoint
/// moves to the first visible element after it; a tombstoned end endpoint to
/// the last visible element before it. `None` = the range collapsed.
fn clamp_axis(start: ResolvedIndex, end: ResolvedIndex) -> Option<(u32, u32)> {
    let s = match start {
        ResolvedIndex::Visible(i) => i,
        ResolvedIndex::Gone { rank } => rank,
        ResolvedIndex::Unknown => return None,
    };
    let e = match end {
        ResolvedIndex::Visible(i) => i,
        ResolvedIndex::Gone { rank } => rank.checked_sub(1)?,
        ResolvedIndex::Unknown => return None,
    };
    (s >= 1 && s <= e).then_some((s, e))
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
    fn range_endpoints_clamp_inward() {
        let mut resolver = TestResolver::pristine();
        let encoded = encode_formula("=SUM(A2:A6)", S0, &resolver).unwrap();

        // Deleting the top endpoint clamps the range start down to row 3
        // (which then displays as row 2).
        resolver
            .rows
            .get_mut(&S0)
            .unwrap()
            .remove(EntityId::Original(2));
        assert_eq!(
            render_formula(&encoded, S0, &resolver).unwrap(),
            "=SUM(A2:A5)"
        );

        // Deleting the bottom endpoint clamps the range end up.
        resolver
            .rows
            .get_mut(&S0)
            .unwrap()
            .remove(EntityId::Original(6));
        assert_eq!(
            render_formula(&encoded, S0, &resolver).unwrap(),
            "=SUM(A2:A4)"
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
    fn collapsed_range_renders_ref_error() {
        let mut resolver = TestResolver::pristine();
        let encoded = encode_formula("=SUM(B3:B3)", S0, &resolver).unwrap();
        resolver
            .rows
            .get_mut(&S0)
            .unwrap()
            .remove(EntityId::Original(3));
        assert_eq!(
            render_formula(&encoded, S0, &resolver).unwrap(),
            "=SUM(#REF!)"
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
    fn full_column_range_round_trips_semantically() {
        // D:D lexes as D1:D1048576 with absolute rows; the rendering is the
        // explicit form, which parses back to the same range.
        let resolver = TestResolver::pristine();
        assert_eq!(round_trip(&resolver, "=SUM(D:D)"), "=SUM(D$1:D$1048576)");
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
