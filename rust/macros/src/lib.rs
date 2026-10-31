//! Proc-macros for plev: the `#[component]` attribute (`view`) and the
//! backend of the experimental markup UI dsl (`markup`). **The dsl is not
//! the official app pattern** (docs/decisoes/estrutura/0023-official-app-pattern.md) -- do not
//! build new app code on it without an ADR.
//!
//! A proc-macro crate exports its macros from the root only; the bodies
//! live in the modules.

mod markup;
mod view;

use proc_macro::TokenStream;

/// Transforms a function into a View-implementing struct.
///
/// ```ignore
/// #[component]
/// fn Header(cx: Scope) -> impl IntoView {
///     div().bg("blue").child(text("Title"))
/// }
/// ```
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    view::component(attr, item)
}

/// Macro for describing UI in verbal, prose-like syntax.
///
/// # Example
/// ```ignore
/// plev_narrate! {
///     col centered, gap 4, p 8, bg "slate-900" {
///         text font_size 24, text_color "white" {
///             show "Hello, plev!"
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn plev_narrate(input: TokenStream) -> TokenStream {
    markup::plev_narrate(input)
}
