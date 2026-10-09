use super::super::keywords::ElementKind;
use super::{parse, parse_err};
use quote::quote;

// ── Error cases ──

#[test]
fn error_unknown_element() {
    let err = parse_err(quote! { foobar });
    assert!(err.contains("unknown element `foobar`"), "got: {err}");
}

#[test]
fn error_missing_modifier_value() {
    let err = parse_err(quote! { div gap });
    assert!(err.contains("requires a value"), "got: {err}");
}

#[test]
fn error_unknown_event() {
    let err = parse_err(quote! { button { on doubleclick { } } });
    assert!(err.contains("unknown event"), "got: {err}");
}

// ── "Did you mean?" suggestions ──

#[test]
fn suggest_element_typo_dive() {
    let err = parse_err(quote! { dive });
    assert!(err.contains("Did you mean `div`"), "got: {err}");
}

#[test]
fn suggest_element_typo_colum() {
    let err = parse_err(quote! { colum });
    assert!(err.contains("Did you mean `col`"), "got: {err}");
}

#[test]
fn suggest_element_typo_buton() {
    let err = parse_err(quote! { buton });
    assert!(err.contains("Did you mean `button`"), "got: {err}");
}

#[test]
fn suggest_element_typo_ttext() {
    let err = parse_err(quote! { ttext });
    assert!(err.contains("Did you mean `text`"), "got: {err}");
}

#[test]
fn suggest_modifier_typo_bgg() {
    let err = parse_err(quote! { div bgg "blue" });
    assert!(err.contains("Did you mean `bg`"), "got: {err}");
}

#[test]
fn suggest_modifier_typo_fontt_size() {
    let err = parse_err(quote! { div fontt_size 24 });
    assert!(err.contains("Did you mean `font_size`"), "got: {err}");
}

#[test]
fn suggest_modifier_typo_opacty() {
    let err = parse_err(quote! { div opacty 0.5 });
    assert!(err.contains("Did you mean `opacity`"), "got: {err}");
}

#[test]
fn suggest_event_typo_clik() {
    let err = parse_err(quote! { button { on clik { } } });
    assert!(err.contains("Did you mean `click`"), "got: {err}");
}

#[test]
fn suggest_event_typo_scrolll() {
    let err = parse_err(quote! { button { on scrolll { } } });
    assert!(err.contains("Did you mean `scroll`"), "got: {err}");
}

#[test]
fn error_event_js_style_onclick() {
    let err = parse_err(quote! { button { onclick { } } });
    assert!(
        err.contains("unknown element") || err.contains("unknown keyword"),
        "got: {err}"
    );
}

#[test]
fn suggest_block_keyword_typo_shw() {
    let err = parse_err(quote! { text { shw "Hello" } });
    assert!(err.contains("Did you mean `show`"), "got: {err}");
}

#[test]
fn suggest_block_keyword_typo_eac() {
    let err = parse_err(quote! { div { eac item in { items } { text { show "x" } } } });
    assert!(err.contains("Did you mean `each`"), "got: {err}");
}

#[test]
fn error_modifier_as_element() {
    let err = parse_err(quote! { bg "blue" });
    assert!(err.contains("is a modifier, not an element"), "got: {err}");
}

#[test]
fn error_modifier_missing_value_bg() {
    let err = parse_err(quote! { div bg });
    assert!(err.contains("requires a value"), "got: {err}");
}

#[test]
fn error_modifier_missing_value_opacity() {
    let err = parse_err(quote! { div opacity });
    assert!(err.contains("opacity 0.5"), "got: {err}");
}

#[test]
fn error_unknown_event_with_suggestion_list() {
    let err = parse_err(quote! { button { on xyz { } } });
    assert!(err.contains("Expected one of"), "got: {err}");
}

#[test]
fn error_unknown_element_suggests_pascalcase() {
    let err = parse_err(quote! { flarble });
    assert!(err.contains("PascalCase"), "got: {err}");
}

// ── Complex / integration ──

#[test]
fn counter_example() {
    let block = parse(quote! {
        col centered, gap 4, p 8, bg "slate-900" {
            text font_size 24, text_color "white" {
                show "Counter Demo"
            }
            row centered, gap 4 {
                button px 6, py 3, bg "blue-500", rounded "xl" {
                    on click { set_count.update(|n| *n += 1) }
                    show "Increment"
                }
            }
            text font_size 48, bold, text_color "white" {
                show "Count: {count}"
            }
        }
    });

    assert_eq!(block.elements.len(), 1);
    let root = &block.elements[0];
    assert!(matches!(root.kind, ElementKind::Col));
    assert_eq!(root.modifiers.len(), 4);

    let body = root.body.as_ref().unwrap();
    assert_eq!(body.len(), 3);
}

#[test]
fn conditional_iteration_example() {
    let _block = parse(quote! {
        col gap 2 {
            when { items.get().is_empty() } {
                text text_color "gray" { show "No items" }
            }
            each item in { items.get() } {
                row p 2, bg "gray-800", rounded "md" {
                    text { show "{item.name}" }
                    spacer
                    button bg "red-500" {
                        on click { remove_item(item.id) }
                        show "X"
                    }
                }
            }
        }
    });
}
