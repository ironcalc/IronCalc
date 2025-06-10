#![deny(missing_docs)]

mod border;
mod border_utils;
mod common;
mod history;
mod event;
mod ui;

pub use common::UserModel;
pub use event::{EventEmitter, Subscription};
pub use history::Diff;

#[cfg(test)]
pub use ui::SelectedView;

pub use common::BorderArea;
pub use common::ClipboardData;
