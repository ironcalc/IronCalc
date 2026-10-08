//! A row of cells, and how it holds them: as cells, or as nothing but the
//! numbers when that is all there is.

use std::borrow::Cow;

use crate::types::Cell;

/// Rows this short are searched from the left; longer ones by bisection.
const LINEAR_SEARCH_LIMIT: usize = 8;

/// The cells of a row.
#[derive(Debug, Clone)]
pub(super) enum Row {
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
pub(super) enum RowIter<'a> {
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
    pub(super) fn from_cells(cells: Vec<(i32, Cell)>) -> Row {
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
    pub(super) fn to_cells(&self) -> Vec<(i32, Cell)> {
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

    /// The cell at a column, to be changed in place. A row of numbers becomes
    /// a row of cells first, if the cell is there at all.
    pub(super) fn get_mut(&mut self, column: i32) -> Option<&mut Cell> {
        // A row of numbers stays one if the cell is not there
        self.get(column)?;
        let cells = self.cells_mut();
        let index = position(cells, column).ok()?;
        Some(&mut cells[index].1)
    }

    pub(super) fn is_empty(&self) -> bool {
        match self {
            Row::Cells(cells) => cells.is_empty(),
            Row::Numbers { values, .. } => values.is_empty(),
        }
    }

    /// Where the column is among the numbers of a row of numbers.
    #[inline]
    pub(super) fn offset(first_column: i32, values: &[f64], column: i32) -> Option<usize> {
        let offset = column as i64 - first_column as i64;
        if (0..values.len() as i64).contains(&offset) {
            Some(offset as usize)
        } else {
            None
        }
    }

    #[inline]
    pub(super) fn get(&self, column: i32) -> Option<Cow<'_, Cell>> {
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

    pub(super) fn iter(&self) -> RowIter<'_> {
        self.iter_from(i32::MIN)
    }

    /// The cells from `first_column` on.
    #[inline]
    pub(super) fn iter_from(&self, first_column: i32) -> RowIter<'_> {
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

    pub(super) fn insert(&mut self, column: i32, cell: Cell) -> Option<Cell> {
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

    pub(super) fn remove(&mut self, column: i32) -> Option<Cell> {
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
