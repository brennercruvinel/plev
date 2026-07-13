mod generators;
mod interpolation;

#[cfg(test)]
mod tests;

use crate::parse::NarrateBlock;
use crate::parse::element::NarrateElement;
use crate::parse::keywords::ElementKind;
use crate::parse::modifier::Modifier;
use crate::parse::value::ModifierValue;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(block: NarrateBlock) -> TokenStream {
    if block.elements.is_empty() {
        return quote! { compile_error!("plev_narrate! requires at least one root element") };
    }

    if block.elements.len() > 1 {
        return quote! {
            compile_error!("plev_narrate! requires exactly one root element; \
                           wrap multiple elements in a container like `col { ... }`")
        };
    }

    let elem = element_to_tokens(&block.elements[0]);

    quote! {
        {
            #[allow(unused_imports)]
            use ::narrate::builder::*;
            ::engine::narrate_resolve(file!(), line!(), || {
                #elem
            })
        }
    }
}

pub(crate) fn element_to_tokens(elem: &NarrateElement) -> TokenStream {
    let constructor = element_constructor(&elem.kind);
    let modifier_calls: Vec<TokenStream> = elem.modifiers.iter().map(modifier_to_tokens).collect();
    let body_calls: Vec<TokenStream> = elem
        .body
        .as_ref()
        .map(|items| items.iter().map(generators::block_item_to_tokens).collect())
        .unwrap_or_default();

    quote! { #constructor #(#modifier_calls)* #(#body_calls)* }
}

fn element_constructor(kind: &ElementKind) -> TokenStream {
    match kind {
        ElementKind::Row => quote! { div().flex().row() },
        ElementKind::Col => quote! { div().flex().col() },
        ElementKind::Div => quote! { div() },
        ElementKind::Text => quote! { text() },
        ElementKind::Button => quote! { button() },
        ElementKind::Image => quote! { image() },
        ElementKind::Spacer => quote! { spacer() },
        ElementKind::Custom(name) => quote! { #name::view() },
    }
}

fn modifier_to_tokens(m: &Modifier) -> TokenStream {
    let method = m.key.method_ident();
    match &m.value {
        Some(v) => {
            let val = value_to_tokens(v);
            quote! { .#method(#val) }
        }
        None => quote! { .#method() },
    }
}

pub(crate) fn value_to_tokens(v: &ModifierValue) -> TokenStream {
    match v {
        ModifierValue::Str(lit) => quote! { #lit },
        ModifierValue::Int(lit) => quote! { #lit },
        ModifierValue::Float(lit) => quote! { #lit },
        ModifierValue::Expr(expr) => quote! { #expr },
    }
}
