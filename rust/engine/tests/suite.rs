//! Every integration test of `engine`, one module per file of this directory,
//! linked as one executable (task-106). As an executable each, the files
//! linked the crate and everything under it once per file.
//!
//! One file's tests: `cargo test -p engine --test suite alerts::`.
//!
//! `device_loss.rs` and `text_audit.rs` are targets of their own (see the
//! manifest): each installs the process's logger.

mod alerts;
mod asset_cost;
mod blur_scale;
mod capture;
mod editor;
mod headless_determinism;
mod links;
mod menu;
mod notify;
mod print;
mod sleep_wake;
mod text_drawn_extent;
mod text_raster_pixels;
mod tray;
mod wgsl;
mod windows;
