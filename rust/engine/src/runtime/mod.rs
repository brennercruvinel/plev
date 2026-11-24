//! The running app: the window and its event loop, signals, the platform
//! integrations, performance, the dev mode and hot reload.

pub mod dev;
#[cfg(hot_reload)]
pub mod hot_reload;
pub mod perf;
pub mod platform;
pub mod signal;
pub mod window;
