//! The running app: the window and its event loop, signals, the platform
//! integrations, performance and hot reload.

#[cfg(hot_reload)]
pub mod hot_reload;
pub mod perf;
pub mod platform;
pub mod signal;
pub mod window;
