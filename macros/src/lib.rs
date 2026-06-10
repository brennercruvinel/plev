use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Transforms a function into a View-implementing struct.
///
/// ```ignore
/// #[component]
/// fn Header(cx: Scope) -> impl IntoView {
///     div().bg("blue").child(text("Title"))
/// }
/// ```
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_body = &input.block;
    let vis = &input.vis;

    let expanded = quote! {
        #vis struct #fn_name;

        impl crate::view::View for #fn_name {
            fn render(&self, cx: &mut crate::view::ViewContext) -> Vec<crate::compositor::SceneNode> {
                let scope = crate::builder::Scope::from(cx);
                let element: crate::builder::Element = {
                    use crate::builder::*;
                    use crate::color::*;
                    #fn_body
                }.into_view();
                element.render(&mut crate::view::ViewContext::new(
                    scope.cx.width,
                    scope.cx.height,
                ))
            }
        }
    };

    TokenStream::from(expanded)
}
