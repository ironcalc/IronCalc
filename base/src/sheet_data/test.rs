#![allow(clippy::unwrap_used)]

use std::borrow::Cow;

use super::row::Row;
use super::*;
use crate::constants::LAST_ROW;
use crate::types::Cell;

fn number(v: f64) -> Cell {
    Cell::NumberCell { v, s: 0 }
}

#[test]
fn a_cell_stays_small() {
    // A sheet of numbers costs this much per cell, whatever the cell holds:
    // anything that makes the largest kind of cell larger makes them all so.
    assert!(std::mem::size_of::<Cell>() <= 48);
    assert!(std::mem::size_of::<(i32, Cell)>() <= 56);
}

#[test]
fn set_get_replace_remove() {
    let mut data = SheetData::new();
    assert!(data.is_empty());
    assert_eq!(data.set_cell(3, 2, number(1.0)), None);
    assert_eq!(data.set_cell(3, 2, number(2.0)), Some(number(1.0)));
    assert_eq!(data.cell(3, 2).as_deref(), Some(&number(2.0)));
    assert_eq!(data.cell(3, 1), None);
    assert_eq!(data.cell(4, 2), None);
    *data.cell_mut(3, 2).unwrap() = number(5.0);
    assert_eq!(data.remove_cell(3, 2), Some(number(5.0)));
    assert_eq!(data.remove_cell(3, 2), None);
    // the last cell of the row took the row with it
    assert!(data.is_empty());
    assert!(data.rows().is_empty());
}

#[test]
fn natural_order_whatever_the_order_of_insertion() {
    let mut data = SheetData::new();
    // two chunks, and a row longer than the linear search limit, filled
    // from the right
    for column in (1..=20).rev() {
        data.set_cell(2000, column, number(column as f64));
    }
    data.set_cell(5, 7, number(0.0));
    data.set_cell(5, 3, number(0.0));
    data.set_cell(1, 9, number(0.0));
    let positions: Vec<(i32, i32)> = data.cells().map(|(r, c, _)| (r, c)).collect();
    let mut sorted = positions.clone();
    sorted.sort_unstable();
    assert_eq!(positions, sorted);
    assert_eq!(positions.len(), 23);
    assert_eq!(data.rows(), vec![1, 5, 2000]);
    assert_eq!(data.columns_in_row(5), vec![3, 7]);
    assert_eq!(data.cell(2000, 13).as_deref(), Some(&number(13.0)));
    assert_eq!(data.cells_in_row(2000).len(), 20);
    assert!(data.columns_in_row(6).is_empty());
}

#[test]
fn a_stretch_of_a_row() {
    let mut data = SheetData::new();
    // a short row, searched from the left, and a long one, bisected
    for column in [2, 3, 5, 9] {
        data.set_cell(1, column, number(column as f64));
    }
    for column in (10..=60).step_by(2) {
        data.set_cell(7, column, number(column as f64));
    }
    let columns = |row, first, last| -> Vec<i32> {
        data.cells_in_row_between(row, first, last)
            .map(|(column, _)| column)
            .collect()
    };
    assert_eq!(columns(1, 1, 100), vec![2, 3, 5, 9]);
    assert_eq!(columns(1, 3, 5), vec![3, 5]);
    assert_eq!(columns(1, 4, 4), Vec::<i32>::new());
    assert_eq!(columns(1, 6, 8), Vec::<i32>::new());
    assert_eq!(columns(1, 10, 20), Vec::<i32>::new());
    assert_eq!(columns(7, 11, 17), vec![12, 14, 16]);
    assert_eq!(columns(7, 58, 1000), vec![58, 60]);
    // no such row
    assert_eq!(columns(2, 1, 100), Vec::<i32>::new());
    assert_eq!(
        data.cells_in_row_between(7, 20, 20).next(),
        Some((20, Cow::Owned(number(20.0))))
    );
}

fn is_numbers(data: &SheetData, row: i32) -> bool {
    matches!(data.row(row), Some(Row::Numbers { .. }))
}

fn text(si: i32) -> Cell {
    Cell::SharedString { si, s: 0 }
}

#[test]
fn a_row_of_numbers_is_held_as_numbers() {
    let mut data = SheetData::new();
    // consecutive numbers of one style
    data.set_row(1, (3..=6).map(|column| (column, number(column as f64))));
    assert!(is_numbers(&data, 1));
    // a gap, something that is not a number, no cells
    data.set_row(2, vec![(1, number(1.0)), (3, number(3.0))]);
    data.set_row(4, vec![(1, number(1.0)), (2, text(0))]);
    data.set_row(5, Vec::new());
    for row in [2, 4, 5] {
        assert!(!is_numbers(&data, row), "row {row}");
    }
    // whatever the style, as long as it is one
    data.set_row(
        6,
        (1..=3).map(|column| (column, Cell::NumberCell { v: 0.5, s: 7 })),
    );
    assert!(is_numbers(&data, 6));
    assert_eq!(
        data.cell(6, 2).as_deref(),
        Some(&Cell::NumberCell { v: 0.5, s: 7 })
    );

    // It reads like any other row
    assert_eq!(data.cell(1, 2), None);
    assert_eq!(data.cell(1, 3).as_deref(), Some(&number(3.0)));
    assert_eq!(data.cell(1, 6).as_deref(), Some(&number(6.0)));
    assert_eq!(data.cell(1, 7), None);
    assert_eq!(data.columns_in_row(1), vec![3, 4, 5, 6]);
    assert_eq!(data.cells_in_row(1).len(), 4);
    let between = |first, last| -> Vec<i32> {
        data.cells_in_row_between(1, first, last)
            .map(|(column, _)| column)
            .collect()
    };
    assert_eq!(between(1, 100), vec![3, 4, 5, 6]);
    assert_eq!(between(4, 5), vec![4, 5]);
    assert_eq!(between(6, 9), vec![6]);
    assert_eq!(between(7, 9), Vec::<i32>::new());
    assert_eq!(between(1, 2), Vec::<i32>::new());
    let cells: Vec<(i32, i32)> = data.cells().map(|(r, c, _)| (r, c)).take(5).collect();
    assert_eq!(cells, vec![(1, 3), (1, 4), (1, 5), (1, 6), (2, 1)]);

    // Some of the numbers can have a style of their own
    let styles = [0, 3, 0, 0, 0, 3, 5, 0];
    let styled = |style: i32, column: i32| Cell::NumberCell {
        v: column as f64,
        s: style,
    };
    let row: Vec<(i32, Cell)> = (10..)
        .zip(styles)
        .map(|(column, style)| (column, styled(style, column)))
        .collect();
    data.set_row(8, row.clone());
    assert!(is_numbers(&data, 8));
    for (column, cell) in &row {
        assert_eq!(data.cell(8, *column).as_deref(), Some(cell));
    }
    for first in 9..=18 {
        let read: Vec<(i32, Cell)> = data
            .cells_in_row_between(8, first, 16)
            .map(|(column, cell)| (column, cell.into_owned()))
            .collect();
        let expected: Vec<(i32, Cell)> = row
            .iter()
            .filter(|(column, _)| (first..=16).contains(column))
            .cloned()
            .collect();
        assert_eq!(read, expected, "from {first}");
    }
    assert_eq!(
        data.numbers_in_row_between(8, 11, 13),
        Some(&[11.0, 12.0, 13.0][..])
    );
    // a number of the style there is in its place: still numbers
    assert_eq!(data.set_cell(8, 11, styled(3, 0)), Some(styled(3, 11)));
    assert_eq!(data.set_cell(8, 12, styled(0, 0)), Some(styled(0, 12)));
    assert!(is_numbers(&data, 8));
    assert_eq!(data.cell(8, 11).as_deref(), Some(&styled(3, 0)));
    // of another style: cells, the others as they were
    assert_eq!(data.set_cell(8, 13, styled(3, 13)), Some(styled(0, 13)));
    assert!(!is_numbers(&data, 8));
    assert_eq!(data.cell(8, 15).as_deref(), Some(&styled(3, 15)));
    assert_eq!(data.cell(8, 16).as_deref(), Some(&styled(5, 16)));
    // most of them in a style that is not the first one's
    data.set_row(
        9,
        vec![(1, styled(4, 1)), (2, styled(2, 2)), (3, styled(2, 3))],
    );
    assert!(is_numbers(&data, 9));
    assert_eq!(data.cell(9, 1).as_deref(), Some(&styled(4, 1)));
    assert_eq!(data.cell(9, 3).as_deref(), Some(&styled(2, 3)));
    // the last one goes only if it is of the style of the row
    assert_eq!(data.remove_cell(9, 3), Some(styled(2, 3)));
    assert!(is_numbers(&data, 9));
    data.set_row(
        9,
        vec![(1, styled(2, 1)), (2, styled(2, 2)), (3, styled(4, 3))],
    );
    assert_eq!(data.remove_cell(9, 3), Some(styled(4, 3)));
    assert!(!is_numbers(&data, 9));
    assert_eq!(data.columns_in_row(9), vec![1, 2]);

    // The numbers themselves, when the row holds all that is asked for
    assert_eq!(
        data.numbers_in_row_between(1, 4, 6),
        Some(&[4.0, 5.0, 6.0][..])
    );
    assert_eq!(data.numbers_in_row_between(1, 3, 3), Some(&[3.0][..]));
    assert_eq!(data.numbers_in_row_between(1, 2, 6), None);
    assert_eq!(data.numbers_in_row_between(1, 3, 7), None);
    assert_eq!(data.numbers_in_row_between(2, 1, 1), None);
    assert_eq!(data.numbers_in_row_between(9, 1, 1), None);
}

#[test]
fn a_row_of_numbers_stays_one_only_while_it_can() {
    let mut data = SheetData::new();
    // filled cell by cell, from the left
    for column in 1..=4 {
        assert_eq!(data.set_cell(1, column, number(column as f64)), None);
    }
    assert!(is_numbers(&data, 1));
    // another number in the place of one, and the next one to the right
    assert_eq!(data.set_cell(1, 2, number(20.0)), Some(number(2.0)));
    assert_eq!(data.set_cell(1, 5, number(5.0)), None);
    assert!(is_numbers(&data, 1));
    // the last one goes, and a cell that is not there
    assert_eq!(data.remove_cell(1, 5), Some(number(5.0)));
    assert_eq!(data.remove_cell(1, 9), None);
    assert!(data.cell_mut(1, 9).is_none());
    assert!(is_numbers(&data, 1));
    let expected = vec![
        (1, number(1.0)),
        (2, number(20.0)),
        (3, number(3.0)),
        (4, number(4.0)),
    ];
    let cells = |data: &SheetData| -> Vec<(i32, Cell)> {
        data.cells_in_row(1)
            .into_iter()
            .map(|(column, cell)| (column, cell.into_owned()))
            .collect()
    };
    assert_eq!(cells(&data), expected);

    // Everything else makes it a row of cells, with the same cells in it
    let changes: [fn(&mut SheetData); 6] = [
        |data| {
            data.set_cell(1, 2, Cell::SharedString { si: 0, s: 0 });
            data.set_cell(1, 2, Cell::NumberCell { v: 20.0, s: 0 });
        },
        |data| {
            data.set_cell(1, 3, Cell::NumberCell { v: 3.0, s: 9 });
            data.cell_mut(1, 3).unwrap().set_style(0);
        },
        |data| {
            data.set_cell(1, 7, Cell::NumberCell { v: 7.0, s: 0 });
            data.remove_cell(1, 7);
        },
        |data| {
            data.set_cell(1, 0, Cell::NumberCell { v: 0.0, s: 0 });
            data.remove_cell(1, 0);
        },
        |data| {
            let removed = data.remove_cell(1, 1).unwrap();
            data.set_cell(1, 1, removed);
        },
        |data| {
            *data.cell_mut(1, 4).unwrap() = Cell::NumberCell { v: 4.0, s: 0 };
        },
    ];
    for (index, change) in changes.iter().enumerate() {
        let mut changed = data.clone();
        change(&mut changed);
        assert!(!is_numbers(&changed, 1), "change {index}");
        assert_eq!(cells(&changed), expected, "change {index}");
        // and the two are the same sheet
        assert_eq!(changed, data, "change {index}");
    }

    // The last number of a row takes the row with it
    let mut data = SheetData::new();
    data.set_cell(3, 3, number(1.0));
    assert!(is_numbers(&data, 3));
    assert_eq!(data.remove_cell(3, 3), Some(number(1.0)));
    assert!(data.is_empty());
}

// A row is saved as its cells, however it is held: a sheet saved before
// there were rows of numbers reads as it did, and saves as it did.
#[test]
fn a_row_of_numbers_is_saved_as_its_cells() {
    let mut numbers = SheetData::new();
    numbers.set_row(
        2,
        (1..=40).map(|column| (column, number(column as f64 / 7.0))),
    );
    numbers.set_row(3, vec![(1, text(1)), (2, number(2.0))]);
    numbers.set_row(
        2000,
        (5..=9).map(|column| {
            (
                column,
                Cell::NumberCell {
                    v: 1.5,
                    s: column % 3,
                },
            )
        }),
    );
    assert!(is_numbers(&numbers, 2) && is_numbers(&numbers, 2000));
    // the same sheet with no row held as numbers
    let mut cells = numbers.clone();
    for row in [2, 2000] {
        let column = cells.columns_in_row(row)[0];
        let cell = cells.cell(row, column).unwrap().into_owned();
        *cells.cell_mut(row, column).unwrap() = cell;
        assert!(!is_numbers(&cells, row));
    }
    assert_eq!(numbers, cells);
    let saved = bitcode::encode(&numbers);
    assert_eq!(saved, bitcode::encode(&cells));
    // what a row was: a struct with the cells and their column
    #[derive(bitcode::Encode)]
    struct OldRow {
        cells: Vec<(i32, Cell)>,
    }
    let row = |data: &SheetData, row: i32| -> Vec<(i32, Cell)> {
        data.cells_in_row(row)
            .into_iter()
            .map(|(column, cell)| (column, cell.into_owned()))
            .collect()
    };
    assert_eq!(
        bitcode::encode(data_row(&numbers, 2)),
        bitcode::encode(&OldRow {
            cells: row(&numbers, 2)
        })
    );
    let read: SheetData = bitcode::decode(&saved).unwrap();
    assert_eq!(read, numbers);
    assert!(is_numbers(&read, 2) && is_numbers(&read, 2000));
    assert!(!is_numbers(&read, 3));
}

fn data_row(data: &SheetData, row: i32) -> &Row {
    data.row(row).unwrap()
}

#[test]
fn a_row_can_exist_without_cells() {
    let mut data = SheetData::new();
    data.set_row(10, Vec::new());
    assert!(!data.is_empty());
    assert_eq!(data.rows(), vec![10]);
    assert_eq!(data.cells().count(), 0);
    data.remove_row(10);
    assert!(data.is_empty());
}

#[test]
fn set_row_replaces_and_the_last_duplicate_stays() {
    let mut data = SheetData::new();
    data.set_cell(4, 1, number(1.0));
    data.set_row(
        4,
        vec![(5, number(5.0)), (2, number(2.0)), (5, number(6.0))],
    );
    assert_eq!(data.columns_in_row(4), vec![2, 5]);
    assert_eq!(data.cell(4, 5).as_deref(), Some(&number(6.0)));
    assert_eq!(data.cell(4, 1), None);
}

#[test]
fn rows_outside_the_sheet_hold_nothing() {
    let mut data = SheetData::new();
    for row in [0, -3, LAST_ROW + 1, i32::MAX, i32::MIN] {
        assert_eq!(data.set_cell(row, 1, number(1.0)), None);
        data.set_row(row, vec![(1, number(1.0))]);
        assert_eq!(data.cell(row, 1), None);
        data.remove_row(row);
    }
    assert!(data.is_empty());
    // the first and the last row of the sheet are fine
    data.set_cell(1, 1, number(1.0));
    data.set_cell(LAST_ROW, 1, number(2.0));
    assert_eq!(data.rows(), vec![1, LAST_ROW]);
}

#[test]
fn equality_is_about_cells_not_about_chunks() {
    let mut a = SheetData::new();
    let mut b = SheetData::new();
    a.set_cell(1, 1, number(1.0));
    b.set_cell(1, 1, number(1.0));
    // `a` has had a cell far away, and no longer has
    a.set_cell(900_000, 4, number(9.0));
    a.remove_cell(900_000, 4);
    assert_eq!(a, b);
    b.set_cell(1, 2, number(1.0));
    assert_ne!(a, b);
}
