//! The monster v0 file: container layout, sections, quantization, asset
//! paths, validation, the writer and the reader.

pub mod asset_path;
pub mod container;
pub mod quantize;
pub mod read;
#[cfg(test)]
pub(super) mod tests;
pub(super) mod validate;
pub mod write;
