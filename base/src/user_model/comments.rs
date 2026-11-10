use crate::types::Comment;

use super::{common::UserModel, history::Diff};

impl UserModel<'_> {
    /// Returns the comment attached to cell (`row`, `column`) or `None` if there isn't one.
    pub fn get_cell_comment(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<Option<Comment>, String> {
        self.model.get_cell_comment(sheet, row, column)
    }

    /// Returns all the comments in the worksheet.
    pub fn get_comments(&self, sheet: u32) -> Result<&[Comment], String> {
        self.model.get_comments(sheet)
    }

    /// Attaches a comment with `text` by `author_name` to cell (`row`, `column`),
    /// replacing the existing comment if there is one. One entry in the undo history.
    pub fn set_cell_comment(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        text: &str,
        author_name: &str,
    ) -> Result<(), String> {
        let old_value = self.model.get_cell_comment(sheet, row, column)?;
        self.model
            .set_cell_comment(sheet, row, column, text, author_name)?;
        let new_value = self.model.get_cell_comment(sheet, row, column)?;
        if old_value == new_value {
            return Ok(());
        }
        self.push_diff_list(vec![Diff::SetCellComment {
            sheet,
            row,
            column,
            old_value: Box::new(old_value),
            new_value: Box::new(new_value),
        }]);
        Ok(())
    }

    /// Removes the comment attached to cell (`row`, `column`). It is not an error
    /// if the cell has no comment.
    pub fn delete_cell_comment(&mut self, sheet: u32, row: i32, column: i32) -> Result<(), String> {
        let old_value = self.model.get_cell_comment(sheet, row, column)?;
        if old_value.is_none() {
            return Ok(());
        }
        self.model.delete_cell_comment(sheet, row, column)?;
        self.push_diff_list(vec![Diff::SetCellComment {
            sheet,
            row,
            column,
            old_value: Box::new(old_value),
            new_value: Box::new(None),
        }]);
        Ok(())
    }

    /// Returns the diffs that remove the comments of the rows (`by_row`) or the
    /// columns in `start..start + count`, so that undoing their deletion
    /// restores them.
    pub(super) fn deleted_comment_diffs(
        &self,
        sheet: u32,
        start: i32,
        count: i32,
        by_row: bool,
    ) -> Result<Vec<Diff>, String> {
        let mut diffs = Vec::new();
        for comment in self.model.get_comments(sheet)? {
            let Some(reference) = crate::expressions::utils::parse_reference_a1(&comment.cell_ref)
            else {
                continue;
            };
            let position = if by_row {
                reference.row
            } else {
                reference.column
            };
            if position >= start && position < start + count {
                diffs.push(Diff::SetCellComment {
                    sheet,
                    row: reference.row,
                    column: reference.column,
                    old_value: Box::new(Some(comment.clone())),
                    new_value: Box::new(None),
                });
            }
        }
        Ok(diffs)
    }
}
