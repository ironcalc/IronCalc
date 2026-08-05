use std::collections::HashMap;

use crate::links::CellLinkView;
use crate::types::Link;

use super::{common::UserModel, history::Diff};

impl UserModel<'_> {
    /// Returns the link attached to cell (`row`, `column`) or `None` if there isn't one.
    pub fn get_cell_link(&self, sheet: u32, row: i32, column: i32) -> Result<Option<Link>, String> {
        self.model.get_cell_link(sheet, row, column)
    }

    /// Returns all the links in the worksheet, keyed by (row, column).
    pub fn get_links(&self, sheet: u32) -> Result<&HashMap<(i32, i32), Link>, String> {
        self.model.get_links(sheet)
    }

    /// Returns all the links in the worksheet as a list sorted by (row, column).
    pub fn get_links_list(&self, sheet: u32) -> Result<Vec<CellLinkView>, String> {
        self.model.get_links_list(sheet)
    }

    /// Attaches `link` to cell (`row`, `column`), replacing the existing link if there
    /// was one. Note that the link is only metadata: the text displayed in the cell is
    /// the cell content and is not modified by this method.
    pub fn set_cell_link(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        link: Link,
    ) -> Result<(), String> {
        let old_value = self.model.get_cell_link(sheet, row, column)?;
        if old_value.as_ref() == Some(&link) {
            // no-op, don't pollute the undo history
            return Ok(());
        }
        self.model.set_cell_link(sheet, row, column, link.clone())?;
        self.push_diff_list(vec![Diff::SetCellLink {
            sheet,
            row,
            column,
            old_value: Box::new(old_value),
            new_value: Box::new(Some(link)),
        }]);
        Ok(())
    }

    /// Removes the link attached to cell (`row`, `column`). It is not an error if the
    /// cell has no link.
    pub fn delete_cell_link(&mut self, sheet: u32, row: i32, column: i32) -> Result<(), String> {
        let old_value = self.model.get_cell_link(sheet, row, column)?;
        if old_value.is_none() {
            return Ok(());
        }
        self.model.delete_cell_link(sheet, row, column)?;
        self.push_diff_list(vec![Diff::SetCellLink {
            sheet,
            row,
            column,
            old_value: Box::new(old_value),
            new_value: Box::new(None),
        }]);
        Ok(())
    }
}
