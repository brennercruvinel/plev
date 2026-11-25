//! Every integration test of `agenteval`, one module per file of this directory,
//! linked as one executable (task-106). As an executable each, the files
//! linked the crate and everything under it once per file.
//!
//! One file's tests: `cargo test -p agenteval --test suite experiment::`.

mod experiment;
mod review;
