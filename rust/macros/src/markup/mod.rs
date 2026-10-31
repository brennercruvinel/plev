//! Backend of the experimental markup UI dsl (`plev_narrate!`). **Not the
//! official app pattern** (docs/decisoes/estrutura/0023-official-app-pattern.md) -- do not
//! build new app code on this crate without an ADR.

mod codegen;
mod parse;

use proc_macro::TokenStream;

/// The expansion of `plev_narrate!` (see the crate root).
pub(crate) fn plev_narrate(input: TokenStream) -> TokenStream {
    let block = syn::parse_macro_input!(input as parse::NarrateBlock);
    codegen::generate(block).into()
}
