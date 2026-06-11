//! prs: a parse-resolve-emit transpiler poc. UI source from another
//! framework goes in, plev builder code comes out, and everything that does
//! not survive the trip is reported on a droplist, never dropped silently.
//!
//! Three stages, one module each per source language:
//! 1. parse  (tsx.rs, sass.rs, gpui.rs): tree-sitter -> raw trees/chains
//! 2. resolve (resolve_react.rs via css_map.rs, resolve_gpui.rs): normalize
//!    to the PrsNode IR, map colors to HOFF theme tokens, apply the
//!    documented flow rewrites, fill the droplist with file:line entries
//! 3. emit   (emit.rs): deterministic rust source against `plev::builder`
//!
//! Scope is brutally small and honest (kdb/brain-fable-e-bre/
//! prs-transpiler.lua): one react component instance (the hoff research
//! card, base variant) and one gpui widget instance (the horizontal labeled
//! separator). The emitted code obeys kdb/how-to/code-against-the-plev-
//! engine.md: theme tokens where colors hit the HOFF palette, one TextStyle
//! per text run, content-driven layout (no absolute positioning).

pub mod css_map;
pub mod emit;
pub mod gpui;
pub(crate) mod gpui_ir;
pub mod ir;
pub mod resolve_gpui;
pub mod resolve_react;
pub mod sass;
pub mod tsx;

pub use ir::{Dropped, Transpiled};

#[derive(Debug, thiserror::Error)]
pub enum PrsError {
    #[error("parse error: {0}")]
    Parse(String),
}

/// Transpile a react component: `(file_name, source)` pairs for the tsx
/// module and its sass module, plus the shared sass variables source.
pub fn transpile_react(
    tsx: (&str, &str),
    sass: (&str, &str),
    vars: &str,
) -> Result<Transpiled, PrsError> {
    let component = tsx::parse_tsx(tsx.1)?;
    let rules = sass::parse_sass(sass.1, vars);
    let res = resolve_react::resolve_react(&component, &rules, tsx.0, sass.0);
    Ok(Transpiled {
        code: emit::emit(&res),
        mapped: res.mapped,
        dropped: res.dropped,
    })
}

/// Transpile a gpui widget: `(file_name, source)`. The poc transpiles the
/// instance dissected in the study: horizontal, solid, with label.
pub fn transpile_gpui(rust: (&str, &str)) -> Result<Transpiled, PrsError> {
    let parsed = gpui::parse_gpui(rust.1)?;
    let res = resolve_gpui::resolve_gpui(&parsed, rust.0)?;
    Ok(Transpiled {
        code: emit::emit(&res),
        mapped: res.mapped,
        dropped: res.dropped,
    })
}
