use crate::parse::block_item::{BindStmt, BlockItem, EachStmt, OnStmt, WhenStmt};
use proc_macro2::TokenStream;
use quote::quote;

use super::interpolation::show_to_tokens;
use super::{element_to_tokens, value_to_tokens};

pub(crate) fn block_item_to_tokens(item: &BlockItem) -> TokenStream {
    match item {
        BlockItem::Element(elem) => {
            let elem_tokens = element_to_tokens(elem);
            quote! { .child(#elem_tokens) }
        }
        BlockItem::Show(show) => show_to_tokens(show),
        BlockItem::On(on) => on_to_tokens(on),
        BlockItem::When(when) => when_to_tokens(when),
        BlockItem::Each(each) => each_to_tokens(each),
        BlockItem::Bind(bind) => bind_to_tokens(bind),
    }
}

// ── On ──

fn on_to_tokens(on: &OnStmt) -> TokenStream {
    let method = on.event.method_ident();
    let body = &on.body;
    match &on.params {
        Some(param) => quote! { .#method(move |#param| { #body }) },
        None => quote! { .#method(move |_| { #body }) },
    }
}

// ── When ──

fn when_to_tokens(when: &WhenStmt) -> TokenStream {
    let condition = &when.condition;
    let body = block_items_to_element(&when.body);

    match &when.otherwise {
        Some(else_items) => {
            let else_body = block_items_to_element(else_items);
            quote! {
                .child_if_else(
                    move || #condition,
                    || #body,
                    || #else_body
                )
            }
        }
        None => {
            quote! {
                .child_if(
                    move || #condition,
                    || #body
                )
            }
        }
    }
}

// ── Each ──

fn each_to_tokens(each: &EachStmt) -> TokenStream {
    let binding = &each.binding;
    let iterable = &each.iterable;
    let body = block_items_to_element(&each.body);

    if let Some(ref key) = each.key {
        quote! {
            .children_each_keyed(
                move || #iterable,
                |#binding| #key,
                |#binding| #body
            )
        }
    } else {
        quote! {
            .children_each(
                move || #iterable,
                |#binding| #body
            )
        }
    }
}

// ── Bind ──

fn bind_to_tokens(bind: &BindStmt) -> TokenStream {
    let target_str = bind.target.to_string();
    let val = value_to_tokens(&bind.value);
    quote! { .bind(#target_str, move || #val) }
}

// ── Helpers ──

/// Convert a list of block items into a single element expression.
///
/// If there's exactly one element child, returns it directly.
/// Otherwise wraps everything in a `div()` container.
pub(crate) fn block_items_to_element(items: &[BlockItem]) -> TokenStream {
    if items.len() == 1 {
        match &items[0] {
            BlockItem::Element(elem) => return element_to_tokens(elem),
            BlockItem::Show(show) => {
                let show_tokens = show_to_tokens(show);
                return quote! { text() #show_tokens };
            }
            _ => {}
        }
    }

    let calls: Vec<TokenStream> = items.iter().map(block_item_to_tokens).collect();
    quote! { div() #(#calls)* }
}
