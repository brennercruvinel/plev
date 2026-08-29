//! .nest backend for the UI: typed view models over the database, plus a
//! worker so the event loop never blocks on search or decode.
//!
//! Layout:
//! - [`types`]: target-independent view models and the command/event
//!   vocabulary every backend speaks.
//! - [`nestread`]: portable in-memory .nest reader (all targets; the web
//!   backend's engine, unit-tested natively against the real writer).
//! - [`graph`], [`bench`]: pure geometry/stats (all targets).
//! - [`backend`] + [`worker`] (native): mmap runtime + worker thread.
//! - [`web`] (wasm): inline worker over `nestread`.
//!
//! The explorer talks to [`Worker`], the platform alias with an identical
//! `spawn`/`send`/`try_recv` surface.

pub mod bench;
pub mod graph;
pub mod nestread;
pub mod types;

#[cfg(not(target_arch = "wasm32"))]
pub mod backend;
#[cfg(not(target_arch = "wasm32"))]
pub mod embed;
#[cfg(not(target_arch = "wasm32"))]
pub mod recents;
#[cfg(not(target_arch = "wasm32"))]
pub mod worker;

#[cfg(target_arch = "wasm32")]
pub mod web;

#[cfg(target_arch = "wasm32")]
pub use web::WebWorker as Worker;
/// The platform worker: a background thread on native, synchronous
/// inline execution on the web (wasm32-unknown-unknown has no threads).
#[cfg(not(target_arch = "wasm32"))]
pub use worker::NestWorker as Worker;
