//! `#[component]`: a function becomes a View-implementing struct.

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// The expansion of `#[component]` (see the crate root).
pub(crate) fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_body = &input.block;
    let vis = &input.vis;

    let expanded = quote! {
        #vis struct #fn_name;

        impl crate::compose::view::View for #fn_name {
            fn render(&self, cx: &mut crate::compose::view::ViewContext) -> Vec<crate::drawing::compositor::SceneNode> {
                let scope = crate::compose::builder::Scope::from(cx);
                let element: crate::compose::builder::Element = {
                    use crate::compose::builder::*;
                    use crate::styling::color::*;
                    #fn_body
                }.into_view();
                element.render(&mut crate::compose::view::ViewContext::new(
                    scope.cx.width,
                    scope.cx.height,
                ))
            }
        }
    };

    TokenStream::from(expanded)
}
