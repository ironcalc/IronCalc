use crate::collab::fractional_index::FractionalKey;
use serde::{Deserialize, Serialize};

/// A description of a continuous range of cells, described using stable identifiers, which can be
/// used to keep track of cell position under various concurrent operations (ex. adding/removing
/// rows or columns).
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct StableRange {
    pub row_hi: FractionalKey,
    pub col_hi: FractionalKey,
    pub row_lo: FractionalKey,
    pub col_lo: FractionalKey,
}

/// A cell position, described using stable identifiers, which can be used to keep track of cell
/// position under various concurrent operations (ex. adding/removing rows or columns).
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct StableCellAddress {
    pub row: FractionalKey,
    pub col: FractionalKey,
}

impl StableCellAddress {
    pub fn new(row: FractionalKey, col: FractionalKey) -> Self {
        StableCellAddress { row, col }
    }
}
