use crate::expressions::types::Area;
use crate::types::{Cell, MergedCell};

use super::{common::UserModel, history::Diff};

impl UserModel<'_> {
    /// Merges the cells of `range` into a single merged cell anchored at its
    /// top-left corner. Like in Excel, the content and links of every covered
    /// cell are cleared and the anchor's style is copied to the whole range
    /// (with its borders kept only on the perimeter); undoing restores
    /// everything.
    ///
    /// Fails if the range is invalid, a single cell, or intersects an existing
    /// merged cell or an array formula.
    ///
    /// See also:
    /// * [Model::merge_cells](crate::Model::merge_cells)
    pub fn merge_cells(&mut self, range: &Area) -> Result<(), String> {
        let sheet = range.sheet;
        let old_merged_cells = self.model.get_merged_cells(sheet)?.to_vec();

        let mut diff_list = self.covered_cells_clear_diffs(range)?;

        // Merging stamps the anchor's style on the whole range: capture the
        // explicit style of every cell before and after so undo restores them
        let mut old_styles = Vec::new();
        for row in range.row..range.row + range.height {
            for column in range.column..range.column + range.width {
                old_styles.push((
                    row,
                    column,
                    self.model.get_cell_style_or_none(sheet, row, column)?,
                ));
            }
        }

        self.model.merge_cells(range)?;

        for (row, column, old_style) in old_styles {
            let new_style = self.model.get_cell_style_or_none(sheet, row, column)?;
            if new_style != old_style {
                if let Some(new_style) = new_style {
                    diff_list.push(Diff::SetCellStyle {
                        sheet,
                        row,
                        column,
                        old_value: Box::new(old_style),
                        new_value: Box::new(new_style),
                    });
                }
            }
        }

        // A covered cell can never stay selected: if the selected cell is now
        // inside the merged range, snap the selection to the whole merged
        // cell, anchored at its top-left corner. The scroll position and the
        // selection are left alone on undo.
        if let Ok(worksheet) = self.model.workbook.worksheet_mut(sheet) {
            if let Some(view) = worksheet.views.get_mut(&self.model.view_id) {
                if view.row >= range.row
                    && view.row < range.row + range.height
                    && view.column >= range.column
                    && view.column < range.column + range.width
                {
                    view.row = range.row;
                    view.column = range.column;
                    view.range = [
                        range.row,
                        range.column,
                        range.row + range.height - 1,
                        range.column + range.width - 1,
                    ];
                    view.focus_row = range.row;
                    view.focus_column = range.column;
                }
            }
        }

        let new_merged_cells = self.model.get_merged_cells(sheet)?.to_vec();
        diff_list.push(Diff::SetMergedCells {
            sheet,
            old_value: old_merged_cells,
            new_value: new_merged_cells,
        });
        self.push_diff_list(diff_list);
        self.evaluate_if_not_paused();
        Ok(())
    }

    /// Removes every merged cell that intersects `range`. The content of the
    /// anchors is kept. Removing no merged cells at all is a no-op.
    ///
    /// See also:
    /// * [Model::unmerge_cells](crate::Model::unmerge_cells)
    pub fn unmerge_cells(&mut self, range: &Area) -> Result<(), String> {
        let sheet = range.sheet;
        let old_value = self.model.get_merged_cells(sheet)?.to_vec();
        self.model.unmerge_cells(range)?;
        let new_value = self.model.get_merged_cells(sheet)?.to_vec();
        if old_value == new_value {
            // no-op, don't pollute the undo history
            return Ok(());
        }
        self.push_diff_list(vec![Diff::SetMergedCells {
            sheet,
            old_value,
            new_value,
        }]);
        self.evaluate_if_not_paused();
        Ok(())
    }

    /// Returns the list of merged cells of the worksheet.
    pub fn get_merged_cells(&self, sheet: u32) -> Result<Vec<MergedCell>, String> {
        Ok(self.model.get_merged_cells(sheet)?.to_vec())
    }

    // Captures the undo diffs for the content and links that merging `range`
    // will clear: everything but the anchor.
    pub(super) fn covered_cells_clear_diffs(&self, range: &Area) -> Result<Vec<Diff>, String> {
        let sheet = range.sheet;
        let mut diff_list: Vec<Diff> = Vec::new();
        let worksheet = self.model.workbook.worksheet(sheet)?;
        for row in range.row..range.row + range.height {
            for column in range.column..range.column + range.width {
                if row == range.row && column == range.column {
                    continue;
                }
                if let Some(link) = worksheet.links.get(&(row, column)) {
                    diff_list.push(Diff::SetCellLink {
                        sheet,
                        row,
                        column,
                        old_value: Box::new(Some(link.clone())),
                        new_value: Box::new(None),
                    });
                }
                let old_value = worksheet.cell(row, column).cloned();
                if matches!(&old_value, None | Some(Cell::EmptyCell { .. })) {
                    continue;
                }
                diff_list.push(Diff::SetCellValue {
                    sheet,
                    row,
                    column,
                    new_value: "".to_string(),
                    old_value: Box::new(old_value),
                });
            }
        }
        Ok(diff_list)
    }
}
