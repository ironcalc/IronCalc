#![deny(missing_docs)]

mod border;
mod border_utils;
mod common;
mod conditional_formatting;
pub(crate) mod history;
mod named_cell_styles;
mod sequence_detector;
mod ui;

pub use common::UserModel;

#[cfg(test)]
pub use ui::SelectedView;

pub use common::BorderArea;
pub use common::ClipboardData;
