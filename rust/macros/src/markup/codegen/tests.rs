use super::{generate, interpolation::parse_interpolation};
use crate::parse::NarrateBlock;
use quote::quote;

fn codegen_str(tokens: proc_macro2::TokenStream) -> String {
    let block: NarrateBlock = syn::parse2(tokens).expect("parse failed");
    generate(block).to_string()
}

#[test]
fn simple_div() {
    let out = codegen_str(quote! { div });
    assert!(out.contains("div ()"));
}

#[test]
fn row_generates_flex_row() {
    let out = codegen_str(quote! { row });
    assert!(out.contains("div ()"));
    assert!(out.contains(". flex ()"));
    assert!(out.contains(". row ()"));
}

#[test]
fn col_generates_flex_col() {
    let out = codegen_str(quote! { col });
    assert!(out.contains(". flex ()"));
    assert!(out.contains(". col ()"));
}

#[test]
fn div_with_bg() {
    let out = codegen_str(quote! { div bg "blue" });
    assert!(out.contains(". bg (\"blue\")"));
}

#[test]
fn text_with_show() {
    let out = codegen_str(quote! { text { show "Hello" } });
    assert!(out.contains("text ()"));
    assert!(out.contains(". child (\"Hello\")"));
}

#[test]
fn show_format_interpolation() {
    let out = codegen_str(quote! { text { show "Count: {count}" } });
    assert!(out.contains("format !"));
    assert!(out.contains("count"));
}

#[test]
fn show_no_interpolation() {
    let out = codegen_str(quote! { text { show "Hello World" } });
    assert!(!out.contains("format"));
    assert!(out.contains(". child (\"Hello World\")"));
}

#[test]
fn show_escaped_braces() {
    let out = codegen_str(quote! { text { show "use {{braces}}" } });
    // No interpolation for escaped braces
    assert!(!out.contains("format"));
}

#[test]
fn on_click_codegen() {
    let out = codegen_str(quote! { button { on click { do_thing() } } });
    assert!(out.contains(". on_click"));
    assert!(out.contains("move | _ |"));
    assert!(out.contains("do_thing ()"));
}

#[test]
fn on_click_with_param() {
    let out = codegen_str(quote! { button { on click |e| { handle(e) } } });
    assert!(out.contains("move | e |"));
}

#[test]
fn when_codegen() {
    let out = codegen_str(quote! {
        div {
            when { flag } {
                text { show "Yes" }
            }
        }
    });
    assert!(out.contains(". child_if"));
    assert!(out.contains("flag"));
}

#[test]
fn when_otherwise_codegen() {
    let out = codegen_str(quote! {
        div {
            when { flag } {
                text { show "Yes" }
            } otherwise {
                text { show "No" }
            }
        }
    });
    assert!(out.contains(". child_if_else"));
}

#[test]
fn each_codegen() {
    let out = codegen_str(quote! {
        div {
            each item in { items } {
                text { show "x" }
            }
        }
    });
    assert!(out.contains(". children_each"));
    assert!(out.contains("| item |"));
}

#[test]
fn each_keyed_codegen() {
    let out = codegen_str(quote! {
        div {
            each item in { items } keyed by { item.id } {
                text { show "x" }
            }
        }
    });
    assert!(out.contains(". children_each_keyed"));
}

#[test]
fn bind_codegen() {
    let out = codegen_str(quote! {
        div {
            bind value to { signal }
        }
    });
    assert!(out.contains(". bind (\"value\""));
}

#[test]
fn nested_child() {
    let out = codegen_str(quote! {
        div bg "blue" {
            text { show "Hello" }
        }
    });
    assert!(out.contains(". child (text ()"));
}

#[test]
fn flag_modifier_codegen() {
    let out = codegen_str(quote! { div bold });
    assert!(out.contains(". bold ()"));
}

#[test]
fn multiple_roots_error() {
    let block: NarrateBlock = syn::parse2(quote! { div {} div {} }).unwrap();
    let out = generate(block).to_string();
    assert!(out.contains("compile_error"));
}

#[test]
fn custom_component_codegen() {
    let out = codegen_str(quote! { MyWidget });
    assert!(out.contains("MyWidget :: view ()"));
}

// ── Interpolation unit tests ──

#[test]
fn interpolation_none() {
    let (fmt, exprs) = parse_interpolation("hello world");
    assert_eq!(fmt, "hello world");
    assert!(exprs.is_empty());
}

#[test]
fn interpolation_single() {
    let (fmt, exprs) = parse_interpolation("Count: {count}");
    assert_eq!(fmt, "Count: {}");
    assert_eq!(exprs, vec!["count"]);
}

#[test]
fn interpolation_multiple() {
    let (fmt, exprs) = parse_interpolation("{a} and {b}");
    assert_eq!(fmt, "{} and {}");
    assert_eq!(exprs, vec!["a", "b"]);
}

#[test]
fn interpolation_complex_expr() {
    let (fmt, exprs) = parse_interpolation("val: {items.len()}");
    assert_eq!(fmt, "val: {}");
    assert_eq!(exprs, vec!["items.len()"]);
}

#[test]
fn interpolation_nested_braces() {
    let (fmt, exprs) = parse_interpolation("val: {items.iter().filter(|x| { x.active }).count()}");
    assert_eq!(fmt, "val: {}");
    assert_eq!(exprs, vec!["items.iter().filter(|x| { x.active }).count()"]);
}

#[test]
fn interpolation_escaped_braces() {
    let (fmt, exprs) = parse_interpolation("use {{braces}}");
    assert_eq!(fmt, "use {{braces}}");
    assert!(exprs.is_empty());
}
