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

mod chunks;
mod row;
mod saved;
#[cfg(test)]
mod test;

pub use chunks::SheetData;
