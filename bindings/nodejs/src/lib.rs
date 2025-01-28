#[macro_use]
extern crate napi_derive;

mod model;
mod user_model;

pub use model::Model;
pub use user_model::UserModel;
