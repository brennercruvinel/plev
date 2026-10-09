//! Unit test suites, one file per module under test. They live inside
//! src (not tests/) because unit tests need access to crate-private
//! items; this directory keeps them out of the production tree.

mod asset_path;
mod golden;
pub(crate) mod operations;
mod properties;
mod quantize;
mod read;
mod read_malformed;
pub(crate) mod write;
