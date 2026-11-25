//! Every integration test of `screen`, one module per file of this directory,
//! linked as one executable (task-106). As an executable each, the files
//! linked the crate and everything under it once per file.
//!
//! One file's tests: `cargo test -p screen --test suite adapt::`.

mod support;

mod adapt;
mod checkpoint;
mod code;
mod emphasis;
mod events;
mod frames;
mod materialize;
mod measure;
mod policy;
mod prefer;
mod promote;
mod render;
mod transform;
mod transition;
