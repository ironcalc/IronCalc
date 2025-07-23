#![deny(missing_docs)]

mod border;
mod border_utils;
mod common;
pub(crate) mod history;
mod ui;

pub use common::UserModel;

#[cfg(test)]
pub use ui::SelectedView;

pub use common::BorderArea;
pub use common::ClipboardData;
