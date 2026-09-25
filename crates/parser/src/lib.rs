//! parser: a parse-resolve-emit transpiler poc. UI source from another
//! framework goes in, plev builder code comes out, and everything that does
//! not survive the trip is reported on a droplist, never dropped silently.
//!
//! **Experimental** -- a research poc, not a supported app path
//! (docs/ADR/official-app-pattern.md).
//!
//! Three stages, one module each per source language:
//! 1. parse  (tsx.rs, sass.rs, gpui.rs): tree-sitter -> raw trees/chains
//! 2. resolve (resolve_react.rs via css_map.rs, resolve_gpui.rs): normalize
//!    to the ParserNode IR, map colors to HOFF theme tokens, apply the
//!    documented flow rewrites, fill the droplist with file:line entries
//! 3. emit, two emitters over the same ParserNode:
//!    - emit.rs: deterministic rust source against `engine::builder`;
//!    - emit_ir/: plev IR (`uir::Ir`, task-71, ADR
//!      importers-target-plev-ir), measured against the theme's tokens
//!      (tokens.rs), with a report that puts every construct of the source
//!      in one of four categories (report.rs). The IR only references the
//!      app's capabilities; `uir::verify` decides whether it may exist.
//!
//! Scope is brutally small and honest (ADR transpiler-reports-every-
//! unmapped-construct): one react component instance (the hoff research
//! card, base variant) and one gpui widget instance (the horizontal labeled
//! separator). The emitted code obeys docs/how-to/code-against-the-plev-
//! engine.md: theme tokens where colors hit the HOFF palette, one TextStyle
//! per text run, content-driven layout (no absolute positioning).

pub mod css_map;
pub mod emit;
pub mod emit_ir;
pub mod gpui;
pub(crate) mod gpui_ir;
pub mod ir;
pub mod report;
pub mod resolve_gpui;
pub mod resolve_react;
pub mod sass;
pub mod tokens;
pub mod tsx;

pub use emit_ir::{Imported, emit_ir};
pub use ir::{Dropped, Resolution, Transpiled};
pub use report::{Binding, Category, Entry, Need, Report, Verdict};
pub use tokens::{Tokens, TypeStyle, Value};

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("parse error: {0}")]
    Parse(String),
    /// An id or a reference the IR emitter wrote that the `sem` grammar
    /// refuses: an importer defect, reported instead of a panic.
    #[error("plev IR: {0}")]
    Ir(String),
}

/// Resolve a react component: `(file_name, source)` pairs for the tsx
/// module and its sass module, plus the shared sass variables source.
pub fn resolve_react(
    tsx: (&str, &str),
    sass: (&str, &str),
    vars: &str,
) -> Result<Resolution, ParserError> {
    let component = tsx::parse_tsx(tsx.1)?;
    let rules = sass::parse_sass(sass.1, vars);
    Ok(resolve_react::resolve_react(
        &component, &rules, tsx.0, sass.0,
    ))
}

/// Resolve a gpui widget: `(file_name, source)`. The poc resolves the
/// instance dissected in the study: horizontal, solid, with label.
pub fn resolve_gpui(rust: (&str, &str)) -> Result<Resolution, ParserError> {
    let parsed = gpui::parse_gpui(rust.1)?;
    resolve_gpui::resolve_gpui(&parsed, rust.0)
}

fn transpiled(res: Resolution) -> Transpiled {
    Transpiled {
        code: emit::emit(&res),
        mapped: res.mapped,
        dropped: res.dropped,
    }
}

/// Transpile a react component to plev builder code (see `resolve_react`).
pub fn transpile_react(
    tsx: (&str, &str),
    sass: (&str, &str),
    vars: &str,
) -> Result<Transpiled, ParserError> {
    resolve_react(tsx, sass, vars).map(transpiled)
}

/// Transpile a gpui widget to plev builder code (see `resolve_gpui`).
pub fn transpile_gpui(rust: (&str, &str)) -> Result<Transpiled, ParserError> {
    resolve_gpui(rust).map(transpiled)
}

/// Import a react component as plev IR, measured against `tokens`, with
/// its report (see `resolve_react`).
pub fn import_react(
    tsx: (&str, &str),
    sass: (&str, &str),
    vars: &str,
    tokens: &Tokens,
) -> Result<Imported, ParserError> {
    emit_ir(&resolve_react(tsx, sass, vars)?, tokens)
}

/// Import a gpui widget as plev IR, measured against `tokens`, with its
/// report (see `resolve_gpui`).
pub fn import_gpui(rust: (&str, &str), tokens: &Tokens) -> Result<Imported, ParserError> {
    emit_ir(&resolve_gpui(rust)?, tokens)
}
