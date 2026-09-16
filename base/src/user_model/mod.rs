#![deny(missing_docs)]

mod autofill;
mod border;
mod border_utils;
mod clipboard;
mod common;
mod conditional_formatting;
pub(crate) mod history;
mod links;
mod merged_cells;
mod named_cell_styles;
mod sequence_detector;
mod ui;
mod undo_redo;

pub(crate) use common::update_style;
// The collab spill tests assert on this; nothing in the library reads it.
#[cfg(test)]
pub(crate) use common::CellArrayStructure;
pub use common::UserModel;
pub use history::OrdinalUserState;

// Only the `user_model` test corpus names it, and that is ordinal-only.
#[cfg(all(test, not(feature = "collab-test")))]
pub use ui::SelectedView;

pub use clipboard::ClipboardData;
pub use common::BorderArea;
