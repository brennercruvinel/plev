//! Every integration test of `gitdrv`, one module per file of this directory,
//! linked as one executable (task-106). As an executable each, the files
//! linked the crate and everything under it once per file.
//!
//! One file's tests: `cargo test -p gitdrv --test suite log_cost::`.

mod log_cost;
mod real_repo;
