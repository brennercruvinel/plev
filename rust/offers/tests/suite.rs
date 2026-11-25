//! Every integration test of `offers`, one module per file of this directory,
//! linked as one executable (task-106). As an executable each, the files
//! linked the crate and everything under it once per file.
//!
//! One file's tests: `cargo test -p offers --test suite adapt::`.

mod support;

mod adapt;
mod change;
#[path = "compose/main.rs"]
mod compose;
mod llm;
mod llm_change;
mod media;
mod rules;
