//! The cells of a worksheet.
//!
//! Everything that reads or writes the cells of a sheet goes through
//! [`SheetData`], so that how they are stored is nobody else's business.
//!
//! # How they are stored
//!
//! By position, with no hashing. Rows are grouped in chunks of `CHUNK_ROWS`
//! consecutive rows, and a chunk is allocated the first time one of its rows is
//! used, so a sheet with a single cell in its last row costs one chunk and not
//! a million empty rows. A row is a vector of `(column, cell)` sorted by column.
//!
//! A row of nothing but numbers in consecutive columns is not even that: it
//! is the numbers, eight bytes for each instead of the size of a cell, with
//! the style they share and the few that have another. Sheets of data are
//! made of such rows, and a workbook with a hundred million numbers in it is
//! mostly them. A cell of such a row exists only
//! while someone looks at it, which is why cells are handed out as
//! `Cow<Cell>`: borrowed from the sheet when the sheet holds a cell, made on
//! the spot when it holds a number. Writing anything else into the row turns
//! it into a row of cells; nothing but `set_row` turns it back.
//!
//! ```text
//!   chunks ──► [ chunk 0 | (none) | chunk 2 | ... ]        row >> CHUNK_BITS
//!                  │
//!                  ▼
//!              [ row 0 | row 1 | (none) | ... ]            row & CHUNK_MASK
//!                           │
//!                           ▼
//!                       [ (A, cell) (C, cell) (D, cell) ]  sorted by column
//! ```
//!
//! Reading a cell is two indexed steps and a short search. Rows and cells come
//! out in natural order, first by row and then by column, without sorting.
//! Evaluation reads every cell several times, and with a hash map per row each
//! read was two hashes and two jumps to unrelated places in memory, which on a
//! large sheet cost more than running the formulas.

use std::borrow::Cow;

use bitcode::{Decode, Encode};

use crate::constants::LAST_ROW;
use crate::types::Cell;

#[cfg(test)]
mod test;

const CHUNK_BITS: u32 = 10;
const CHUNK_ROWS: usize = 1 << CHUNK_BITS;
const CHUNK_MASK: usize = CHUNK_ROWS - 1;

/// Rows this short are searched from the left; longer ones by bisection.
const LINEAR_SEARCH_LIMIT: usize = 8;

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

/// The cells of a row.
#[derive(Debug, Clone)]
enum Row {
    /// Any cells, with their column, sorted by column.
    Cells(Vec<(i32, Cell)>),
    /// Numbers in consecutive columns, the first of them in `first_column`.
    Numbers {
        first_column: i32,
        /// The style of the numbers, all but those in `other_styles`.
        style: i32,
        values: Vec<f64>,
        /// The numbers of another style: where each is among `values`, and
        /// its style, in the order of `values`. A sheet of data has a number
        /// here and there shown in its own way, and is no less a sheet of
        /// numbers for it.
        other_styles: Box<[(u32, i32)]>,
    },
}

/// The style of the number at `offset` in a row of numbers.
#[inline]
fn style_at(style: i32, other_styles: &[(u32, i32)], offset: usize) -> i32 {
    if other_styles.is_empty() {
        return style;
    }
    match other_styles.binary_search_by_key(&(offset as u32), |(at, _)| *at) {
        Ok(index) => other_styles[index].1,
        Err(_) => style,
    }
}

impl Default for Row {
    fn default() -> Self {
        Row::Cells(Vec::new())
    }
}

impl Chunk {
    fn new() -> Self {
        let mut rows = Vec::new();
        rows.resize_with(CHUNK_ROWS, || None);
        Chunk { rows, used: 0 }
    }
}

/// Where the column is among the cells, or where it would go.
#[inline]
fn position(cells: &[(i32, Cell)], column: i32) -> Result<usize, usize> {
    if cells.len() <= LINEAR_SEARCH_LIMIT {
        for (index, (c, _)) in cells.iter().enumerate() {
            if *c >= column {
                return if *c == column { Ok(index) } else { Err(index) };
            }
        }
        Err(cells.len())
    } else {
        cells.binary_search_by_key(&column, |(c, _)| *c)
    }
}

/// The cells of a row from some column on, in ascending order of column.
enum RowIter<'a> {
    Cells(std::slice::Iter<'a, (i32, Cell)>),
    Numbers {
        /// The column of the next value, and where it is in the row.
        column: i32,
        offset: u32,
        style: i32,
        values: std::slice::Iter<'a, f64>,
        /// The numbers of another style from here on.
        other_styles: &'a [(u32, i32)],
    },
}

impl<'a> Iterator for RowIter<'a> {
    type Item = (i32, Cow<'a, Cell>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RowIter::Cells(cells) => cells
                .next()
                .map(|(column, cell)| (*column, Cow::Borrowed(cell))),
            RowIter::Numbers {
                column,
                offset,
                style,
                values,
                other_styles,
            } => {
                let v = *values.next()?;
                let s = match other_styles.split_first() {
                    Some(((at, other), rest)) if at == offset => {
                        *other_styles = rest;
                        *other
                    }
                    _ => *style,
                };
                let cell_column = *column;
                *column += 1;
                *offset += 1;
                Some((cell_column, Cow::Owned(Cell::NumberCell { v, s })))
            }
        }
    }
}

impl Row {
    /// The row for these cells, which are sorted by column with no column
    /// given twice: the numbers alone if that is all there is to them.
    fn from_cells(cells: Vec<(i32, Cell)>) -> Row {
        let first_column = match cells.first() {
            Some((column, Cell::NumberCell { .. })) => *column,
            _ => return Row::Cells(cells),
        };
        // The style most of them have, if most of them have one: the one
        // left standing when each is set against one that differs.
        let mut style = 0;
        let mut lead = 0;
        let mut values = Vec::with_capacity(cells.len());
        for (index, (column, cell)) in cells.iter().enumerate() {
            match cell {
                Cell::NumberCell { v, s }
                    if *column as i64 == first_column as i64 + index as i64 =>
                {
                    values.push(*v);
                    if lead == 0 {
                        style = *s;
                    }
                    lead += if *s == style { 1 } else { -1 };
                }
                _ => return Row::Cells(cells),
            }
        }
        let other_styles = cells
            .iter()
            .enumerate()
            .filter_map(|(index, (_, cell))| match cell {
                Cell::NumberCell { s, .. } if *s != style => Some((index as u32, *s)),
                _ => None,
            })
            .collect();
        Row::Numbers {
            first_column,
            style,
            values,
            other_styles,
        }
    }

    /// The cells of the row with their column, each one made if need be.
    fn to_cells(&self) -> Vec<(i32, Cell)> {
        self.iter()
            .map(|(column, cell)| (column, cell.into_owned()))
            .collect()
    }

    /// The cells of the row, to be changed: a row of numbers becomes a row of
    /// cells first.
    fn cells_mut(&mut self) -> &mut Vec<(i32, Cell)> {
        if let Row::Numbers { .. } = self {
            *self = Row::Cells(self.to_cells());
        }
        match self {
            Row::Cells(cells) => cells,
            // It has just been made into cells
            Row::Numbers { .. } => unreachable!(),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Row::Cells(cells) => cells.is_empty(),
            Row::Numbers { values, .. } => values.is_empty(),
        }
    }

    /// Where the column is among the numbers of a row of numbers.
    #[inline]
    fn offset(first_column: i32, values: &[f64], column: i32) -> Option<usize> {
        let offset = column as i64 - first_column as i64;
        if (0..values.len() as i64).contains(&offset) {
            Some(offset as usize)
        } else {
            None
        }
    }

    #[inline]
    fn get(&self, column: i32) -> Option<Cow<'_, Cell>> {
        match self {
            Row::Cells(cells) => {
                let index = position(cells, column).ok()?;
                Some(Cow::Borrowed(&cells[index].1))
            }
            Row::Numbers {
                first_column,
                style,
                values,
                other_styles,
            } => {
                let offset = Row::offset(*first_column, values, column)?;
                Some(Cow::Owned(Cell::NumberCell {
                    v: values[offset],
                    s: style_at(*style, other_styles, offset),
                }))
            }
        }
    }

    fn iter(&self) -> RowIter<'_> {
        self.iter_from(i32::MIN)
    }

    /// The cells from `first_column` on.
    #[inline]
    fn iter_from(&self, first_column: i32) -> RowIter<'_> {
        match self {
            Row::Cells(cells) => {
                let (Ok(start) | Err(start)) = position(cells, first_column);
                RowIter::Cells(cells[start..].iter())
            }
            Row::Numbers {
                first_column: row_first_column,
                style,
                values,
                other_styles,
            } => {
                let skipped = (first_column as i64 - *row_first_column as i64)
                    .clamp(0, values.len() as i64) as usize;
                let others_skipped =
                    other_styles.partition_point(|(at, _)| (*at as usize) < skipped);
                RowIter::Numbers {
                    column: *row_first_column + skipped as i32,
                    offset: skipped as u32,
                    style: *style,
                    values: values[skipped..].iter(),
                    other_styles: &other_styles[others_skipped..],
                }
            }
        }
    }

    fn insert(&mut self, column: i32, cell: Cell) -> Option<Cell> {
        match self {
            Row::Numbers {
                first_column,
                style,
                values,
                other_styles,
            } => {
                // A number in the place of one of the same style, or one of
                // the style of the row right after the last: the row stays
                // what it is.
                if let Cell::NumberCell { v, s } = &cell {
                    if let Some(offset) = Row::offset(*first_column, values, column) {
                        if *s == style_at(*style, other_styles, offset) {
                            let old = std::mem::replace(&mut values[offset], *v);
                            return Some(Cell::NumberCell { v: old, s: *s });
                        }
                    } else if s == style
                        && column as i64 == *first_column as i64 + values.len() as i64
                    {
                        values.push(*v);
                        return None;
                    }
                }
            }
            Row::Cells(cells) => {
                if let (true, Cell::NumberCell { v, s }) = (cells.is_empty(), &cell) {
                    *self = Row::Numbers {
                        first_column: column,
                        style: *s,
                        values: vec![*v],
                        other_styles: Box::default(),
                    };
                    return None;
                }
            }
        }
        let cells = self.cells_mut();
        // Rows are mostly filled from left to right.
        if cells.last().is_none_or(|(last, _)| *last < column) {
            cells.push((column, cell));
            return None;
        }
        match position(cells, column) {
            Ok(index) => Some(std::mem::replace(&mut cells[index].1, cell)),
            Err(index) => {
                cells.insert(index, (column, cell));
                None
            }
        }
    }

    fn remove(&mut self, column: i32) -> Option<Cell> {
        if let Row::Numbers {
            first_column,
            style,
            values,
            other_styles,
        } = self
        {
            let offset = Row::offset(*first_column, values, column)?;
            // The last one, if it is of the style of the row
            let last = other_styles.last().map(|(at, _)| *at as usize);
            if offset + 1 == values.len() && last != Some(offset) {
                let v = values.pop()?;
                return Some(Cell::NumberCell { v, s: *style });
            }
        }
        let cells = self.cells_mut();
        let index = position(cells, column).ok()?;
        Some(cells.remove(index).1)
    }
}

/// Two rows are equal when they have the same cells, however they hold them.
impl PartialEq for Row {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

/// A row is saved as its cells with their column, which is what a row was
/// before there were rows of numbers: a workbook saved before reads as it
/// always did, and what is saved now is what would have been saved then.
/// `bitcode` writes a type the way its encoder says, and has no public way to
/// say it by hand: these are the traits its `derive` uses.
mod saved {
    use std::num::NonZeroUsize;

    use bitcode::__private::{Buffer, Decoder, Encoder, Result, View};
    use bitcode::{Decode, Encode};

    use super::Row;
    use crate::types::Cell;

    type SavedRow = Vec<(i32, Cell)>;

    #[derive(Default)]
    pub struct RowEncoder(<SavedRow as Encode>::Encoder);

    impl Buffer for RowEncoder {
        fn collect_into(&mut self, out: &mut Vec<u8>) {
            self.0.collect_into(out);
        }

        fn reserve(&mut self, additional: NonZeroUsize) {
            self.0.reserve(additional);
        }
    }

    impl Encoder<Row> for RowEncoder {
        fn encode(&mut self, row: &Row) {
            match row {
                Row::Cells(cells) => self.0.encode(cells),
                Row::Numbers { .. } => self.0.encode(&row.to_cells()),
            }
        }
    }

    impl Encode for Row {
        type Encoder = RowEncoder;
    }

    #[derive(Default)]
    pub struct RowDecoder<'a>(<SavedRow as Decode<'a>>::Decoder);

    impl<'a> View<'a> for RowDecoder<'a> {
        fn populate(&mut self, input: &mut &'a [u8], length: usize) -> Result<()> {
            self.0.populate(input, length)
        }
    }

    impl<'a> Decoder<'a, Row> for RowDecoder<'a> {
        fn decode(&mut self) -> Row {
            Row::from_cells(self.0.decode())
        }
    }

    impl<'a> Decode<'a> for Row {
        type Decoder = RowDecoder<'a>;
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
    fn row(&self, row: i32) -> Option<&Row> {
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
        let row = self.row_mut(row)?;
        // A row of numbers stays one if the cell is not there
        row.get(column)?;
        let cells = row.cells_mut();
        let index = position(cells, column).ok()?;
        Some(&mut cells[index].1)
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

    /// Removes every cell of a row, and the row.
    pub fn remove_row(&mut self, row: i32) {
        let Some((chunk_index, index)) = locate(row) else {
            return;
        };
        let Some(Some(chunk)) = self.chunks.get_mut(chunk_index) else {
            return;
        };
        if chunk.rows[index].take().is_some() {
            chunk.used -= 1;
            self.row_count -= 1;
            if chunk.used == 0 {
                self.chunks[chunk_index] = None;
            }
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
