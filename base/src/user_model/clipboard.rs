#![deny(missing_docs)]

use std::{
    collections::{HashMap, HashSet},
    io::Cursor,
};

use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

use crate::{
    expressions::types::{Area, CellReferenceIndex},
    model::CellStructure,
    types::{ArrayKind, Cell, Style},
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
        let row_end = row_end.min(dimension.max_row);
        let column_end = column_end.min(dimension.max_column);
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
                data_row.insert(
                    column,
                    ClipboardCell {
                        text: content,
                        is_spill,
                        style,
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
        let [selected_row, selected_column, _, _] = view.range;
        let mut max_row = selected_row;
        let mut max_column = selected_column;
        let area = &Area {
            sheet,
            row: source_first_row,
            column: source_first_column,
            width: source_last_column - source_first_column + 1,
            height: source_last_row - source_first_row + 1,
        };
        let target_area = &Area {
            sheet,
            row: selected_row,
            column: selected_column,
            width: source_last_column - source_first_column + 1,
            height: source_last_row - source_first_row + 1,
        };

        let mut seen_cells = HashSet::new();
        // Compute all changes
        let mut changes = Vec::new();
        for (source_row, data_row) in clipboard {
            let delta_row = source_row - source_first_row;
            let target_row = selected_row + delta_row;
            max_row = max_row.max(target_row);
            for (source_column, value) in data_row {
                let delta_column = source_column - source_first_column;
                let target_column = selected_column + delta_column;
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

        self.model.range_clear_contents(&paste_area)?;

        // Second pass: write values and build diff list.
        let mut diff_list = Vec::new();
        let mut row = area.row;
        let mut last_column = area.column;
        for row_data in &records {
            let mut column = area.column;
            for value in row_data {
                let old_value = old_values.remove(&(row, column)).unwrap_or(None);
                self.model
                    .set_user_input(sheet, row, column, value.to_string())?;
                diff_list.push(Diff::SetCellValue {
                    sheet,
                    row,
                    column,
                    new_value: value.to_string(),
                    old_value: Box::new(old_value),
                });
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
