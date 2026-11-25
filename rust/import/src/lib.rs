//! parser: a parse-resolve-emit transpiler poc. UI source from another
//! framework goes in, plev builder code comes out, and everything that does
//! not survive the trip is reported on a droplist, never dropped silently.
//!
//! **Experimental** -- a research poc, not a supported app path
//! (docs/adr/Structure/0023-official-app-pattern/0023-official-app-pattern.md).
//!
//! Three stages, one module each per source language:
//! 1. parse  (tsx.rs, sass.rs, gpui.rs, html.rs): tree-sitter -> raw
//!    trees/chains; figma.rs: a Figma export's JSON -> its layers
//! 2. resolve (resolve_react.rs via css_map.rs, resolve_gpui.rs,
//!    resolve_html/ via html_map.rs and css_map.rs, resolve_figma/):
//!    normalize
//!    to the ParserNode IR, map colors to HOFF theme tokens, apply the
//!    documented flow rewrites, fill the droplist with file:line entries
//! 3. emit, two emitters over the same ParserNode:
//!    - emit.rs: deterministic rust source against `engine::compose::builder`;
//!    - emit_ir/: plev IR (`plevir::Ir`, task-71, ADR
//!      importers-target-plev-ir), measured against the theme's tokens
//!      (tokens.rs), with a report that puts every construct of the source
//!      in one of four categories (report.rs). The IR only references the
//!      app's capabilities; `plevir::verify` decides whether it may exist.
//!
//! Scope is brutally small and honest (ADR transpiler-reports-every-
//! unmapped-construct): one react component instance (the hoff research
//! card, base variant) and one gpui widget instance (the horizontal labeled
//! separator); task-71 phase 2 adds an HTML/CSS subset (one document, the
//! body's element) and a Figma export (one screen). The emitted code obeys
//! docs/Research/Workflows/code-against-the-plev-engine.md: theme tokens where colors hit the HOFF palette, one TextStyle
//! per text run, content-driven layout (no absolute positioning).

pub mod emit;
pub mod formats;
pub mod report;
pub mod tokens;
pub mod tree;

pub use emit::plevir::{Imported, emit_ir};
pub use report::{Binding, Category, Entry, Need, Report, Verdict};
pub use tokens::{Tokens, TypeStyle, Value};
pub use tree::{Dropped, Resolution, Transpiled};

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
    let component = formats::react::parse::parse_tsx(tsx.1)?;
    let rules = formats::react::sass::parse_sass(sass.1, vars);
    Ok(formats::react::resolve::resolve_react(
        &component, &rules, tsx.0, sass.0,
    ))
}

/// Resolve a gpui widget: `(file_name, source)`. The poc resolves the
/// instance dissected in the study: horizontal, solid, with label.
pub fn resolve_gpui(rust: (&str, &str)) -> Result<Resolution, ParserError> {
    let parsed = formats::gpui::parse::parse_gpui(rust.1)?;
    formats::gpui::resolve::resolve_gpui(&parsed, rust.0)
}

/// Resolve an HTML document (task-71 phase 2): `(file_name, source)`. The
/// component is the body's element; the CSS is its `<style>` and `style=""`.
pub fn resolve_html(html: (&str, &str)) -> Result<Resolution, ParserError> {
    let doc = formats::html::parse::parse_html(html.1)?;
    formats::html::resolve::resolve_html(&doc, html.0)
}

/// Resolve a Figma export (task-71 phase 2): `(file_name, json)`, the REST
/// API's shape as `tool/tasks/figmapull.py` writes it. The screen is the first
/// layer under the document's page and section.
pub fn resolve_figma(json: (&str, &str)) -> Result<Resolution, ParserError> {
    let doc = formats::figma::parse::parse_figma(json.1)?;
    formats::figma::resolve::resolve_figma(&doc, json.0)
}

fn transpiled(res: Resolution) -> Transpiled {
    Transpiled {
        code: emit::builder::emit(&res),
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

/// Transpile an HTML document to plev builder code (see `resolve_html`).
pub fn transpile_html(html: (&str, &str)) -> Result<Transpiled, ParserError> {
    resolve_html(html).map(transpiled)
}

/// Import an HTML document as plev IR, measured against `tokens`, with its
/// report (see `resolve_html`).
pub fn import_html(html: (&str, &str), tokens: &Tokens) -> Result<Imported, ParserError> {
    emit_ir(&resolve_html(html)?, tokens)
}

/// Transpile a Figma export to plev builder code (see `resolve_figma`).
pub fn transpile_figma(json: (&str, &str)) -> Result<Transpiled, ParserError> {
    resolve_figma(json).map(transpiled)
}

/// Import a Figma export as plev IR, measured against `tokens`, with its
/// report (see `resolve_figma`).
pub fn import_figma(json: (&str, &str), tokens: &Tokens) -> Result<Imported, ParserError> {
    emit_ir(&resolve_figma(json)?, tokens)
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
