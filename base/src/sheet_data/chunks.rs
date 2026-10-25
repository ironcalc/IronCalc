//! The rows of a sheet, by position: chunks of consecutive rows, allocated as
//! they are needed.

use std::borrow::Cow;

use bitcode::{Decode, Encode};

use super::row::Row;
use crate::constants::LAST_ROW;
use crate::types::Cell;

const CHUNK_BITS: u32 = 10;
const CHUNK_ROWS: usize = 1 << CHUNK_BITS;
const CHUNK_MASK: usize = CHUNK_ROWS - 1;

/// The cells of a worksheet, by row and column. Both are 1-indexed.
///
/// A row can exist without cells: `set_row` with no cells creates one. The xlsx
/// importer does that for rows that carry a height or a style and nothing
/// else, and the exporter writes a `<row>` element for every row that exists
/// here, so such rows survive a round trip. Removing the last cell of a row
/// with `remove_cell` removes the row.
///
/// Rows outside the sheet, below 1 or beyond the last row, cannot hold cells:
/// writing to them does nothing and reading them finds nothing.
#[derive(Encode, Decode, Debug, Clone, Default)]
pub struct SheetData {
    /// `chunks[i]` holds rows `i * CHUNK_ROWS ..`, if any of them exists.
    chunks: Vec<Option<Box<Chunk>>>,
    /// How many rows exist.
    row_count: usize,
}

#[derive(Encode, Decode, Debug, Clone)]
struct Chunk {
    /// Always `CHUNK_ROWS` long.
    rows: Vec<Option<Row>>,
    /// How many of them exist. A chunk with none is freed.
    used: usize,
}

impl Chunk {
    fn new() -> Self {
        let mut rows = Vec::new();
        rows.resize_with(CHUNK_ROWS, || None);
        Chunk { rows, used: 0 }
    }
}

/// The chunk and the place in it of a row, if the row is within the sheet.
#[inline]
fn locate(row: i32) -> Option<(usize, usize)> {
    if !(1..=LAST_ROW).contains(&row) {
        return None;
    }
    let row = row as usize;
    Some((row >> CHUNK_BITS, row & CHUNK_MASK))
}

impl SheetData {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub(super) fn row(&self, row: i32) -> Option<&Row> {
        let (chunk, index) = locate(row)?;
        self.chunks.get(chunk)?.as_ref()?.rows[index].as_ref()
    }

    #[inline]
    fn row_mut(&mut self, row: i32) -> Option<&mut Row> {
        let (chunk, index) = locate(row)?;
        self.chunks.get_mut(chunk)?.as_mut()?.rows[index].as_mut()
    }

    /// The row, created if it does not exist. `None` outside the sheet.
    fn row_or_insert(&mut self, row: i32) -> Option<&mut Row> {
        let (chunk, index) = locate(row)?;
        if chunk >= self.chunks.len() {
            self.chunks.resize_with(chunk + 1, || None);
        }
        let chunk = self.chunks[chunk].get_or_insert_with(|| Box::new(Chunk::new()));
        if chunk.rows[index].is_none() {
            chunk.rows[index] = Some(Row::default());
            chunk.used += 1;
            self.row_count += 1;
        }
        chunk.rows[index].as_mut()
    }

    /// Every row that exists with its number, in ascending order.
    fn rows_in_order(&self) -> impl Iterator<Item = (i32, &Row)> {
        self.chunks
            .iter()
            .enumerate()
            .filter_map(|(chunk_index, chunk)| Some((chunk_index, chunk.as_ref()?)))
            .flat_map(|(chunk_index, chunk)| {
                chunk
                    .rows
                    .iter()
                    .enumerate()
                    .filter_map(move |(index, row)| {
                        let number = (chunk_index << CHUNK_BITS) | index;
                        Some((number as i32, row.as_ref()?))
                    })
            })
    }

    /// The cell at a position, if there is one.
    #[inline]
    pub fn cell(&self, row: i32, column: i32) -> Option<Cow<'_, Cell>> {
        self.row(row)?.get(column)
    }

    /// The cell at a position, to be changed in place.
    pub fn cell_mut(&mut self, row: i32, column: i32) -> Option<&mut Cell> {
        self.row_mut(row)?.get_mut(column)
    }

    /// Puts a cell at a position and returns the one that was there.
    pub fn set_cell(&mut self, row: i32, column: i32, cell: Cell) -> Option<Cell> {
        self.row_or_insert(row)?.insert(column, cell)
    }

    /// Removes the cell at a position and returns it.
    pub fn remove_cell(&mut self, row: i32, column: i32) -> Option<Cell> {
        let row_data = self.row_mut(row)?;
        let cell = row_data.remove(column)?;
        if row_data.is_empty() {
            self.remove_row(row);
        }
        Some(cell)
    }

    /// True if the sheet has no rows.
    pub fn is_empty(&self) -> bool {
        self.row_count == 0
    }

    /// Every cell of the sheet with its row and column, in natural order:
    /// by row, and within a row by column.
    pub fn cells(&self) -> impl Iterator<Item = (i32, i32, Cow<'_, Cell>)> {
        self.rows_in_order().flat_map(|(row, row_data)| {
            row_data
                .iter()
                .map(move |(column, cell)| (row, column, cell))
        })
    }

    /// The rows that exist, in ascending order.
    pub fn rows(&self) -> Vec<i32> {
        self.rows_in_order().map(|(row, _)| row).collect()
    }

    /// The columns of a row that have cells, in ascending order.
    pub fn columns_in_row(&self, row: i32) -> Vec<i32> {
        match self.row(row) {
            Some(row_data) => row_data.iter().map(|(column, _)| column).collect(),
            None => Vec::new(),
        }
    }

    /// The cells of a row with their column, in ascending order of column.
    pub fn cells_in_row(&self, row: i32) -> Vec<(i32, Cow<'_, Cell>)> {
        match self.row(row) {
            Some(row_data) => row_data.iter().collect(),
            None => Vec::new(),
        }
    }

    /// The cells of a row from `first_column` to `last_column`, both included,
    /// with their column, in ascending order of column. Reading a stretch of a
    /// row this way finds the row once, not once per cell.
    pub fn cells_in_row_between(
        &self,
        row: i32,
        first_column: i32,
        last_column: i32,
    ) -> impl Iterator<Item = (i32, Cow<'_, Cell>)> {
        self.row(row)
            .map(|row_data| row_data.iter_from(first_column))
            .into_iter()
            .flatten()
            .take_while(move |(column, _)| *column <= last_column)
    }

    /// The numbers of a row from `first_column` to `last_column`, both
    /// included, if the row holds them as numbers and nothing else, with none
    /// missing. `None` is no news about the row: its cells say what it holds.
    #[inline]
    pub fn numbers_in_row_between(
        &self,
        row: i32,
        first_column: i32,
        last_column: i32,
    ) -> Option<&[f64]> {
        match self.row(row)? {
            Row::Numbers {
                first_column: row_first_column,
                values,
                ..
            } => {
                let first = Row::offset(*row_first_column, values, first_column)?;
                let last = Row::offset(*row_first_column, values, last_column)?;
                values.get(first..=last)
            }
            Row::Cells(_) => None,
        }
    }

    /// Removes every cell of a row, and the row.
    pub fn remove_row(&mut self, row: i32) {
        self.take_row(row);
    }

    /// Takes a row out of the sheet. The row no longer exists afterwards.
    fn take_row(&mut self, row: i32) -> Option<Row> {
        let (chunk_index, index) = locate(row)?;
        let chunk = self.chunks.get_mut(chunk_index)?.as_mut()?;
        let taken = chunk.rows[index].take()?;
        chunk.used -= 1;
        self.row_count -= 1;
        if chunk.used == 0 {
            self.chunks[chunk_index] = None;
        }
        Some(taken)
    }

    /// Puts a row in the sheet in place of whatever was there. Outside the
    /// sheet it is dropped.
    fn put_row(&mut self, row: i32, data: Row) {
        if let Some(slot) = self.row_or_insert(row) {
            *slot = data;
        }
    }

    /// Moves every row from `first_row` on by `delta` rows: row `r` becomes
    /// row `r + delta`, cells and all. This is what inserting rows above
    /// `first_row` does, and deleting them: with a negative `delta` the rows
    /// the others land on, `first_row + delta` to `first_row - 1`, are removed
    /// first. A row that would land beyond the last row is dropped. The sheet
    /// is not read cell by cell: a row moves in one piece.
    pub fn displace_rows(&mut self, first_row: i32, delta: i32) {
        if delta == 0 {
            return;
        }
        let rows: Vec<i32> = self
            .rows_in_order()
            .map(|(row, _)| row)
            .filter(|row| *row >= first_row + delta.min(0))
            .collect();
        if delta > 0 {
            // From the bottom up, so that a row lands on a row already moved
            for row in rows.into_iter().rev() {
                if let Some(data) = self.take_row(row) {
                    self.put_row(row + delta, data);
                }
            }
        } else {
            for row in rows {
                if row < first_row {
                    self.remove_row(row);
                } else if let Some(data) = self.take_row(row) {
                    self.put_row(row + delta, data);
                }
            }
        }
    }

    /// Moves every cell from `first_column` on by `delta` columns, in every
    /// row: the cell in column `c` goes to column `c + delta`. This is what
    /// inserting columns before `first_column` does, and deleting them: with a
    /// negative `delta` the cells in the columns the others land on,
    /// `first_column + delta` to `first_column - 1`, are removed first. A row
    /// left with no cells by that is removed, as it is when its last cell is
    /// removed. A cell that would land beyond the last column is dropped.
    pub fn displace_columns(&mut self, first_column: i32, delta: i32) {
        if delta == 0 {
            return;
        }
        let mut emptied = Vec::new();
        for (chunk_index, chunk) in self.chunks.iter_mut().enumerate() {
            let Some(chunk) = chunk else {
                continue;
            };
            for (index, row) in chunk.rows.iter_mut().enumerate() {
                let Some(row_data) = row else {
                    continue;
                };
                let was_empty = row_data.is_empty();
                row_data.displace_columns(first_column, delta);
                if !was_empty && row_data.is_empty() {
                    emptied.push(((chunk_index << CHUNK_BITS) | index) as i32);
                }
            }
        }
        for row in emptied {
            self.remove_row(row);
        }
    }

    /// Replaces the cells of a row. The row exists afterwards, even with no
    /// cells. If a column is given twice the last one stays.
    pub fn set_row(&mut self, row: i32, cells: impl IntoIterator<Item = (i32, Cell)>) {
        let Some(row_data) = self.row_or_insert(row) else {
            return;
        };
        // Collected at its exact size: a row grown cell by cell from empty
        // starts with room for four, and a sheet with a million short rows
        // paid for that four times over.
        let mut cells: Vec<(i32, Cell)> = cells.into_iter().collect();
        // A file gives the cells of a row from left to right, and then there
        // is nothing to sort and no column given twice.
        if !cells.is_sorted_by(|earlier, later| earlier.0 < later.0) {
            cells.sort_by_key(|(column, _)| *column);
            // Of two cells in the same column the later one stays: it is moved
            // into the slot of the earlier one, which `dedup_by` then keeps.
            cells.dedup_by(|later, earlier| {
                if later.0 == earlier.0 {
                    std::mem::swap(&mut later.1, &mut earlier.1);
                    true
                } else {
                    false
                }
            });
        }
        cells.shrink_to_fit();
        *row_data = Row::from_cells(cells);
    }
}

/// Two sheets are equal when they have the same rows and the same cells.
/// Which chunks happen to be allocated is not part of that.
impl PartialEq for SheetData {
    fn eq(&self, other: &Self) -> bool {
        self.row_count == other.row_count && self.rows_in_order().eq(other.rows_in_order())
    }
}
