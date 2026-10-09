use crate::parse::block_item::ShowStmt;
use crate::parse::value::ModifierValue;
use proc_macro2::TokenStream;
use quote::quote;

use super::value_to_tokens;

pub(crate) fn show_to_tokens(show: &ShowStmt) -> TokenStream {
    match &show.value {
        ModifierValue::Str(lit) => {
            let s = lit.value();
            let (fmt, exprs) = parse_interpolation(&s);
            if exprs.is_empty() {
                quote! { .child(#lit) }
            } else {
                let expr_tokens: Vec<TokenStream> = exprs
                    .iter()
                    .map(|e| match syn::parse_str::<syn::Expr>(e) {
                        Ok(expr) => quote! { #expr },
                        Err(err) => {
                            let msg = format!("invalid expression in string interpolation: {err}");
                            quote! { compile_error!(#msg) }
                        }
                    })
                    .collect();
                let fmt_lit = syn::LitStr::new(&fmt, lit.span());
                quote! { .child(format!(#fmt_lit, #(#expr_tokens),*)) }
            }
        }
        other => {
            let val = value_to_tokens(other);
            quote! { .child(#val) }
        }
    }
}

/// Parse `{expr}` interpolations within a string.
///
/// Returns `(format_string, expressions)` where `format_string` has `{expr}`
/// replaced with `{}` and `expressions` contains the extracted expression strings.
///
/// Supports:
/// - `{ident}` -> single ident
/// - `{complex.expr()}` -> nested braces tracked by depth
/// - `{{` -> escaped literal `{`
/// - `}}` -> escaped literal `}`
pub(crate) fn parse_interpolation(s: &str) -> (String, Vec<String>) {
    let mut format_string = String::new();
    let mut expressions = Vec::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '{' => {
                if chars.peek() == Some(&'{') {
                    format_string.push_str("{{");
                    chars.next();
                } else {
                    let mut expr = String::new();
                    let mut depth = 1;
                    for c in chars.by_ref() {
                        match c {
                            '{' => {
                                depth += 1;
                                expr.push(c);
                            }
                            '}' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                                expr.push(c);
                            }
                            _ => expr.push(c),
                        }
                    }
                    format_string.push_str("{}");
                    expressions.push(expr);
                }
            }
            '}' => {
                if chars.peek() == Some(&'}') {
                    format_string.push_str("}}");
                    chars.next();
                } else {
                    format_string.push('}');
                }
            }
            _ => format_string.push(c),
        }
    }

    (format_string, expressions)
}
