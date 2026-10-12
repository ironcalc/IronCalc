#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod browser_tz;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub(crate) use browser_tz::*;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
mod os_tz;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub(crate) use os_tz::*;
