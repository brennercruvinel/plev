//! Hot reload for development (desktop debug builds only, see build.rs).
//!
//! - Tier 1 (shader): `GpuContext` owns a watcher on `src/gpu/shaders/`; a
//!   saved `.wgsl` recreates its pipeline on the next frame and wakes the
//!   event loop so that frame happens now. Every app gets it through
//!   `present_frame`, nothing to wire.
//! - Literals: a saved `.rs` whose only change is in strings or hex colors
//!   is applied to the open window, state intact (`literals`). A code
//!   change rebuilds the bin and re-execs it (`source`), so
//!   `cargo run -p showcase` alone is the dev loop.
//! - Tier 2 (narrate DSL): file watcher + runtime parse + override map,
//!   polled by the engine's own `window::App` (experimental, no app uses it).
//! - View source (`view_source`): a point in the window leads to the code
//!   that made it, a control to its projection site and a text to the
//!   literals that hold it; the shell opens the editor there.

pub mod literals;
mod shader;
mod source;
#[cfg(test)]
mod tests;
pub mod view_source;
mod markup;

use std::borrow::Cow;
use std::path::PathBuf;

// Re-export all public items
pub use markup::{
    NarrateWatcher, narrate_override, process_narrate_file, project_root, update_narrate_overrides,
};
pub use shader::ShaderWatcher;
pub use source::{SourceWatcher, source_dirs};

/// A place in the workspace sources: the file (absolute), and the line and
/// column from 1 of its first character, the column counted in chars. It
/// prints as `file:line:column`, the form editors open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Site {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
}

impl std::fmt::Display for Site {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file.display(), self.line, self.column)
    }
}

/// Directory containing `.wgsl` shader source files.
pub fn shader_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/drawing/gpu/shaders")
}

/// Read a shader source file from disk (hot-reload path).
///
/// Falls back to the compile-time embedded copy if the file cannot be read.
pub fn shader_source(filename: &str) -> Cow<'static, str> {
    let path = shader_dir().join(filename);
    match std::fs::read_to_string(&path) {
        Ok(source) => Cow::Owned(source),
        Err(e) => {
            log::error!(
                "Failed to read shader {}: {} -- using embedded fallback",
                path.display(),
                e
            );
            Cow::Borrowed(fallback_shader(filename))
        }
    }
}

pub(crate) fn fallback_shader(filename: &str) -> &'static str {
    match filename {
        "quad.wgsl" => include_str!("../../drawing/gpu/shaders/quad.wgsl"),
        "text.wgsl" => include_str!("../../drawing/gpu/shaders/text.wgsl"),
        "rect_sdf.wgsl" => include_str!("../../drawing/gpu/shaders/rect_sdf.wgsl"),
        "shadow_analytic.wgsl" => include_str!("../../drawing/gpu/shaders/shadow_analytic.wgsl"),
        "image.wgsl" => include_str!("../../drawing/gpu/shaders/image.wgsl"),
        "backdrop.wgsl" => include_str!("../../drawing/gpu/shaders/backdrop.wgsl"),
        "composite.wgsl" => include_str!("../../drawing/gpu/shaders/composite.wgsl"),
        "blur.wgsl" => include_str!("../../drawing/gpu/shaders/blur.wgsl"),
        "shadow.wgsl" => include_str!("../../drawing/gpu/shaders/shadow.wgsl"),
        _ => {
            log::error!("Unknown shader for fallback: {filename}");
            ""
        }
    }
}
