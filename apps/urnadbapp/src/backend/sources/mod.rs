//! Where the bytes of a .urna come from: the native backend, the portable
//! reader and the media frames.

#[cfg(not(target_arch = "wasm32"))]
pub mod frames;
#[cfg(not(target_arch = "wasm32"))]
pub mod native;
pub mod reader;
