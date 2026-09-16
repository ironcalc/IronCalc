use std::collections::HashMap;

use crate::cf_types::ConditionalFormatting;
use crate::collab::bind::MintPlan;
use crate::collab::fractional_index::{
    virtual_key, FractionalIndex, FractionalKey, KeyBuf, SESSION_SUFFIX_LEN,
};
use crate::collab::log::SessionId;
use crate::collab::model::{CollabModel, Stable, StableRange};
use crate::collab::patch::{
    CellInput, ColState, ConditionalFormatState, Patch, RowState, SheetContent, SheetIndexSeed,
};
use crate::constants::LAST_COLUMN;
use crate::expressions::lexer::LexerMode;
use crate::expressions::parser::stringify::to_rc_format;
use crate::expressions::parser::Node;
use crate::expressions::types::CellReferenceRC;
use crate::language::get_default_language;
use crate::locale::get_default_locale;
use crate::types::{
    Cell, Col, Comment, Link, MergedCell, Ordinal, Position, RangeRef, Row, SheetData,
    StyleIncludes, Styles, Workbook, Worksheet,
};

/// How far the sheet reaches on each axis.
pub(crate) fn used_extent(ws: &Worksheet) -> (i32, i32) {
    let (mut rows, mut cols) = (0, 0);
    for (r, row) in &ws.sheet_data {
        rows = rows.max(*r);
        for c in row.keys() {
            cols = cols.max(*c);
        }
    }
    for r in &ws.rows {
        rows = rows.max(r.r);
    }
    for c in &ws.cols {
        cols = cols.max(c.max);
    }
    for m in &ws.merged_cells {
        rows = rows.max(m.last_row());
        cols = cols.max(m.last_column());
    }
    for range in ws.conditional_formatting.iter().flat_map(|cf| &cf.ranges) {
        if let Some((_, hi)) = range.rows {
            rows = rows.max(hi);
        }
        if let Some((_, hi)) = range.cols {
            cols = cols.max(hi);
        }
    }
    for c in &ws.comments {
        rows = rows.max(c.cell_ref.0);
        cols = cols.max(c.cell_ref.1);
    }
    (rows, cols)
}

/// The seed payload an `AddSheet` carries for `ws`.
pub(crate) fn content_from_ordinal(
    ws: &Worksheet,
    styles: &Styles,
    shared_strings: &[String],
    suffix: [u8; SESSION_SUFFIX_LEN],
) -> SheetContent {
    let (virtual_rows, virtual_columns) = used_extent(ws);
    let range = |r: &RangeRef| StableRange {
        rows: r
            .rows
            .map(|(lo, hi)| (virtual_key(lo as u32), virtual_key(hi as u32))),
        cols: r
            .cols
            .map(|(lo, hi)| (virtual_key(lo as u32), virtual_key(hi as u32))),
    };

    let mut cell_values = Vec::new();
    let mut cell_styles = Vec::new();
    for (r, row) in &ws.sheet_data {
        for (c, cell) in row {
            let at = (virtual_key(*r as u32), virtual_key(*c as u32));
            let input = match cell {
                Cell::NumberCell { v, .. } => Some(CellInput::Number(*v)),
                Cell::BooleanCell { v, .. } => Some(CellInput::Boolean(*v)),
                Cell::ErrorCell { ei, .. } => Some(CellInput::Error(ei.clone())),
                Cell::SharedString { si, .. } => shared_strings
                    .get(*si as usize)
                    .cloned()
                    .map(CellInput::Text),
                // Formulas come in the second pass; arrays are skipped and spills are derived.
                _ => None,
            };
            if let Some(input) = input {
                cell_values.push((at.clone(), input));
            }
            // Only a cell holding a style of its own: 0 is the table's default entry.
            if cell.get_style() != 0 {
                if let Ok(style) = styles.get_style(cell.get_style()) {
                    cell_styles.push((at, style));
                }
            }
        }
    }
    // same as `add_conditional_formatting` would work like
    fn cf_key(at: usize, suffix: &[u8]) -> FractionalKey {
        let mut buf = KeyBuf::from(&(2 * (at as u32 + 1)).to_be_bytes()[1..]);
        buf.extend_from_slice(suffix);
        FractionalKey::try_from_bytes(&buf).unwrap()
    }

    SheetContent {
        state: ws.state.clone(),
        color: ws.color.clone(),
        show_grid_lines: ws.show_grid_lines,
        frozen_rows: ws.frozen_rows,
        frozen_columns: ws.frozen_columns,
        index: SheetIndexSeed::Extent {
            rows: virtual_rows.max(0) as u32,
            columns: virtual_columns.max(0) as u32,
        },
        rows: ws
            .rows
            .iter()
            .map(|row| {
                let state = RowState {
                    height: row.height,
                    hidden: row.hidden,
                    style: match row.custom_format {
                        true => styles.get_style(row.s).ok().map(Box::new),
                        false => None,
                    },
                    custom_height: row.custom_height,
                    custom_format: row.custom_format,
                };
                (virtual_key(row.r as u32), state)
            })
            .collect(),
        columns: ws
            .cols
            .iter()
            .map(|col| {
                let state = ColState {
                    width: col.width,
                    hidden: col.hidden,
                    style: col
                        .style
                        .and_then(|s| styles.get_style(s).ok())
                        .map(Box::new),
                    custom_width: col.custom_width,
                };
                (
                    (virtual_key(col.min as u32), virtual_key(col.max as u32)),
                    state,
                )
            })
            .collect(),
        cell_values,
        cell_styles,
        merge_cells: ws
            .merged_cells
            .iter()
            .map(|m| range(&RangeRef::from(m)))
            .collect(),
        links: Vec::new(), // links come in the second pass
        comments: ws
            .comments
            .iter()
            .map(|c| Comment {
                text: c.text.clone(),
                author_name: c.author_name.clone(),
                author_id: c.author_id.clone(),
                cell_ref: (
                    virtual_key(c.cell_ref.0 as u32),
                    virtual_key(c.cell_ref.1 as u32),
                ),
            })
            .collect(),
        conditional_formatting: ws
            .conditional_formatting
            .iter()
            .enumerate()
            .map(|(at, cf)| {
                let state = ConditionalFormatState {
                    rule: cf.cf_rule.clone(),
                    ranges: cf.ranges.iter().map(&range).collect(),
                };
                (cf_key(at, &suffix), state)
            })
            .collect(),
    }
}

impl CollabModel<'static> {
    /// Imports an ordinal workbook as a replica's whole history: the returned model carries the
    /// import in its pending commits, so [`flush`](CollabModel::flush) ships it to peers.
    pub fn from_workbook_with_session(
        workbook: Workbook,
        language_id: &str,
        session: SessionId,
    ) -> Result<Self, String> {
        let mut model = CollabModel::new(session);
        // A local reading preference, as in `new_empty_with_session`; not replicated.
        let language = crate::language::get_language(language_id)
            .map_err(|_| format!("Invalid language: {language_id}"))?;
        model.language = language;
        model.parser.set_language(language);
        // Tables are not replicated yet, so they are left behind here.
        // A conditional formatting rule names its format by index into the local `dxfs` table, the
        // way one added through `add_conditional_formatting` does, so that table comes along.
        model.workbook.styles.dxfs = workbook.styles.dxfs.clone();

        model.set_name(&workbook.name);
        model.set_locale(&workbook.settings.locale)?;
        model.set_timezone(&workbook.settings.tz)?;
        model.set_theme(workbook.theme.clone());

        // One commit per sheet, in file order, so each position is minted after the previous sheet
        // exists. The file's own ids are kept where they can be: two replicas importing the same
        // file then agree on the sheets as well as on the cells.
        let suffix = model.suffix();
        for ws in &workbook.worksheets {
            let id = match ws.sheet_id {
                id if id != 0 && !model.workbook.meta.sheet_existence.contains_key(&id) => id,
                _ => model.new_sheet_id(),
            };
            let content =
                content_from_ordinal(ws, &workbook.styles, &workbook.shared_strings, suffix);
            let position = model.sheet_position();
            model.commit_local(vec![Patch::AddSheet {
                id,
                name: ws.name.clone(),
                position,
                content: Some(Box::new(content)),
            }]);
        }

        // Before the formulas: a formula naming one then binds to the name's id, which survives a
        // later rename of the name.
        for name in &workbook.defined_names {
            let scope = match name.sheet_id {
                Some(id) => match workbook.worksheets.iter().position(|ws| ws.sheet_id == id) {
                    Some(at) => Some(at as u32),
                    None => continue,
                },
                None => None,
            };
            model.new_defined_name(&name.name, scope, &name.formula)?;
        }

        for (i, ws) in workbook.worksheets.iter().enumerate() {
            let sheet = i as u32;
            let id = model.workbook.worksheets[i].sheet_id;
            let mut plan = MintPlan::default();
            let mut writes = Vec::new();
            // Array formulas are skipped: there is no emitter for them yet.
            let formulas = ws.sheet_data.iter().flat_map(|(&row, cells)| {
                cells.iter().filter_map(move |(&column, cell)| match cell {
                    Cell::CellFormula { f, .. } => Some((row, column, *f)),
                    _ => None,
                })
            });
            for (row, column, f) in formulas {
                let Some(text) = ws.shared_formulas.get(f as usize).cloned() else {
                    continue;
                };
                let node = model.parse_rc(i, row, column, &text);
                let bound = model
                    .bind_formula(&node, sheet, row, column, &mut plan)
                    .map_err(|err| format!("Invalid formula \"{text}\": {err}"))?;
                writes.push(Patch::SetCellValue {
                    sheet: id,
                    at: (virtual_key(row as u32), virtual_key(column as u32)),
                    value: Some(CellInput::Formula(bound)),
                    ts: None,
                    prev: Box::new(None),
                });
            }
            // links bind in the same pass: by now referenced sheets should exist
            for (&(row, column), link) in &ws.links {
                let bound = model.bind_link(link.clone(), &mut plan)?;
                writes.push(Patch::SetCellLink {
                    sheet: id,
                    at: (virtual_key(row as u32), virtual_key(column as u32)),
                    link: Some(bound),
                    prev: None,
                });
            }
            if !writes.is_empty() {
                // Whatever the formulas name has to exist before the writes that name it.
                let mut patches = model.mint_patches(&plan);
                patches.extend(writes);
                model.commit_local(patches);
            }
        }

        for named in &workbook.styles.cell_styles {
            if workbook.styles.is_builtin_style(&named.name) {
                continue;
            }
            let Ok(style) = workbook.styles.get_style_by_name(&named.name) else {
                continue;
            };
            model.create_named_style(&named.name, &style, StyleIncludes::default())?;
        }

        model.evaluate();
        Ok(model)
    }
}

impl CollabModel<'_> {
    /// Parses a stored formula — R1C1, English, no leading `=` — as if authored in `(row, column)`
    /// of worksheet `i`, the way [`Model::parse_formulas`](crate::Model) reads the same table.
    fn parse_rc(&mut self, i: usize, row: i32, column: i32, formula: &str) -> Node {
        let context = CellReferenceRC {
            sheet: self.workbook.worksheets[i].get_name(),
            row,
            column,
        };
        let (locale, language) = (self.locale, self.language);
        self.parser.set_locale(get_default_locale());
        self.parser.set_language(get_default_language());
        self.parser.set_lexer_mode(LexerMode::R1C1);
        let node = self.parser.parse(formula, &context);
        self.parser.set_lexer_mode(LexerMode::A1);
        self.parser.set_locale(locale);
        self.parser.set_language(language);
        node
    }

    /// The document as an ordinal workbook: what the xlsx exporter reads, through
    /// [`Model::from_workbook`](crate::Model::from_workbook).
    ///
    /// A full copy taken at export time. Cells whose row or column was deleted are not in it — their
    /// keys resolve to nothing — and neither are tables, which are not replicated.
    pub fn to_ordinal_workbook(&self) -> Workbook {
        let worksheets = self
            .workbook
            .worksheets
            .iter()
            .enumerate()
            .map(|(at, ws)| self.project_sheet(at, ws))
            .collect();
        Workbook {
            shared_strings: self.workbook.shared_strings.clone(),
            defined_names: self.workbook.defined_names.clone(),
            worksheets,
            styles: self.workbook.styles.clone(),
            name: self.workbook.name.clone(),
            settings: self.workbook.settings.clone(),
            metadata: self.workbook.metadata.clone(),
            tables: HashMap::new(),
            views: self.workbook.views.clone(),
            theme: self.workbook.theme.clone(),
            meta: (),
        }
    }

    fn project_sheet(&self, at: usize, ws: &Worksheet<Stable>) -> Worksheet {
        /// Map fractional index keys back to their ordinal position.
        fn ordinals(axis: &FractionalIndex) -> HashMap<FractionalKey, i32> {
            axis.view()
                .enumerate()
                .map(|(i, key)| (key.clone(), i as i32 + 1))
                .collect()
        }
        let (rows, cols) = (ordinals(&ws.index.rows), ordinals(&ws.index.cols));
        let row = |k: &FractionalKey| rows.get(k).copied();
        let col = |k: &FractionalKey| cols.get(k).copied();
        let range = |r: &StableRange| -> Option<RangeRef> {
            let rows = match &r.rows {
                Some((lo, hi)) => Some((row(lo)?, row(hi)?)),
                None => None,
            };
            let cols = match &r.cols {
                Some((lo, hi)) => Some((col(lo)?, col(hi)?)),
                None => None,
            };
            Some(RangeRef { rows, cols })
        };

        // The formula table is rebuilt as we go: a cell's stream is written back as R1C1 and
        // interned, so the indices the cells carry are the ones this table hands out.
        let mut shared_formulas: Vec<String> = Vec::new();
        let mut sheet_data = SheetData::<Ordinal>::default();
        for (r, cells) in &ws.sheet_data {
            let Some(r) = row(r) else { continue };
            for (c, cell) in cells {
                let Some(c) = col(c) else { continue };
                let cell = match cell.get_formula() {
                    Some(f) => {
                        let Some(node) = Stable::materialize_formula(self, at as u32, r, c, f)
                        else {
                            continue;
                        };
                        let text = to_rc_format(&node);
                        let f = match shared_formulas.iter().position(|s| s == &text) {
                            Some(f) => f as i32,
                            None => {
                                shared_formulas.push(text);
                                shared_formulas.len() as i32 - 1
                            }
                        };
                        match cell.clone() {
                            Cell::CellFormula { s, v, .. } => Cell::CellFormula { f, s, v },
                            Cell::ArrayFormula { s, r, kind, v, .. } => {
                                Cell::ArrayFormula { f, s, r, kind, v }
                            }
                            other => other,
                        }
                    }
                    None => cell.clone(),
                };
                sheet_data.entry(r).or_default().insert(c, cell);
            }
        }

        Worksheet {
            dimension: ws.dimension.clone(),
            // An open corner is the axis' extreme; ordinal addressing has no sentinel for it.
            cols: ws
                .cols
                .iter()
                .filter_map(|c| {
                    let min = match c.min.is_empty() {
                        true => 1,
                        false => col(&c.min)?,
                    };
                    let max = match c.max.is_empty() {
                        true => LAST_COLUMN,
                        false => col(&c.max)?,
                    };
                    Some(Col {
                        min,
                        max,
                        width: c.width,
                        custom_width: c.custom_width,
                        hidden: c.hidden,
                        style: c.style,
                    })
                })
                .collect(),
            rows: ws
                .rows
                .iter()
                .filter_map(|r| {
                    Some(Row {
                        r: row(&r.r)?,
                        height: r.height,
                        custom_format: r.custom_format,
                        custom_height: r.custom_height,
                        s: r.s,
                        hidden: r.hidden,
                    })
                })
                .collect(),
            name: ws.name.clone(),
            sheet_data,
            shared_formulas,
            sheet_id: ws.sheet_id,
            state: ws.state.clone(),
            color: ws.color.clone(),
            merged_cells: ws
                .merged_cells
                .iter()
                .filter_map(&range)
                .map(|r| MergedCell::from(&r))
                .collect(),
            comments: ws
                .comments
                .iter()
                .filter_map(|c| {
                    Some(Comment {
                        text: c.text.clone(),
                        author_name: c.author_name.clone(),
                        author_id: c.author_id.clone(),
                        cell_ref: (row(&c.cell_ref.0)?, col(&c.cell_ref.1)?),
                    })
                })
                .collect(),
            links: ws
                .links
                .iter()
                .filter_map(|((r, c), link)| Some(((row(r)?, col(c)?), self.link_view(link))))
                .collect(),
            frozen_rows: ws.frozen_rows,
            frozen_columns: ws.frozen_columns,
            views: ws.views.clone(),
            show_grid_lines: ws.show_grid_lines,
            conditional_formatting: ws
                .conditional_formatting
                .iter()
                .map(|cf| ConditionalFormatting {
                    ranges: cf.ranges.iter().filter_map(&range).collect(),
                    cf_rule: cf.cf_rule.clone(),
                    priority: cf.priority,
                })
                .collect(),
            index: (),
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::cf_types::{CfRuleInput, ValueOperator};
    use crate::collab::log::{Commit, Consumer};
    use crate::types::{Color, Comment as OrdinalComment, Dxf, Fill, Style};
    use crate::Model;

    fn source() -> Model<'static> {
        let mut m = Model::new_empty("import", "en", "UTC", "en").unwrap();
        m.new_sheet();
        m.rename_sheet_by_index(1, "Data").unwrap();

        // Every value type, with one shared string used twice.
        m.set_user_input(0, 1, 1, "42".to_string()).unwrap();
        m.set_user_input(0, 2, 1, "hello".to_string()).unwrap();
        m.set_user_input(0, 3, 1, "hello".to_string()).unwrap();
        m.set_user_input(0, 4, 1, "TRUE".to_string()).unwrap();
        m.set_user_input(0, 5, 1, "#VALUE!".to_string()).unwrap();
        m.set_user_input(1, 1, 1, "7".to_string()).unwrap();

        // A local formula, a cross-sheet one, and one reaching past the used extent.
        m.set_user_input(0, 1, 2, "=A1*2".to_string()).unwrap();
        m.set_user_input(0, 2, 2, "=Data!A1+1".to_string()).unwrap();
        m.set_user_input(0, 3, 2, "=SUM(A1:A20)".to_string())
            .unwrap();
        // A sheet the workbook does not have, and a name the import creates after the formulas.
        m.set_user_input(0, 5, 2, "=Nope!A1".to_string()).unwrap();
        m.set_user_input(0, 6, 2, "=total*2".to_string()).unwrap();

        let mut bold = Style::default();
        bold.font.b = true;
        m.set_cell_style(0, 1, 1, &bold).unwrap();
        let mut italic = Style::default();
        italic.font.i = true;
        m.set_row_style(0, 4, &italic).unwrap();
        m.set_row_height(0, 4, 30.0).unwrap();
        let mut underline = Style::default();
        underline.font.u = true;
        m.set_column_style(0, 3, &underline).unwrap();
        m.set_column_width(0, 3, 120.0).unwrap();

        m.add_conditional_formatting(
            0,
            "A1:B5",
            CfRuleInput::CellIs {
                operator: ValueOperator::GreaterThan,
                formula: "5".to_string(),
                formula2: None,
                format: Dxf {
                    fill: Some(Fill {
                        color: Color::Rgb("#FF0000".to_string()),
                    }),
                    ..Default::default()
                },
                stop_if_true: false,
            },
        )
        .unwrap();

        // An external and an internal link, the latter naming the other sheet.
        m.set_cell_link(
            0,
            7,
            1,
            Link::External {
                target: "https://ironcalc.com".to_string(),
                tooltip: None,
            },
        )
        .unwrap();
        m.set_cell_link(
            0,
            8,
            1,
            Link::Internal {
                location: "Data!A1".to_string(),
                tooltip: None,
            },
        )
        .unwrap();

        m.new_defined_name("total", None, "Sheet1!$A$1").unwrap();
        m.new_defined_name("local", Some(0), "Sheet1!$A$2").unwrap();
        m.create_named_style("Fancy", &bold, StyleIncludes::default())
            .unwrap();

        {
            let ws = m.workbook.worksheet_mut(0).unwrap();
            ws.merged_cells.push(MergedCell {
                row: 6,
                column: 3,
                width: 2,
                height: 2,
            });
            ws.comments.push(OrdinalComment {
                text: "hi".to_string(),
                author_name: "me".to_string(),
                author_id: None,
                cell_ref: (6, 1),
            });
        }
        m.evaluate();
        m
    }

    fn compare(ordinal: &Model, collab: &CollabModel, rows: i32, cols: i32, step: &str) {
        for sheet in 0..ordinal.workbook.worksheets.len() as u32 {
            for row in 1..=rows {
                for col in 1..=cols {
                    assert_eq!(
                        collab.get_formatted_cell_value(sheet, row, col),
                        ordinal.get_formatted_cell_value(sheet, row, col),
                        "value at sheet {sheet} row {row} column {col} after {step}"
                    );
                    assert_eq!(
                        collab.get_cell_formula(sheet, row, col),
                        ordinal.get_cell_formula(sheet, row, col),
                        "formula at sheet {sheet} row {row} column {col} after {step}"
                    );
                }
            }
        }
    }

    fn deliver(model: &mut CollabModel<'_>, commits: &[Commit]) {
        for commit in commits {
            model.apply(commit).unwrap();
        }
    }

    /// Rows and columns without the style index, which is local to each workbook's style table.
    fn row_shape(ws: &Worksheet) -> Vec<(i32, f64, bool, bool, bool)> {
        let mut rows: Vec<_> = ws
            .rows
            .iter()
            .map(|r| (r.r, r.height, r.hidden, r.custom_height, r.custom_format))
            .collect();
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        rows
    }

    fn col_shape(ws: &Worksheet) -> Vec<(i32, i32, f64, bool, bool)> {
        let mut cols: Vec<_> = ws
            .cols
            .iter()
            .map(|c| (c.min, c.max, c.width, c.custom_width, c.hidden))
            .collect();
        cols.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
        cols
    }

    #[test]
    fn ordinal_round_trip() {
        let mut ordinal = source();
        let mut a =
            CollabModel::from_workbook_with_session(ordinal.workbook.clone(), "en", 1).unwrap();
        a.evaluate();
        compare(&ordinal, &a, 8, 4, "import");
        // Not just the values: the formatting, the merge, the comment and the rule came too.
        assert!(a.get_style_for_cell(0, 1, 1).unwrap().font.b);
        let row = &a.workbook.worksheets[0].rows[0];
        assert!(row.custom_format);
        assert!(a.workbook.styles.get_style(row.s).unwrap().font.i);
        assert!(a.get_style_for_cell(0, 1, 3).unwrap().font.u);
        assert_eq!(a.workbook.worksheets[0].merged_cells.len(), 1);
        assert_eq!(a.workbook.worksheets[0].comments.len(), 1);
        // Both links survived the import and read back as they were written.
        for (row, column) in [(7, 1), (8, 1)] {
            assert_eq!(
                a.get_cell_link(0, row, column),
                ordinal.get_cell_link(0, row, column),
                "link at {row}:{column}"
            );
        }
        assert_eq!(a.workbook.worksheets[0].conditional_formatting.len(), 1);
        assert_eq!(a.workbook.defined_names.len(), 2);
        assert!(a
            .workbook
            .styles
            .cell_styles
            .iter()
            .any(|s| s.name == "Fancy"));

        // The import is a patch stream: a peer that only ever saw the commits agrees with A.
        let commits = a.flush();
        let mut b = CollabModel::new(2);
        deliver(&mut b, &commits);
        b.evaluate();
        assert_eq!(
            b.workbook.get_worksheet_names(),
            a.workbook.get_worksheet_names()
        );
        for (i, sheet) in a.workbook.worksheets.iter().enumerate() {
            assert_eq!(b.workbook.worksheets[i].index, sheet.index);
        }
        for sheet in 0..2 {
            for row in 1..=8 {
                for col in 1..=4 {
                    assert_eq!(
                        b.get_formatted_cell_value(sheet, row, col),
                        a.get_formatted_cell_value(sheet, row, col),
                        "value at sheet {sheet} row {row} column {col} on the peer"
                    );
                    assert_eq!(
                        b.get_style_for_cell(sheet, row, col),
                        a.get_style_for_cell(sheet, row, col),
                        "style at sheet {sheet} row {row} column {col} on the peer"
                    );
                }
            }
        }

        // Concurrent edits to two imported cells: both survive, both replicas agree.
        a.set_user_input(0, 1, 1, "100".to_string()).unwrap();
        b.set_user_input(0, 2, 1, "world".to_string()).unwrap();
        let from_a = a.flush();
        let from_b = b.flush();
        deliver(&mut b, &from_a);
        deliver(&mut a, &from_b);
        a.evaluate();
        b.evaluate();
        assert_eq!(a.get_formatted_cell_value(0, 1, 1), Ok("100".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 2, 1), Ok("world".to_string()));
        assert_eq!(
            b.get_formatted_cell_value(0, 1, 1),
            a.get_formatted_cell_value(0, 1, 1)
        );
        assert_eq!(
            b.get_formatted_cell_value(0, 2, 1),
            a.get_formatted_cell_value(0, 2, 1)
        );

        // Back to ordinal. Compare against a fresh import so the concurrent edits above are out
        // of the way.
        let a = CollabModel::from_workbook_with_session(ordinal.workbook.clone(), "en", 1).unwrap();
        let projected = a.to_ordinal_workbook();
        assert_eq!(
            projected.worksheets.len(),
            ordinal.workbook.worksheets.len()
        );
        assert!(!projected.worksheets[0].cols.is_empty());
        assert!(!projected.worksheets[0].rows.is_empty());
        assert!(!projected.worksheets[0].shared_formulas.is_empty());
        for (i, ws) in ordinal.workbook.worksheets.iter().enumerate() {
            let out = &projected.worksheets[i];
            assert_eq!(out.name, ws.name);
            assert_eq!(out.sheet_id, ws.sheet_id);
            assert_eq!(out.state, ws.state);
            assert_eq!(out.color, ws.color);
            assert_eq!(out.show_grid_lines, ws.show_grid_lines);
            assert_eq!(out.frozen_rows, ws.frozen_rows);
            assert_eq!(out.frozen_columns, ws.frozen_columns);
            assert_eq!(row_shape(out), row_shape(ws));
            assert_eq!(col_shape(out), col_shape(ws));
            assert_eq!(out.merged_cells, ws.merged_cells);
            assert_eq!(out.comments, ws.comments);
            assert_eq!(out.links, ws.links);
            let ranges = |w: &Worksheet| -> Vec<Vec<RangeRef>> {
                w.conditional_formatting
                    .iter()
                    .map(|cf| cf.ranges.clone())
                    .collect()
            };
            assert_eq!(ranges(out), ranges(ws));
            // Non-formula cells come back as they went in; formula cells are compared by text.
            for (r, cells) in &ws.sheet_data {
                for (c, cell) in cells {
                    let back = out.sheet_data.get(r).and_then(|row| row.get(c));
                    if cell.has_formula() {
                        assert!(back.unwrap().has_formula(), "formula lost at {r}:{c}");
                    } else {
                        assert_eq!(
                            back.map(|b| (b.get_type(), b.get_style() != 0)),
                            Some((cell.get_type(), cell.get_style() != 0)),
                            "cell at sheet {i} {r}:{c}"
                        );
                    }
                }
            }
        }
        let names = |w: &Workbook| {
            let mut n: Vec<_> = w
                .defined_names
                .iter()
                .map(|d| (d.name.clone(), d.sheet_id))
                .collect();
            n.sort();
            n
        };
        assert_eq!(names(&projected), names(&ordinal.workbook));
        let styles = |w: &Workbook| {
            let mut n: Vec<_> = w
                .styles
                .cell_styles
                .iter()
                .map(|s| s.name.clone())
                .collect();
            n.sort();
            n
        };
        assert_eq!(styles(&projected), styles(&ordinal.workbook));

        // The exported workbook is a workbook: it reads back the way the original does.
        let reloaded = Model::from_workbook(projected, "en").unwrap();
        compare(&ordinal, &a, 8, 4, "export");
        for row in 1..=8 {
            for col in 1..=4 {
                assert_eq!(
                    reloaded.get_formatted_cell_value(0, row, col),
                    ordinal.get_formatted_cell_value(0, row, col),
                    "reloaded value at row {row} column {col}"
                );
            }
        }

        // A row deleted on the collaborative side is a row gone from the projection, and the rows
        // below it moved up — exactly as the ordinal model reports after the same delete.
        let mut a =
            CollabModel::from_workbook_with_session(ordinal.workbook.clone(), "en", 1).unwrap();
        a.delete_rows(0, 2, 1).unwrap();
        a.evaluate();
        ordinal.delete_rows(0, 2, 1).unwrap();
        ordinal.evaluate();
        let deleted = Model::from_workbook(a.to_ordinal_workbook(), "en").unwrap();
        for row in 1..=8 {
            for col in 1..=4 {
                assert_eq!(
                    deleted.get_formatted_cell_value(0, row, col),
                    ordinal.get_formatted_cell_value(0, row, col),
                    "value after delete at row {row} column {col}"
                );
            }
        }
    }
}
