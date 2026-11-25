//! Every integration test of `uigallery`, one module per file of this directory,
//! linked as one executable (task-106). As an executable each, the files
//! linked the crate and everything under it once per file.
//!
//! One file's tests: `cargo test -p uigallery --test suite access_tree::`.
//!
//! `edit_literal.rs` is a target of its own (see the manifest): it rewrites
//! the process's literal baseline, which `view_source` reads.

mod support;

mod access_tree;
mod capture;
#[path = "component_pages/main.rs"]
mod component_pages;
mod composites;
mod demo;
mod detached;
mod ephemeral;
mod hit_targets;
mod links;
mod mirror;
mod mirror_input;
mod mirror_mask;
mod mirror_probe;
mod plevir_fixture;
mod print;
mod record;
mod record_cost;
mod reduced_motion;
mod scene_codec;
mod scene_probe;
mod sleep_wake;
mod text_scale;
mod view_source;
