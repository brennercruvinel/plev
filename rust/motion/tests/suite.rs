//! Every integration test of `motion`, one module per file of this directory,
//! linked as one executable (task-106). As an executable each, the files
//! linked the crate and everything under it once per file.
//!
//! One file's tests: `cargo test -p motion --test suite gate_discover::`.

mod gate_discover;
mod raise;
