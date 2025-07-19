#![deny(missing_docs)]

mod border;
mod border_utils;
pub mod common;
pub mod event;
pub mod history;
pub mod ui;

pub use common::UserModel;

#[cfg(test)]
pub use ui::SelectedView;

pub use common::BorderArea;
