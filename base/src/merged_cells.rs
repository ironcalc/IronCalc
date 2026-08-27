use serde::{Deserialize, Serialize};

use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::types::Area;
use crate::model::{CellStructure, Model};
use crate::types::MergedCell;

/// Position of a cell relative to the merged cells of a worksheet
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum MergeStructure {
    /// The cell is not part of any merged range
    None,
    /// The cell is the anchor (top-left cell) of a merged range
    Anchor {
        /// Number of columns of the merged range
        width: i32,
        /// Number of rows of the merged range
        height: i32,
    },
    /// The cell is inside a merged range but is not its anchor
    Covered {
        /// Row of the anchor of the merged range
        anchor_row: i32,
        /// Column of the anchor of the merged range
        anchor_column: i32,
    },
}

impl<'a> Model<'a> {
    /// Merges the cells in `range` into a single merged cell anchored at its
    /// top-left corner, mimicking Excel's behavior:
    /// * the content and links of every cell other than the anchor are cleared
    /// * the style of the anchor is copied to every cell of the range, so the
    ///   old styles of the covered cells are forgotten (after unmerging the
    ///   whole range shows the anchor's formatting)
    /// * borders are the exception: the anchor's borders outline the merged
    ///   cell as a whole, so each cell keeps only the border sides that lie on
    ///   the perimeter of the range (interior edges have no border)
    ///
    /// Fails if:
    /// * the range is invalid, out of bounds or a single cell
    /// * the range intersects an existing merged cell
    /// * the range intersects an array formula
    pub fn merge_cells(&mut self, range: &Area) -> Result<(), String> {
        self.merge_cells_keep_styles(range)?;
        let Area {
            sheet,
            row,
            column,
            width,
            height,
        } = *range;
        let anchor_style = self.get_style_for_cell(sheet, row, column)?;
        for r in row..row + height {
            for c in column..column + width {
                let mut style = anchor_style.clone();
                if r != row {
                    style.border.top = None;
                }
                if r != row + height - 1 {
                    style.border.bottom = None;
                }
                if c != column {
                    style.border.left = None;
                }
                if c != column + width - 1 {
                    style.border.right = None;
                }
                if self.get_style_for_cell(sheet, r, c)? != style {
                    self.set_cell_style(sheet, r, c, &style)?;
                }
            }
        }
        Ok(())
    }

    // Registers the merged cell and clears the content and links of the
    // covered cells, but leaves every cell's style untouched. Used by the
    // clipboard when recreating a pasted merge: the pasted cells already carry
    // the merged style pattern of the source, and stamping the anchor's style
    // would drop the perimeter borders that live on non-anchor cells.
    pub(crate) fn merge_cells_keep_styles(&mut self, range: &Area) -> Result<(), String> {
        let Area {
            sheet,
            row,
            column,
            width,
            height,
        } = *range;
        if row < 1 || column < 1 || width < 1 || height < 1 {
            return Err("Invalid range".to_string());
        }
        if row + height - 1 > LAST_ROW || column + width - 1 > LAST_COLUMN {
            return Err("Range is out of bounds".to_string());
        }
        if width == 1 && height == 1 {
            return Err("Cannot merge a single cell".to_string());
        }
        let worksheet = self.workbook.worksheet(sheet)?;
        if let Some(m) = worksheet
            .merged_cells
            .iter()
            .find(|m| m.intersects(row, column, width, height))
        {
            return Err(format!(
                "Range intersects the merged cell at row {}, column {}",
                m.row, m.column
            ));
        }
        for r in row..row + height {
            for c in column..column + width {
                match worksheet.get_cell_structure(r, c)? {
                    CellStructure::ArrayFormula {
                        range: (array_width, array_height),
                    } => {
                        if array_width > 1 || array_height > 1 {
                            return Err(
                                "Cannot merge cells that intersect an array formula".to_string()
                            );
                        }
                    }
                    CellStructure::SpillArray { .. } => {
                        return Err(
                            "Cannot merge cells that intersect an array formula".to_string()
                        );
                    }
                    _ => {}
                }
            }
        }
        // Clear the content and links of the covered cells. Going through
        // prepare_cell_for_user_input keeps dynamic array formulas consistent:
        // a covered spill anchor loses its spill and an outside anchor spilling
        // into the range is reset, so it re-evaluates (to #SPILL!) afterwards.
        for r in row..row + height {
            for c in column..column + width {
                if r == row && c == column {
                    continue;
                }
                self.prepare_cell_for_user_input(sheet, r, c)?;
                let worksheet = self.workbook.worksheet_mut(sheet)?;
                if worksheet.cell(r, c).is_some() {
                    worksheet.cell_clear_contents(r, c)?;
                }
                worksheet.links.remove(&(r, c));
            }
        }
        self.workbook
            .worksheet_mut(sheet)?
            .merged_cells
            .push(MergedCell {
                row,
                column,
                width,
                height,
            });
        Ok(())
    }

    /// Removes every merged cell that intersects `range`. The content of the
    /// anchors is kept. Removing no merged cells at all is not an error.
    pub fn unmerge_cells(&mut self, range: &Area) -> Result<(), String> {
        let Area {
            sheet,
            row,
            column,
            width,
            height,
        } = *range;
        if row < 1 || column < 1 || width < 1 || height < 1 {
            return Err("Invalid range".to_string());
        }
        let worksheet = self.workbook.worksheet_mut(sheet)?;
        worksheet
            .merged_cells
            .retain(|m| !m.intersects(row, column, width, height));
        Ok(())
    }

    /// Returns the list of merged cells of the worksheet
    pub fn get_merged_cells(&self, sheet: u32) -> Result<&[MergedCell], String> {
        Ok(&self.workbook.worksheet(sheet)?.merged_cells)
    }

    /// Returns the position of (row, column) relative to the merged cells of
    /// the worksheet: not merged, the anchor of a merged range or covered by one.
    pub fn get_merge_structure(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<MergeStructure, String> {
        let worksheet = self.workbook.worksheet(sheet)?;
        Ok(match worksheet.merged_cell_containing(row, column) {
            Some(m) if m.row == row && m.column == column => MergeStructure::Anchor {
                width: m.width,
                height: m.height,
            },
            Some(m) => MergeStructure::Covered {
                anchor_row: m.row,
                anchor_column: m.column,
            },
            None => MergeStructure::None,
        })
    }
}
