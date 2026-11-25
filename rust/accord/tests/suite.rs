//! Every integration test of `accord`, one module per file of this directory,
//! linked as one executable (task-106). As an executable each, the files
//! linked the crate and everything under it once per file.
//!
//! One file's tests: `cargo test -p accord --test suite assembly::`.

mod assembly;
mod decision;
mod dispatch;
mod encoding;
mod identity;
mod mutation;
mod property;
mod receipts;
mod requests;
mod snapshot;
