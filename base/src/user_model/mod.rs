#![deny(missing_docs)]

mod common;
mod history;
mod ui;

pub use common::UserModel;

#[cfg(test)]
pub use ui::SelectedView;

pub use common::BorderArea;
