//! Unit test suites, one file per module under test. They live inside
//! src (not tests/) because unit tests need access to crate-private
//! items; this directory keeps them out of the production tree.

mod asset_path;
mod discover;
mod discover_ops;
mod easing;
mod golden;
mod ir;
mod lower;
mod ops;
mod optimize;
mod optimize_pipe;
mod play;
mod play_ops;
mod prop;
mod quant;
mod read;
mod read_malformed;
mod write;
