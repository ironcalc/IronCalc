#[cfg(target_arch = "wasm32")]
mod browser_tz;
#[cfg(target_arch = "wasm32")]
pub use browser_tz::*;

#[cfg(not(target_arch = "wasm32"))]
mod os_tz;
#[cfg(not(target_arch = "wasm32"))]
pub use os_tz::*;
