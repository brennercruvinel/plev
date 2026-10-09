//! The threads and processes the UI never blocks on: the worker, its web
//! twin, the embedder bridge, subprocesses and the messages between them.

#[cfg(not(target_arch = "wasm32"))]
pub mod embed;
pub mod messages;
#[cfg(not(target_arch = "wasm32"))]
pub mod subprocess;
#[cfg(target_arch = "wasm32")]
pub mod web;
#[cfg(not(target_arch = "wasm32"))]
pub mod worker;
