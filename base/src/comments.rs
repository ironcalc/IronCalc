//! Cell comments, the legacy ones Excel calls notes. A comment is attached to
//! its cell by the cell's A1 reference, as it is stored in the xlsx file.

use crate::expressions::utils::{number_to_column, parse_reference_a1};
use crate::links::check_valid_cell;
use crate::model::Model;
use crate::types::{Comment, Worksheet};

fn to_cell_ref(row: i32, column: i32) -> Result<String, String> {
    let column_name =
        number_to_column(column).ok_or_else(|| format!("Invalid column: '{column}'"))?;
    Ok(format!("{column_name}{row}"))
}

/// The (row, column) of a comment, or `None` if its reference does not parse.
fn comment_cell(comment: &Comment) -> Option<(i32, i32)> {
    parse_reference_a1(&comment.cell_ref).map(|r| (r.row, r.column))
}

fn comment_index(worksheet: &Worksheet, row: i32, column: i32) -> Option<usize> {
    worksheet
        .comments
        .iter()
        .position(|c| comment_cell(c) == Some((row, column)))
}

/// Applies `map` to the cell of every comment in the worksheet, so that
/// comments follow their cells when rows or columns are inserted, deleted or
/// moved: `Some((row, column))` moves the comment there, `None` removes it.
pub(crate) fn displace_comments<F>(worksheet: &mut Worksheet, map: F)
where
    F: Fn(i32, i32) -> Option<(i32, i32)>,
{
    let comments = std::mem::take(&mut worksheet.comments);
    worksheet.comments = comments
        .into_iter()
        .filter_map(|mut comment| match comment_cell(&comment) {
            Some((row, column)) => {
                let (row, column) = map(row, column)?;
                comment.cell_ref = to_cell_ref(row, column).ok()?;
                Some(comment)
            }
            // Keep what we cannot place, rather than lose it
            None => Some(comment),
        })
        .collect();
}

impl Model<'_> {
    /// Returns the comment attached to cell (`row`, `column`) or `None` if there isn't one.
    pub fn get_cell_comment(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<Option<Comment>, String> {
        check_valid_cell(row, column)?;
        let worksheet = self.workbook.worksheet(sheet)?;
        Ok(comment_index(worksheet, row, column).map(|i| worksheet.comments[i].clone()))
    }

    /// Returns all the comments in the worksheet.
    pub fn get_comments(&self, sheet: u32) -> Result<&[Comment], String> {
        Ok(&self.workbook.worksheet(sheet)?.comments)
    }

    /// Attaches a comment with `text` by `author_name` to cell (`row`, `column`),
    /// replacing the existing comment if there is one.
    pub fn set_cell_comment(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        text: &str,
        author_name: &str,
    ) -> Result<(), String> {
        let comment = Comment {
            text: text.to_string(),
            author_name: author_name.to_string(),
            author_id: None,
            cell_ref: to_cell_ref(row, column)?,
        };
        self.put_cell_comment(sheet, row, column, comment)
    }

    /// Attaches `comment` to cell (`row`, `column`) as it is, replacing the
    /// existing comment if there is one. Undo and redo use this to restore a
    /// comment exactly.
    pub(crate) fn put_cell_comment(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        mut comment: Comment,
    ) -> Result<(), String> {
        check_valid_cell(row, column)?;
        comment.cell_ref = to_cell_ref(row, column)?;
        let worksheet = self.workbook.worksheet_mut(sheet)?;
        match comment_index(worksheet, row, column) {
            Some(i) => worksheet.comments[i] = comment,
            None => worksheet.comments.push(comment),
        }
        Ok(())
    }

    /// Removes the comment attached to cell (`row`, `column`). It is not an
    /// error if the cell has no comment.
    pub fn delete_cell_comment(&mut self, sheet: u32, row: i32, column: i32) -> Result<(), String> {
        check_valid_cell(row, column)?;
        let worksheet = self.workbook.worksheet_mut(sheet)?;
        if let Some(i) = comment_index(worksheet, row, column) {
            worksheet.comments.remove(i);
        }
        Ok(())
    }
}
