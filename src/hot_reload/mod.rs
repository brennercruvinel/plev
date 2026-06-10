//! Hot reload for development.
//!
//! Watches `shaders/*.wgsl` for shader changes and `src/`/`examples/` for
//! plev_narrate! DSL changes. Feature-gated behind `hot-reload`.
//!
//! - Tier 1 (Shader): file watcher + pipeline recreation
//! - Tier 2 (DSL): file watcher + runtime parse + override map

#[cfg(target_arch = "wasm32")]
compile_error!("hot-reload feature is not supported on WASM (no filesystem, no threads)");

mod narrate;
mod shader;
#[cfg(test)]
mod tests;

use std::borrow::Cow;
use std::path::PathBuf;

// Re-export all public items
pub use narrate::{
    NarrateWatcher, narrate_override, process_narrate_file, project_root, update_narrate_overrides,
};
pub use shader::ShaderWatcher;

/// Directory containing `.wgsl` shader source files.
pub fn shader_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shaders")
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
        "quad.wgsl" => include_str!("../../shaders/quad.wgsl"),
        "text.wgsl" => include_str!("../../shaders/text.wgsl"),
        "rect_sdf.wgsl" => include_str!("../../shaders/rect_sdf.wgsl"),
        "composite.wgsl" => include_str!("../../shaders/composite.wgsl"),
        "blur.wgsl" => include_str!("../../shaders/blur.wgsl"),
        "shadow.wgsl" => include_str!("../../shaders/shadow.wgsl"),
        _ => {
            log::error!("Unknown shader for fallback: {}", filename);
            ""
        }
    }
}
