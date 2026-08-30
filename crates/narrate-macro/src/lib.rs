//! Proc-macro backend for the experimental narrate UI dsl. **Not the
//! official app pattern** (docs/adr/official-app-pattern.md) -- do not
//! build new app code on this crate without an ADR.

mod codegen;
mod parse;

use proc_macro::TokenStream;

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
    let block = syn::parse_macro_input!(input as parse::NarrateBlock);
    codegen::generate(block).into()
}
