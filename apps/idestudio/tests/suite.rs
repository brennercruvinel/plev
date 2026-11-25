//! Every integration test of `idestudio`, one module per file of this directory,
//! linked as one executable (task-106). As an executable each, the files
//! linked the crate and everything under it once per file.
//!
//! One file's tests: `cargo test -p idestudio --test suite agent_port::`.

mod support;

mod agent_port;
mod claude;
mod human_agent;
mod line;
mod native_confirm;
mod observe_cost;
mod plevir_fixture;
mod record;
mod review;
mod review_acts;
mod surface;
