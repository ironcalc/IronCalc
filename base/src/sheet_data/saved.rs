//! A row is saved as its cells with their column, which is what a row was
//! before there were rows of numbers: a workbook saved before reads as it
//! always did, and what is saved now is what would have been saved then.
//! `bitcode` writes a type the way its encoder says, and has no public way to
//! say it by hand: these are the traits its `derive` uses.

use std::num::NonZeroUsize;

use bitcode::__private::{Buffer, Decoder, Encoder, Result, View};
use bitcode::{Decode, Encode};

use super::row::Row;
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
