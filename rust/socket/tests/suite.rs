//! Every integration test of `socket`, one module per file of this directory,
//! linked as one executable (task-106). As an executable each, the files
//! linked the crate and everything under it once per file.
//!
//! One file's tests: `cargo test -p socket --test suite cli::`.

mod support;

mod cli;
mod codec;
mod env;
mod look;
mod ring;
mod scene;
mod server;
mod shown;
mod socket;
mod traffic;
