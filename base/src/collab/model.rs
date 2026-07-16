use crate::collab::fractional_index::{FractionalIndex, FractionalKey};

/// A description of a continuous range of cells, described using stable identifiers, which can be
/// used to keep track of cell position under various concurrent operations (ex. adding/removing
/// rows or columns).
pub struct StableRange {
    pub row_hi: FractionalKey,
    pub row_lo: FractionalKey,
    pub col_hi: FractionalKey,
    pub col_lo: FractionalKey,
}

/// A cell position, described using stable identifiers, which can be used to keep track of cell
/// position under various concurrent operations (ex. adding/removing rows or columns).
pub struct StableCellAddress {
    pub row: FractionalKey,
    pub col: FractionalKey,
}

impl StableCellAddress {
    pub fn new(row: FractionalKey, col: FractionalKey) -> Self {
        StableCellAddress { row, col }
    }
}
