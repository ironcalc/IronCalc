//! Cell hyperlinks. Links are cell metadata: the text displayed in the cell is the
//! cell content and is not part of the link.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    constants::{LAST_COLUMN, LAST_ROW},
    types::Link,
    Model,
};

/// A link together with the cell (`row`, `column`) it is attached to.
/// This is the shape the bindings expose to UIs, with the link fields flattened:
/// `{"row": 2, "column": 2, "type": "External", "target": "...", "tooltip": null}`.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct CellLinkView {
    /// Row of the cell the link is attached to
    pub row: i32,
    /// Column of the cell the link is attached to
    pub column: i32,
    /// The link itself
    #[serde(flatten)]
    pub link: Link,
}

fn check_valid_cell(row: i32, column: i32) -> Result<(), String> {
    if !(1..=LAST_ROW).contains(&row) {
        return Err(format!("Invalid row: '{row}'"));
    }
    if !(1..=LAST_COLUMN).contains(&column) {
        return Err(format!("Invalid column: '{column}'"));
    }
    Ok(())
}

impl Model<'_> {
    /// Returns the link attached to cell (`row`, `column`) or `None` if there isn't one.
    pub fn get_cell_link(&self, sheet: u32, row: i32, column: i32) -> Result<Option<Link>, String> {
        check_valid_cell(row, column)?;
        Ok(self
            .workbook
            .worksheet(sheet)?
            .links
            .get(&(row, column))
            .cloned())
    }

    /// Attaches `link` to cell (`row`, `column`), replacing any existing link.
    pub fn set_cell_link(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        link: Link,
    ) -> Result<(), String> {
        check_valid_cell(row, column)?;
        self.workbook
            .worksheet_mut(sheet)?
            .links
            .insert((row, column), link);
        Ok(())
    }

    /// Removes the link attached to cell (`row`, `column`). It is not an error
    /// if the cell has no link.
    pub fn delete_cell_link(&mut self, sheet: u32, row: i32, column: i32) -> Result<(), String> {
        check_valid_cell(row, column)?;
        self.workbook
            .worksheet_mut(sheet)?
            .links
            .remove(&(row, column));
        Ok(())
    }

    /// Returns all the links in the worksheet, keyed by (row, column).
    pub fn get_links(&self, sheet: u32) -> Result<&HashMap<(i32, i32), Link>, String> {
        Ok(&self.workbook.worksheet(sheet)?.links)
    }

    /// Returns all the links in the worksheet as a list sorted by (row, column).
    pub fn get_links_list(&self, sheet: u32) -> Result<Vec<CellLinkView>, String> {
        let mut list: Vec<CellLinkView> = self
            .workbook
            .worksheet(sheet)?
            .links
            .iter()
            .map(|(&(row, column), link)| CellLinkView {
                row,
                column,
                link: link.clone(),
            })
            .collect();
        list.sort_by_key(|l| (l.row, l.column));
        Ok(list)
    }
}
