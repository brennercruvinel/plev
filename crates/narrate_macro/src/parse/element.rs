use proc_macro2::Span;
use syn::Ident;
use syn::parse::{Parse, ParseStream};

use super::block_item::{BlockItem, parse_block_items};
use super::keywords::{ELEMENT_NAMES, ElementKind, MODIFIER_NAMES, suggest_similar};
use super::modifier::{Modifier, parse_modifiers};

#[derive(Debug, Clone)]
pub struct NarrateElement {
    pub kind: ElementKind,
    pub modifiers: Vec<Modifier>,
    pub body: Option<Vec<BlockItem>>,
    pub span: Span,
}

impl Parse for NarrateElement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let span = ident.span();

        let kind = ElementKind::from_ident(&ident).ok_or_else(|| {
            let name = ident.to_string();

            // Check if it's a typo for a known element
            if let Some(suggestion) = suggest_similar(&name, ELEMENT_NAMES) {
                return syn::Error::new(
                    span,
                    format!("unknown element `{}`. Did you mean `{}`?", name, suggestion,),
                );
            }

            // Check if the user accidentally used a modifier as an element
            if let Some(suggestion) = suggest_similar(&name, MODIFIER_NAMES) {
                return syn::Error::new(
                    span,
                    format!(
                        "unknown element `{}`. `{}` is a modifier, not an element. \
                         Modifiers go after the element name, e.g. `div {} \"value\"`",
                        name, suggestion, suggestion,
                    ),
                );
            }

            syn::Error::new(
                span,
                format!(
                    "unknown element `{}`. Expected one of: row, col, div, text, button, \
                     image, spacer, or a PascalCase component name like `MyWidget`",
                    name,
                ),
            )
        })?;

        let modifiers = parse_modifiers(input)?;

        // Consume optional trailing comma before block
        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
        }

        let body = if input.peek(syn::token::Brace) {
            let content;
            syn::braced!(content in input);
            Some(parse_block_items(&content)?)
        } else {
            None
        };

        Ok(NarrateElement {
            kind,
            modifiers,
            body,
            span,
        })
    }
}
