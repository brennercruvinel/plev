mod errors;

use super::block_item::BlockItem;
use super::keywords::{ElementKind, ModifierKey};
use super::value::ModifierValue;
use super::*;
use quote::quote;

pub(super) fn parse(tokens: proc_macro2::TokenStream) -> NarrateBlock {
    syn::parse2(tokens).expect("parse failed")
}

pub(super) fn parse_err(tokens: proc_macro2::TokenStream) -> String {
    syn::parse2::<NarrateBlock>(tokens)
        .expect_err("expected parse error")
        .to_string()
}

// ── Element parsing ──

#[test]
fn bare_div() {
    let block = parse(quote! { div });
    assert_eq!(block.elements.len(), 1);
    assert!(matches!(block.elements[0].kind, ElementKind::Div));
    assert!(block.elements[0].modifiers.is_empty());
    assert!(block.elements[0].body.is_none());
}

#[test]
fn row_element() {
    let block = parse(quote! { row });
    assert!(matches!(block.elements[0].kind, ElementKind::Row));
}

#[test]
fn col_element() {
    let block = parse(quote! { col });
    assert!(matches!(block.elements[0].kind, ElementKind::Col));
}

#[test]
fn custom_element_pascal_case() {
    let block = parse(quote! { MyComponent });
    assert!(matches!(&block.elements[0].kind, ElementKind::Custom(id) if id == "MyComponent"));
}

// ── Modifier parsing ──

#[test]
fn single_modifier_with_string_value() {
    let block = parse(quote! { div bg "blue" });
    assert_eq!(block.elements[0].modifiers.len(), 1);
    assert_eq!(block.elements[0].modifiers[0].key, ModifierKey::Bg);
    assert!(
        matches!(&block.elements[0].modifiers[0].value, Some(ModifierValue::Str(s)) if s.value() == "blue")
    );
}

#[test]
fn single_modifier_with_int_value() {
    let block = parse(quote! { div gap 4 });
    assert_eq!(block.elements[0].modifiers[0].key, ModifierKey::Gap);
    assert!(matches!(
        &block.elements[0].modifiers[0].value,
        Some(ModifierValue::Int(_))
    ));
}

#[test]
fn flag_modifier() {
    let block = parse(quote! { div bold });
    assert_eq!(block.elements[0].modifiers[0].key, ModifierKey::Bold);
    assert!(block.elements[0].modifiers[0].value.is_none());
}

#[test]
fn multiple_modifiers_with_commas() {
    let block = parse(quote! { col centered, gap 4, p 8, bg "slate-900" });
    let mods = &block.elements[0].modifiers;
    assert_eq!(mods.len(), 4);
    assert_eq!(mods[0].key, ModifierKey::Center);
    assert_eq!(mods[1].key, ModifierKey::Gap);
    assert_eq!(mods[2].key, ModifierKey::P);
    assert_eq!(mods[3].key, ModifierKey::Bg);
}

#[test]
fn multiple_modifiers_without_commas() {
    let block = parse(quote! { text font_size 24 bold text_color "white" });
    let mods = &block.elements[0].modifiers;
    assert_eq!(mods.len(), 3);
    assert_eq!(mods[0].key, ModifierKey::FontSize);
    assert_eq!(mods[1].key, ModifierKey::Bold);
    assert_eq!(mods[2].key, ModifierKey::TextColor);
}

#[test]
fn modifier_with_expr_value() {
    let block = parse(quote! { div gap { spacing + 2 } });
    assert!(matches!(
        &block.elements[0].modifiers[0].value,
        Some(ModifierValue::Expr(_))
    ));
}

#[test]
fn modifier_with_float_value() {
    let block = parse(quote! { div opacity 0.5 });
    assert!(matches!(
        &block.elements[0].modifiers[0].value,
        Some(ModifierValue::Float(_))
    ));
}

// ── Body / nesting ──

#[test]
fn element_with_empty_body() {
    let block = parse(quote! { div {} });
    assert!(block.elements[0].body.as_ref().unwrap().is_empty());
}

#[test]
fn nested_elements() {
    let block = parse(quote! {
        div bg "blue" {
            text { show "Hello" }
        }
    });
    let body = block.elements[0].body.as_ref().unwrap();
    assert_eq!(body.len(), 1);
    assert!(matches!(&body[0], BlockItem::Element(e) if matches!(e.kind, ElementKind::Text)));
}

#[test]
fn deeply_nested() {
    let block = parse(quote! {
        div {
            div {
                div {
                    text { show "deep" }
                }
            }
        }
    });
    let l1 = block.elements[0].body.as_ref().unwrap();
    let BlockItem::Element(l1e) = &l1[0] else {
        panic!("expected element");
    };
    let l2 = l1e.body.as_ref().unwrap();
    let BlockItem::Element(l2e) = &l2[0] else {
        panic!("expected element");
    };
    let l3 = l2e.body.as_ref().unwrap();
    assert!(matches!(&l3[0], BlockItem::Element(e) if matches!(e.kind, ElementKind::Text)));
}

// ── Show ──

#[test]
fn show_string() {
    let block = parse(quote! { text { show "Hello" } });
    let body = block.elements[0].body.as_ref().unwrap();
    assert!(
        matches!(&body[0], BlockItem::Show(s) if matches!(&s.value, ModifierValue::Str(lit) if lit.value() == "Hello"))
    );
}

#[test]
fn show_expr() {
    let block = parse(quote! { text { show { count.get() } } });
    let body = block.elements[0].body.as_ref().unwrap();
    assert!(matches!(&body[0], BlockItem::Show(s) if matches!(&s.value, ModifierValue::Expr(_))));
}

// ── On ──

#[test]
fn on_click_no_params() {
    let block = parse(quote! { button { on click { do_thing() } } });
    let body = block.elements[0].body.as_ref().unwrap();
    let BlockItem::On(on) = &body[0] else {
        panic!("expected on");
    };
    assert!(matches!(on.event, super::keywords::EventKind::Click));
    assert!(on.params.is_none());
}

#[test]
fn on_click_with_params() {
    let block = parse(quote! { button { on click |e| { handle(e) } } });
    let body = block.elements[0].body.as_ref().unwrap();
    let BlockItem::On(on) = &body[0] else {
        panic!("expected on");
    };
    assert_eq!(on.params.as_ref().unwrap().to_string(), "e");
}

// ── When ──

#[test]
fn when_simple() {
    let block = parse(quote! {
        div {
            when { items.is_empty() } {
                text { show "Empty" }
            }
        }
    });
    let body = block.elements[0].body.as_ref().unwrap();
    let BlockItem::When(when) = &body[0] else {
        panic!("expected when");
    };
    assert_eq!(when.body.len(), 1);
    assert!(when.otherwise.is_none());
}

#[test]
fn when_with_otherwise() {
    let block = parse(quote! {
        div {
            when { flag } {
                text { show "Yes" }
            } otherwise {
                text { show "No" }
            }
        }
    });
    let body = block.elements[0].body.as_ref().unwrap();
    let BlockItem::When(when) = &body[0] else {
        panic!("expected when");
    };
    assert!(when.otherwise.is_some());
    assert_eq!(when.otherwise.as_ref().unwrap().len(), 1);
}

// ── Each ──

#[test]
fn each_simple() {
    let block = parse(quote! {
        div {
            each item in { items.get() } {
                text { show "item" }
            }
        }
    });
    let body = block.elements[0].body.as_ref().unwrap();
    let BlockItem::Each(each) = &body[0] else {
        panic!("expected each");
    };
    assert_eq!(each.binding.to_string(), "item");
    assert!(each.key.is_none());
}

#[test]
fn each_keyed() {
    let block = parse(quote! {
        div {
            each item in { items.get() } keyed by { item.id } {
                text { show "item" }
            }
        }
    });
    let body = block.elements[0].body.as_ref().unwrap();
    let BlockItem::Each(each) = &body[0] else {
        panic!("expected each");
    };
    assert!(each.key.is_some());
}

// ── Bind ──

#[test]
fn bind_stmt() {
    let block = parse(quote! {
        div {
            bind value to { signal }
        }
    });
    let body = block.elements[0].body.as_ref().unwrap();
    let BlockItem::Bind(bind) = &body[0] else {
        panic!("expected bind");
    };
    assert_eq!(bind.target.to_string(), "value");
}

// ── Multiple top-level ──

#[test]
fn multiple_top_level_elements() {
    let block = parse(quote! {
        div bg "blue" {}
        div bg "red" {}
    });
    assert_eq!(block.elements.len(), 2);
}
