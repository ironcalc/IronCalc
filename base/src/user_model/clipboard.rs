#![deny(missing_docs)]

use std::{
    collections::{HashMap, HashSet},
    io::Cursor,
};

use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

use crate::{
    cf_types::ConditionalFormatting,
    expressions::types::{Area, CellReferenceIndex},
    model::CellStructure,
    types::{ArrayKind, Cell, Link, Style},
    UserModel,
};

use crate::user_model::history::Diff;

/// Data for the clipboard
pub type ClipboardData = HashMap<i32, HashMap<i32, ClipboardCell>>;

pub type ClipboardTuple = (i32, i32, i32, i32);

#[derive(Serialize, Deserialize)]
pub struct ClipboardCell {
    text: String,
    is_spill: bool,
    style: Style,
    // the link attached to the cell, if any (`default` keeps older clipboard
    // payloads without the field deserializable)
    #[serde(default)]
    link: Option<Link>,
}

#[derive(Serialize, Deserialize)]
pub struct Clipboard {
    pub(crate) csv: String,
    pub(crate) data: ClipboardData,
    pub(crate) sheet: u32,
    pub(crate) range: (i32, i32, i32, i32),
}

impl<'a> UserModel<'a> {
    /// Returns a copy of the selected area
    pub fn copy_to_clipboard(&self) -> Result<Clipboard, String> {
        let selected_area = self.get_selected_view();
        let sheet = selected_area.sheet;
        let mut wtr = WriterBuilder::new().delimiter(b'\t').from_writer(vec![]);

        let mut data = HashMap::new();
        let [row_start, column_start, row_end, column_end] = selected_area.range;
        let dimension = self.model.workbook.worksheet(sheet)?.dimension();
        let row_end = row_end.min(dimension.max_row).max(row_start);
        let column_end = column_end.min(dimension.max_column).max(column_start);
        for row in row_start..=row_end {
            let mut data_row = HashMap::new();
            let mut text_row = Vec::new();
            for column in column_start..=column_end {
                let text = self.get_formatted_cell_value(sheet, row, column)?;
                let content = self.get_cell_content(sheet, row, column)?;
                let style = self.model.get_style_for_cell(sheet, row, column)?;
                let is_spill = matches!(
                    self.model.get_cell_structure(sheet, row, column)?,
                    CellStructure::SpillArray { .. } | CellStructure::SpillDynamic { .. }
                );
                let link = self.model.get_cell_link(sheet, row, column)?;
                data_row.insert(
                    column,
                    ClipboardCell {
                        text: content,
                        is_spill,
                        style,
                        link,
                    },
                );
                text_row.push(text);
            }
            wtr.write_record(text_row)
                .map_err(|e| format!("Error while processing csv: {e}"))?;
            data.insert(row, data_row);
        }

        let csv = String::from_utf8(
            wtr.into_inner()
                .map_err(|e| format!("Processing error: '{e}'"))?,
        )
        .map_err(|e| format!("Error converting from utf8: '{e}'"))?;

        Ok(Clipboard {
            csv: csv.trim().to_string(),
            data,
            sheet,
            range: (row_start, column_start, row_end, column_end),
        })
    }

    /// Paste text that we copied
    ///
    /// If this is a copy (not a cut) and the selected area is a whole multiple of the copied
    /// rectangle and strictly larger than it, the copy is *repeated* to fill the whole selection,
    /// like Excel does. Every repetition re-anchors its relative references to its own position
    /// (copying `=B3+1` from `B4` into `B5:B10` gives `=B4+1`, `=B5+1`, … `=B9+1`), and the whole
    /// fill is still a single undo step. Any other selection pastes the copy once at the
    /// selection's top-left corner.
    ///
    /// A fill writes (and records an undo diff for) every cell of the selection, so a caller that
    /// lets the user select a whole column or the whole sheet should bound the selection before
    /// pasting into it.
    pub fn paste_from_clipboard(
        &mut self,
        source_sheet: u32,
        source_range: ClipboardTuple,
        clipboard: &ClipboardData,
        is_cut: bool,
    ) -> Result<(), String> {
        let mut diff_list = Vec::new();
        let view = self.get_selected_view();
        let (source_first_row, source_first_column, source_last_row, source_last_column) =
            source_range;
        let sheet = view.sheet;
        let [selected_row, selected_column, selected_last_row, selected_last_column] = view.range;
        let mut max_row = selected_row;
        let mut max_column = selected_column;
        let source_width = source_last_column - source_first_column + 1;
        let source_height = source_last_row - source_first_row + 1;
        // A cut is a move, so it is always pasted exactly once.
        let (row_repeats, column_repeats) = if is_cut {
            (1, 1)
        } else {
            fill_repeats(
                source_height,
                source_width,
                selected_last_row - selected_row + 1,
                selected_last_column - selected_column + 1,
            )
        };
        let area = &Area {
            sheet,
            row: source_first_row,
            column: source_first_column,
            width: source_width,
            height: source_height,
        };
        let target_area = &Area {
            sheet,
            row: selected_row,
            column: selected_column,
            width: source_width * column_repeats,
            height: source_height * row_repeats,
        };

        let mut seen_cells = HashSet::new();
        // Compute all changes
        let mut changes = Vec::new();
        for (row_offset, column_offset) in
            fill_offsets(row_repeats, column_repeats, source_height, source_width)
        {
            for (source_row, data_row) in clipboard {
                let delta_row = source_row - source_first_row;
                let target_row = selected_row + delta_row + row_offset;
                max_row = max_row.max(target_row);
                for (source_column, value) in data_row {
                    let delta_column = source_column - source_first_column;
                    let target_column = selected_column + delta_column + column_offset;
                    max_column = max_column.max(target_column);

                    if value.is_spill {
                        // Spill cells carry no formula/value, but their style should still be copied.
                        let old_style =
                            self.model
                                .get_cell_style_or_none(sheet, target_row, target_column)?;
                        changes.push((
                            target_row,
                            target_column,
                            None,
                            old_style,
                            None,
                            value.style.clone(),
                        ));
                        seen_cells.insert((target_row, target_column));
                        continue;
                    }

                    // We are copying the value in
                    // (source_row, source_column) to (target_row , target_column)
                    // References in formulas are displaced

                    // remain in the copied area
                    let source = &CellReferenceIndex {
                        sheet,
                        column: *source_column,
                        row: *source_row,
                    };
                    let target = &CellReferenceIndex {
                        sheet,
                        column: target_column,
                        row: target_row,
                    };
                    let new_value = if is_cut {
                        self.model
                            .move_cell_value_to_area(&value.text, source, target, area)?
                    } else {
                        self.model
                            .extend_copied_value(&value.text, source, target)?
                    };

                    let old_value = self
                        .model
                        .workbook
                        .worksheet(sheet)?
                        .cell(target_row, target_column)
                        .cloned();

                    let old_style =
                        self.model
                            .get_cell_style_or_none(sheet, target_row, target_column)?;
                    changes.push((
                        target_row,
                        target_column,
                        old_value.clone(),
                        old_style.clone(),
                        Some(new_value.clone()),
                        value.style.clone(),
                    ));
                    seen_cells.insert((target_row, target_column));
                }
            }
        }
        // Clearing the target area also removes its links: capture them for undo
        diff_list.extend(self.range_link_diffs(target_area)?);
        // clear the whole area (this resets array formulas)
        self.model.range_clear_contents(target_area)?;
        // set the new values and styles
        for (target_row, target_column, old_value, old_style, new_value, style) in changes {
            if let Some(ref v) = new_value {
                self.model
                    .set_user_input(sheet, target_row, target_column, v.clone())?;
                diff_list.push(Diff::SetCellValue {
                    sheet,
                    row: target_row,
                    column: target_column,
                    new_value: v.clone(),
                    old_value: Box::new(old_value),
                });
            }
            self.model
                .set_cell_style(sheet, target_row, target_column, &style)?;

            diff_list.push(Diff::SetCellStyle {
                sheet,
                row: target_row,
                column: target_column,
                old_value: Box::new(old_style),
                new_value: Box::new(style),
            });
        }
        // Paste the links of the copied cells. This runs after the values are
        // set so that it also overrides any link auto-created by an URL value.
        for (source_row, data_row) in clipboard {
            let target_row = selected_row + (source_row - source_first_row);
            for (source_column, value) in data_row {
                let target_column = selected_column + (source_column - source_first_column);
                let old_link = self.model.get_cell_link(sheet, target_row, target_column)?;
                if old_link == value.link {
                    continue;
                }
                match &value.link {
                    Some(link) => {
                        self.model
                            .set_cell_link(sheet, target_row, target_column, link.clone())?
                    }
                    None => self
                        .model
                        .delete_cell_link(sheet, target_row, target_column)?,
                }
                diff_list.push(Diff::SetCellLink {
                    sheet,
                    row: target_row,
                    column: target_column,
                    old_value: Box::new(old_link),
                    new_value: Box::new(value.link.clone()),
                });
            }
        }
        if is_cut {
            for row in source_first_row..=source_last_row {
                for column in source_first_column..=source_last_column {
                    if (source_sheet == sheet) && seen_cells.contains(&(row, column)) {
                        continue;
                    }
                    let old_value = self
                        .model
                        .workbook
                        .worksheet(source_sheet)?
                        .cell(row, column)
                        .cloned();

                    diff_list.push(Diff::RangeClearContents {
                        sheet: source_sheet,
                        row,
                        column,
                        width: 1,
                        height: 1,
                        old_value: vec![vec![old_value.clone()]],
                    });

                    // a cut also moves the link away from the source cell
                    let old_link = self.model.get_cell_link(source_sheet, row, column)?;
                    if let Some(old_link) = old_link {
                        self.model.delete_cell_link(source_sheet, row, column)?;
                        diff_list.push(Diff::SetCellLink {
                            sheet: source_sheet,
                            row,
                            column,
                            old_value: Box::new(Some(old_link)),
                            new_value: Box::new(None),
                        });
                    }

                    // If the source is a dynamic formula anchor, range_clear_contents
                    // would erase its entire spill — including cells that were just
                    // written to by this paste.  Clear the anchor and its spill cells
                    // individually instead, skipping any paste-target cells.
                    let spill_dims = match &old_value {
                        Some(Cell::ArrayFormula {
                            kind: ArrayKind::Dynamic,
                            r,
                            ..
                        }) => Some(*r),
                        _ => None,
                    };
                    if let Some((spill_w, spill_h)) = spill_dims {
                        let ws = self.model.workbook.worksheet_mut(source_sheet)?;
                        for sr in row..row + spill_h {
                            for sc in column..column + spill_w {
                                if (source_sheet == sheet) && seen_cells.contains(&(sr, sc)) {
                                    continue;
                                }
                                let _ = ws.cell_clear_contents(sr, sc);
                            }
                        }
                    } else {
                        let area = Area {
                            sheet: source_sheet,
                            row,
                            column,
                            width: 1,
                            height: 1,
                        };
                        self.model.range_clear_contents(&area)?;
                    }
                    let old_style = self
                        .model
                        .get_cell_style_or_none(source_sheet, row, column)?;
                    let default_style = Style::default();
                    self.model
                        .set_cell_style(source_sheet, row, column, &default_style)?;
                    diff_list.push(Diff::SetCellStyle {
                        sheet: source_sheet,
                        row,
                        column,
                        old_value: Box::new(old_style),
                        new_value: Box::new(default_style),
                    });
                }
            }
            // Update external formulas that reference cells in the moved area.
            // source_sheet is used here (not `sheet`) so cross-sheet paste works.
            let ext_area = Area {
                sheet: source_sheet,
                row: source_first_row,
                column: source_first_column,
                width: source_last_column - source_first_column + 1,
                height: source_last_row - source_first_row + 1,
            };
            let ext_updates = self.model.get_external_formula_updates_for_cut(
                &ext_area,
                selected_row,
                selected_column,
            )?;
            for (ext_sheet, ext_row, ext_col, new_formula) in ext_updates {
                let old_cell = self
                    .model
                    .workbook
                    .worksheet(ext_sheet)?
                    .cell(ext_row, ext_col)
                    .cloned();
                self.model
                    .set_user_input(ext_sheet, ext_row, ext_col, new_formula.clone())?;
                diff_list.push(Diff::SetCellValue {
                    sheet: ext_sheet,
                    row: ext_row,
                    column: ext_col,
                    new_value: new_formula,
                    old_value: Box::new(old_cell),
                });
            }
            // Update defined names whose references land inside the moved area.
            let dn_updates = self.model.get_defined_name_updates_for_cut(
                &ext_area,
                selected_row,
                selected_column,
            );
            for (dn_name, dn_scope, old_formula, new_formula) in dn_updates {
                diff_list.push(Diff::UpdateDefinedName {
                    name: dn_name.clone(),
                    scope: dn_scope,
                    old_formula: old_formula.clone(),
                    new_name: dn_name.clone(),
                    new_scope: dn_scope,
                    new_formula: new_formula.clone(),
                });
                self.model.update_defined_name(
                    &dn_name,
                    dn_scope,
                    &dn_name,
                    dn_scope,
                    &new_formula,
                )?;
            }
            // Update conditional formatting ranges and formula references.
            let cf_updates = self.model.get_conditional_formatting_updates_for_cut(
                &ext_area,
                selected_row,
                selected_column,
            )?;
            for (cf_sheet, cf_idx, new_range, new_rule) in cf_updates {
                let old_cf = self
                    .model
                    .workbook
                    .worksheet(cf_sheet)?
                    .conditional_formatting
                    .get(cf_idx)
                    .ok_or_else(|| format!("CF index {cf_idx} not found"))?
                    .clone();
                {
                    let ws = self.model.workbook.worksheet_mut(cf_sheet)?;
                    ws.conditional_formatting[cf_idx].range = new_range.clone();
                    ws.conditional_formatting[cf_idx].cf_rule = new_rule.clone();
                }
                diff_list.push(Diff::UpdateConditionalFormatting {
                    sheet: cf_sheet,
                    index: cf_idx as u32,
                    old_range: old_cf.range,
                    old_rule: Box::new(old_cf.cf_rule),
                    old_priority: old_cf.priority,
                    new_range,
                    new_rule: Box::new(new_rule),
                });
            }
        } else {
            // Copy-paste: duplicate CF rules from the source area to the target. A fill adds ONE
            // entry per source rule, its sqref covering every repetition (`get_cf_rules_to_copy`
            // merges them), so the added rules never scale with the number of filled cells.
            let cf_copies = self.model.get_cf_rules_to_copy(
                source_sheet,
                source_first_row,
                source_first_column,
                source_last_row,
                source_last_column,
                selected_row,
                selected_column,
                (row_repeats, column_repeats),
            );
            // The next free priority, read once and then counted up — re-scanning the (growing)
            // rule list per added rule would be quadratic.
            let mut priority = self
                .model
                .workbook
                .worksheet(sheet)?
                .conditional_formatting
                .iter()
                .map(|cf| cf.priority)
                .max()
                .map(|m| m + 1)
                .unwrap_or(1);
            for (new_range, new_rule) in cf_copies {
                self.model
                    .workbook
                    .worksheet_mut(sheet)?
                    .conditional_formatting
                    .push(ConditionalFormatting {
                        range: new_range.clone(),
                        cf_rule: new_rule.clone(),
                        priority,
                    });
                diff_list.push(Diff::AddConditionalFormatting {
                    sheet,
                    range: new_range,
                    rule: Box::new(new_rule),
                    priority,
                });
                priority += 1;
            }
        }
        self.push_diff_list(diff_list);
        // select the pasted area
        self.set_selected_range(selected_row, selected_column, max_row, max_column)?;
        self.evaluate_if_not_paused();
        Ok(())
    }

    /// Paste a csv-string into the model
    pub fn paste_csv_string(&mut self, area: &Area, csv: &str) -> Result<(), String> {
        let sheet = area.sheet;

        // First pass: parse all records so we know the full extent before touching any cells.
        let mut records: Vec<Vec<String>> = Vec::new();
        let mut max_width: i32 = 0;
        let csv_reader = Cursor::new(csv);
        let mut reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .from_reader(csv_reader);
        for r in reader.records().flatten() {
            let row_data: Vec<String> = r.iter().map(|v| v.to_string()).collect();
            max_width = max_width.max(row_data.len() as i32);
            records.push(row_data);
        }
        if records.is_empty() {
            return Ok(());
        }

        // Check whether any static array formula would be partially overwritten.
        let paste_area = Area {
            sheet,
            row: area.row,
            column: area.column,
            width: max_width,
            height: records.len() as i32,
        };

        // Capture old values BEFORE clearing so undo can restore them correctly.
        let mut old_values: HashMap<(i32, i32), Option<Cell>> = HashMap::new();
        {
            let ws = self.model.workbook.worksheet(sheet)?;
            for r in area.row..area.row + records.len() as i32 {
                for c in area.column..area.column + max_width {
                    old_values.insert((r, c), ws.cell(r, c).cloned());
                }
            }
        }

        // Clearing the target area also removes its links: capture them for undo
        let mut diff_list = self.range_link_diffs(&paste_area)?;
        self.model.range_clear_contents(&paste_area)?;

        // Second pass: write values and build diff list.
        let mut row = area.row;
        let mut last_column = area.column;
        for row_data in &records {
            let mut column = area.column;
            for value in row_data {
                let old_value = old_values.remove(&(row, column)).unwrap_or(None);
                diff_list.push(Diff::SetCellValue {
                    sheet,
                    row,
                    column,
                    new_value: value.to_string(),
                    old_value: Box::new(old_value),
                });
                // pasted URLs are auto-linked: capture the link and style diffs too
                self.set_user_input_with_link_diffs(
                    sheet,
                    row,
                    column,
                    value.to_string(),
                    &mut diff_list,
                )?;
                column += 1;
            }
            last_column = last_column.max(column - 1);
            row += 1;
        }
        self.push_diff_list(diff_list);
        // select the pasted area
        self.set_selected_range(area.row, area.column, row - 1, last_column)?;
        self.evaluate_if_not_paused();
        Ok(())
    }
}

/// How many times a copied rectangle repeats down and to the right to fill the selected target
/// rectangle, as `(row_repeats, column_repeats)`.
///
/// A copy fills the selection only when the selection is a whole multiple of it in *both* axes
/// and strictly larger in at least one of them (Excel's rule). Every other selection — including
/// a partial multiple like a 2-row copy into a 3-row selection — pastes the copy once.
fn fill_repeats(
    source_height: i32,
    source_width: i32,
    target_height: i32,
    target_width: i32,
) -> (i32, i32) {
    if source_width < 1 || source_height < 1 || target_width < 1 || target_height < 1 {
        return (1, 1);
    }
    let fills = target_width % source_width == 0
        && target_height % source_height == 0
        && (target_width > source_width || target_height > source_height);
    if fills {
        (target_height / source_height, target_width / source_width)
    } else {
        (1, 1)
    }
}

/// The `(row_offset, column_offset)` of every repetition of a `source_height`×`source_width`
/// rectangle in a fill of `row_repeats`×`column_repeats` repetitions, relative to the target's
/// top-left corner. Always yields at least `(0, 0)` — the plain single paste.
fn fill_offsets(
    row_repeats: i32,
    column_repeats: i32,
    source_height: i32,
    source_width: i32,
) -> impl Iterator<Item = (i32, i32)> {
    (0..row_repeats).flat_map(move |repeat_row| {
        (0..column_repeats)
            .map(move |repeat_column| (repeat_row * source_height, repeat_column * source_width))
    })
}
