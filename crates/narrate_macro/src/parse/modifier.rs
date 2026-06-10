use proc_macro2::Span;
use syn::parse::ParseStream;
use syn::{Ident, Token};

use super::keywords::{MODIFIER_NAMES, ModifierKey, is_element_or_block_keyword, suggest_similar};
use super::value::{ModifierValue, peek_value};

#[derive(Debug, Clone)]
pub struct Modifier {
    pub key: ModifierKey,
    pub value: Option<ModifierValue>,
    pub span: Span,
}

/// Parse a sequence of modifiers from the input stream.
///
/// Modifiers are consumed until the next token is not a known modifier key
/// (i.e., it's an element keyword, block-item keyword, `{`, or end of input).
/// Commas between modifiers are optional.
///
/// If an unknown identifier appears in modifier position and looks like a typo
/// of a known modifier, a "did you mean?" error is produced instead of silently
/// stopping modifier parsing (which would later produce a confusing "unknown
/// element" error).
pub fn parse_modifiers(input: ParseStream) -> syn::Result<Vec<Modifier>> {
    let mut modifiers = Vec::new();

    loop {
        // Skip optional comma between modifiers
        if !modifiers.is_empty() && input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        // Check if next token is an ident that could be a modifier key
        if !input.peek(Ident) {
            break;
        }

        // Fork to inspect without consuming
        let fork = input.fork();
        let ident: Ident = fork.parse()?;
        let ident_str = ident.to_string();

        let Some(key) = ModifierKey::from_str(&ident_str) else {
            // Not a modifier key. Before giving up, check if it's a typo of a
            // known modifier. Only do this if it's NOT a known element/block
            // keyword (those should fall through to be parsed as elements).
            if !is_element_or_block_keyword(&ident_str)
                && let Some(suggestion) = suggest_similar(&ident_str, MODIFIER_NAMES)
            {
                // Consume the ident so the error span points at the right token
                let bad_ident: Ident = input.parse()?;
                return Err(syn::Error::new(
                    bad_ident.span(),
                    format!(
                        "unknown modifier `{}`. Did you mean `{}`?",
                        ident_str, suggestion,
                    ),
                ));
            }
            break;
        };

        // Consume the ident for real
        let ident: Ident = input.parse()?;
        let span = ident.span();

        let value = if key.is_flag() {
            // Flags never take a value
            None
        } else if peek_value(input) {
            // Value-required modifier: parse the value
            Some(input.parse::<ModifierValue>()?)
        } else {
            // Value-required but no value found — give a helpful example
            let example = match key {
                ModifierKey::Bg
                | ModifierKey::TextColor
                | ModifierKey::Rounded
                | ModifierKey::Shadow
                | ModifierKey::Border => {
                    format!(
                        "modifier `{}` requires a value, e.g. `{} \"value\"`",
                        ident_str, ident_str,
                    )
                }
                ModifierKey::Opacity => format!(
                    "modifier `{}` requires a value, e.g. `{} 0.5`",
                    ident_str, ident_str,
                ),
                _ => format!(
                    "modifier `{}` requires a value, e.g. `{} 4` or `{} {{expr}}`",
                    ident_str, ident_str, ident_str,
                ),
            };
            return Err(syn::Error::new(span, example));
        };

        modifiers.push(Modifier { key, value, span });
    }

    Ok(modifiers)
}
