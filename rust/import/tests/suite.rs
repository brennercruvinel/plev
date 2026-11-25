//! Every integration test of `import`, one module per file of this directory,
//! linked as one executable (task-106). As an executable each, the files
//! linked the crate and everything under it once per file.
//!
//! One file's tests: `cargo test -p import --test suite emitted_compiles::`.

mod support;

mod emitted_compiles;
mod figma;
mod golden;
mod html;
mod pixels;
mod plevir;
