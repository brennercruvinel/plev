//! Tests for the narrate runtime interpreter: tokenizer and parser.

use super::*;
use crate::view::{View, ViewContext};

mod extraction_tests;

fn render(el: &Element) -> Vec<crate::compositor::SceneNode> {
    el.render(&mut ViewContext::new(800.0, 600.0))
}

// ── Tokenizer ──

#[test]
fn tokenize_basic() {
    let tokens = tokenize(r#"div bg "blue" gap 4"#);
    assert_eq!(
        tokens,
        vec![
            Token::Ident("div".into()),
            Token::Ident("bg".into()),
            Token::Str("blue".into()),
            Token::Ident("gap".into()),
            Token::Int(4),
        ]
    );
}

#[test]
fn tokenize_float() {
    let tokens = tokenize("opacity 0.5");
    assert_eq!(
        tokens,
        vec![Token::Ident("opacity".into()), Token::Float(0.5),]
    );
}

#[test]
fn tokenize_braces_and_commas() {
    let tokens = tokenize("div { text {} }");
    assert_eq!(
        tokens,
        vec![
            Token::Ident("div".into()),
            Token::OpenBrace,
            Token::Ident("text".into()),
            Token::OpenBrace,
            Token::CloseBrace,
            Token::CloseBrace,
        ]
    );
}

#[test]
fn tokenize_string_escapes() {
    let tokens = tokenize(r#""hello \"world\"""#);
    assert_eq!(tokens, vec![Token::Str(r#"hello "world""#.into())]);
}

// ── Parser: elements ──

#[test]
fn parse_simple_div() {
    let el = parse_narrate("div").unwrap();
    let nodes = render(&el);
    assert!(nodes.is_empty()); // div without bg = no visual
}

#[test]
fn parse_div_with_bg_and_size() {
    let el = parse_narrate(r#"div bg "blue" w 100 h 50"#).unwrap();
    let nodes = render(&el);
    assert_eq!(nodes.len(), 1);
}

#[test]
fn parse_text_with_show() {
    let el = parse_narrate(r#"text { show "Hello" }"#).unwrap();
    let nodes = render(&el);
    assert_eq!(nodes.len(), 1);
    if let crate::compositor::SceneNode::Text { key, .. } = &nodes[0] {
        assert_eq!(key.text, "Hello");
    } else {
        panic!("expected Text node");
    }
}

#[test]
fn parse_col_with_children() {
    let el = parse_narrate(
        r#"
        col gap 4, p 8, bg "dark_gray" {
            text font_size 24, text_color "white" {
                show "Hello"
            }
        }
    "#,
    )
    .unwrap();
    let nodes = render(&el);
    assert!(nodes.len() >= 2); // bg rect + text
}

#[test]
fn parse_nested_divs() {
    let el = parse_narrate(
        r#"
        div bg "dark_gray" w 400 h 300 {
            div bg "blue" w 200 h 100 {
                text { show "Nested" }
            }
        }
    "#,
    )
    .unwrap();
    let nodes = render(&el);
    assert!(nodes.len() >= 3);
}

#[test]
fn parse_row_layout() {
    let el = parse_narrate(
        r#"
        row gap 10 {
            text { show "Left" }
            text { show "Right" }
        }
    "#,
    )
    .unwrap();
    let nodes = render(&el);
    assert_eq!(nodes.len(), 2);
}

#[test]
fn parse_spacer() {
    let el = parse_narrate(
        r#"
        row {
            text { show "A" }
            spacer
            text { show "B" }
        }
    "#,
    )
    .unwrap();
    let nodes = render(&el);
    assert_eq!(nodes.len(), 2); // spacer has no visual output
}

#[test]
fn parse_modifiers_without_commas() {
    let el = parse_narrate(r#"text font_size 24 bold text_color "white" { show "Hi" }"#).unwrap();
    let nodes = render(&el);
    assert_eq!(nodes.len(), 1);
}

#[test]
fn parse_rounded_string_preset() {
    let el = parse_narrate(r#"div bg "blue" w 100 h 50 rounded "xl""#).unwrap();
    let nodes = render(&el);
    assert_eq!(nodes.len(), 1);
    assert!(matches!(
        &nodes[0],
        crate::compositor::SceneNode::RoundedRect { corner_radius, .. }
            if (*corner_radius - 12.0).abs() < 0.01
    ));
}

#[test]
fn parse_rounded_numeric() {
    let el = parse_narrate(r#"div bg "blue" w 100 h 50 rounded 8"#).unwrap();
    let nodes = render(&el);
    assert_eq!(nodes.len(), 1);
    assert!(matches!(
        &nodes[0],
        crate::compositor::SceneNode::RoundedRect { corner_radius, .. }
            if (*corner_radius - 8.0).abs() < 0.01
    ));
}

// ── Parser: skipped blocks ──

#[test]
fn parse_skips_on_block() {
    let el = parse_narrate(
        r#"
        button bg "blue" {
            on click { do_thing() }
            show "Click"
        }
    "#,
    )
    .unwrap();
    let nodes = render(&el);
    assert!(nodes.len() >= 1, "button bg should produce at least 1 node");
}

#[test]
fn parse_skips_on_with_params() {
    let el = parse_narrate(
        r#"
        button bg "blue" {
            on click |e| { handle(e) }
            show "Click"
        }
    "#,
    )
    .unwrap();
    let nodes = render(&el);
    assert!(nodes.len() >= 1, "button bg should produce at least 1 node");
}

#[test]
fn parse_skips_when_block() {
    let el = parse_narrate(
        r#"
        div bg "blue" w 100 h 50 {
            when { flag } {
                text { show "Yes" }
            } otherwise {
                text { show "No" }
            }
        }
    "#,
    )
    .unwrap();
    let nodes = render(&el);
    assert_eq!(
        nodes.len(),
        1,
        "div bg+size without when children = 1 rect node"
    );
}

#[test]
fn parse_skips_each_block() {
    let el = parse_narrate(
        r#"
        div bg "blue" w 100 h 50 {
            each item in { items.get() } keyed by { item.id } {
                text { show "item" }
            }
        }
    "#,
    )
    .unwrap();
    let nodes = render(&el);
    assert_eq!(
        nodes.len(),
        1,
        "div bg+size without each children = 1 rect node"
    );
}

#[test]
fn parse_skips_bind() {
    let el = parse_narrate(
        r#"
        div bg "blue" w 100 h 50 {
            bind value to { signal }
        }
    "#,
    )
    .unwrap();
    let nodes = render(&el);
    assert_eq!(
        nodes.len(),
        1,
        "div bg+size without bind effect = 1 rect node"
    );
}

#[test]
fn parse_skips_expression_values() {
    let el = parse_narrate(
        r#"
        div gap { spacing + 2 } bg "blue" w 100 h 50
    "#,
    )
    .unwrap();
    let nodes = render(&el);
    assert_eq!(nodes.len(), 1); // bg still applied
}

// ── Empty / error cases ──

#[test]
fn parse_empty_returns_none() {
    assert!(parse_narrate("").is_none());
}

#[test]
fn parse_unknown_returns_none() {
    assert!(parse_narrate("foobar").is_none());
}
