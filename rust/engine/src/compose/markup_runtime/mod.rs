//! Runtime interpreter for plev_narrate! DSL.
//!
//! Parses narrate DSL text at runtime and produces `Element` trees
//! using the builder API. Used by the hot-reload system to update
//! UI without recompilation.
//!
//! Limitations (by design):
//! - `on`, `when`, `each`, `bind` blocks are SKIPPED (require Rust evaluation)
//! - Only static `show "text"` is interpreted (expression values skipped)
//! - Custom PascalCase components render as empty div placeholders

#[cfg(target_arch = "wasm32")]
compile_error!("narrate_runtime is not supported on WASM");

mod extraction;
mod keywords;
mod modifiers;
mod parser;
mod tokenizer;

#[cfg(test)]
mod tests;

use crate::builder::{self, Element};

// Re-export public API items.
pub use extraction::extract_narrate_blocks;

// Re-export crate-internal items used by tests.
#[cfg(test)]
pub(crate) use tokenizer::{Token, tokenize};

/// Parse a narrate DSL string into an Element tree.
///
/// Returns `None` if the input is empty or the root element is unrecognized.
pub fn parse_narrate(input: &str) -> Option<Element> {
    let mut parser = parser::Parser::new(input);
    let elements = parser.parse_top_level();

    match elements.len() {
        0 => None,
        1 => Some(elements.into_iter().next().unwrap()),
        _ => {
            let mut root = builder::div();
            for el in elements {
                root = root.child(el);
            }
            Some(root)
        }
    }
}
