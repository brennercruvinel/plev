//! Unit test suites, one file per module under test. They live inside
//! src (not tests/) because unit tests need access to crate-private
//! items; this directory keeps them out of the production tree.

mod asset_path;
mod golden;
mod read;
mod read_malformed;
mod write;
mod operations;
mod properties;
mod quantize;
